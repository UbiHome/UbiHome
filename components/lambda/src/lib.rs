use log::{debug, error};
use rquickjs::{Context, Ctx, Function, Runtime};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{future::Future, pin::Pin};
use tokio::{
    sync::broadcast::{Receiver, Sender},
    time,
};
use ubihome_core::internal::sensors::{UbiBinarySensor, UbiComponent, UbiSensor, UbiTextSensor};
use ubihome_core::state::{EntityState, StateStore};
use ubihome_core::NoConfig;
use ubihome_core::{config_template, ChangedMessage, Module, PublishedMessage};

use duration_str::deserialize_duration;
use ubihome_core::template_binary_sensor;
use ubihome_core::template_sensor;
use ubihome_core::template_text_sensor;
use ubihome_core::with_base_entity_properties;

#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct LambdaConfig {
    /// Maximum wall-clock time a single lambda evaluation may run before it is
    /// interrupted by the engine.
    #[serde(default = "default_timeout")]
    #[serde(deserialize_with = "deserialize_duration")]
    pub timeout: Duration,
}

fn default_timeout() -> Duration {
    Duration::from_secs(5)
}

fn default_interval_none() -> Option<Duration> {
    None
}

template_sensor! {
    #[derive(Clone, Deserialize, Debug, Validate)]
    #[garde(allow_unvalidated)]
    pub struct LambdaSensorConfig {
        pub lambda: String,

        #[serde(default = "default_interval_none")]
        #[serde(deserialize_with = "deserialize_option_duration")]
        pub update_interval: Option<Duration>,
    }
}

template_binary_sensor! {
    #[derive(Clone, Deserialize, Debug, Validate)]
    #[garde(allow_unvalidated)]
    pub struct LambdaBinarySensorConfig {
        pub lambda: String,

        #[serde(default = "default_interval_none")]
        #[serde(deserialize_with = "deserialize_option_duration")]
        pub update_interval: Option<Duration>,
    }
}

template_text_sensor! {
    #[derive(Clone, Deserialize, Debug, Validate)]
    #[garde(allow_unvalidated)]
    pub struct LambdaTextSensorConfig {
        pub lambda: String,

        #[serde(default = "default_interval_none")]
        #[serde(deserialize_with = "deserialize_option_duration")]
        pub update_interval: Option<Duration>,
    }
}

config_template!(
    lambda,
    LambdaConfig,
    NoConfig,
    LambdaBinarySensorConfig,
    LambdaSensorConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    LambdaTextSensorConfig
);

#[derive(Clone, Copy, Debug, PartialEq)]
enum LambdaKind {
    Sensor,
    BinarySensor,
    TextSensor,
}

#[derive(Clone, Debug, PartialEq)]
enum LambdaValue {
    Number(f32),
    Bool(bool),
    Text(String),
}

struct EvalRequest {
    key: String,
    respond: tokio::sync::oneshot::Sender<Result<Option<LambdaValue>, String>>,
}

/// Wraps the lambda body in a function expression. Evaluating the expression
/// compiles the body (surfacing syntax errors) without executing it; appending
/// `()` executes it and yields the `return` value.
fn wrap_lambda(source: &str) -> String {
    format!("(function() {{ {}\n }})", source)
}

fn js_error_message(ctx: &Ctx<'_>, err: rquickjs::Error) -> String {
    match err {
        rquickjs::Error::Exception => {
            let caught = ctx.catch();
            caught
                .as_exception()
                .and_then(|exception| exception.message())
                .unwrap_or_else(|| format!("{:?}", caught))
        }
        other => other.to_string(),
    }
}

/// Compile (but do not run) a lambda to surface syntax errors at config
/// validation time.
fn check_syntax(id: &str, source: &str) -> Result<(), String> {
    let runtime = Runtime::new().map_err(|e| e.to_string())?;
    let context = Context::full(&runtime).map_err(|e| e.to_string())?;
    context.with(|ctx| {
        ctx.eval::<rquickjs::Value, _>(wrap_lambda(source))
            .map(|_| ())
            .map_err(|e| format!("Invalid lambda for '{}': {}", id, js_error_message(&ctx, e)))
    })
}

/// Rewraps the native getters so that a missing value is `null` (instead of
/// the `undefined` that `Option::None` converts to), which is what lambda
/// authors will expect to compare against.
const PRELUDE: &str = r#"
for (const name of ["get_sensor", "get_binary_sensor", "get_text_sensor"]) {
    const native = globalThis["__native_" + name];
    globalThis[name] = (key) => {
        const value = native(key);
        return value === undefined ? null : value;
    };
}
"#;

fn register_host_functions(ctx: &Ctx<'_>, state: StateStore) -> Result<(), rquickjs::Error> {
    let globals = ctx.globals();

    let sensor_state = state.clone();
    globals.set(
        "__native_get_sensor",
        Function::new(ctx.clone(), move |key: String| -> Option<f64> {
            match sensor_state.get(&key) {
                Some(EntityState::Sensor(value)) | Some(EntityState::Number(value)) => {
                    Some(value as f64)
                }
                _ => None,
            }
        })?,
    )?;

    let binary_state = state.clone();
    globals.set(
        "__native_get_binary_sensor",
        Function::new(ctx.clone(), move |key: String| -> Option<bool> {
            match binary_state.get(&key) {
                Some(EntityState::BinarySensor(value)) | Some(EntityState::Switch(value)) => {
                    Some(value)
                }
                _ => None,
            }
        })?,
    )?;

    let text_state = state.clone();
    globals.set(
        "__native_get_text_sensor",
        Function::new(ctx.clone(), move |key: String| -> Option<String> {
            match text_state.get(&key) {
                Some(EntityState::TextSensor(value)) => Some(value),
                _ => None,
            }
        })?,
    )?;

    globals.set(
        "log",
        Function::new(ctx.clone(), |message: String| {
            log::info!("[lambda] {}", message);
        })?,
    )?;

    ctx.eval::<(), _>(PRELUDE)?;

    Ok(())
}

fn eval_lambda(
    ctx: &Ctx<'_>,
    kind: LambdaKind,
    source: &str,
) -> Result<Option<LambdaValue>, String> {
    let code = format!("{}()", wrap_lambda(source));
    let value: rquickjs::Value = ctx.eval(code).map_err(|e| js_error_message(ctx, e))?;

    // Like ESPHome lambdas returning `{}`: no return value means "publish nothing".
    if value.is_undefined() || value.is_null() {
        return Ok(None);
    }

    match kind {
        LambdaKind::Sensor => value
            .as_number()
            .map(|number| Some(LambdaValue::Number(number as f32)))
            .ok_or_else(|| "lambda must return a number (or null to skip)".to_string()),
        LambdaKind::BinarySensor => value
            .as_bool()
            .map(|boolean| Some(LambdaValue::Bool(boolean)))
            .ok_or_else(|| "lambda must return a boolean (or null to skip)".to_string()),
        LambdaKind::TextSensor => value
            .as_string()
            .and_then(|string| string.to_string().ok())
            .map(|string| Some(LambdaValue::Text(string)))
            .ok_or_else(|| "lambda must return a string (or null to skip)".to_string()),
    }
}

/// Owns the QuickJS runtime. QuickJS contexts are not `Send`, so all lambdas of
/// this module are evaluated on this dedicated thread, one request at a time.
/// All lambdas share one context, so top-level `var`/`globalThis` assignments
/// act like ESPHome globals.
fn engine_thread(
    lambdas: HashMap<String, (LambdaKind, String)>,
    state: StateStore,
    timeout: Duration,
    requests: std::sync::mpsc::Receiver<EvalRequest>,
) {
    let runtime = match Runtime::new() {
        Ok(runtime) => runtime,
        Err(e) => {
            error!("Failed to start lambda engine: {}", e);
            return;
        }
    };
    runtime.set_memory_limit(32 * 1024 * 1024);

    let deadline = Arc::new(Mutex::new(Instant::now() + timeout));
    {
        let deadline = deadline.clone();
        runtime.set_interrupt_handler(Some(Box::new(move || {
            Instant::now() > *deadline.lock().unwrap()
        })));
    }

    let context = match Context::full(&runtime) {
        Ok(context) => context,
        Err(e) => {
            error!("Failed to create lambda context: {}", e);
            return;
        }
    };
    if let Err(e) = context.with(|ctx| register_host_functions(&ctx, state)) {
        error!("Failed to register lambda host functions: {}", e);
        return;
    }

    while let Ok(request) = requests.recv() {
        let result = match lambdas.get(&request.key) {
            Some((kind, source)) => {
                *deadline.lock().unwrap() = Instant::now() + timeout;
                context.with(|ctx| eval_lambda(&ctx, *kind, source))
            }
            None => Err(format!("Unknown lambda '{}'", request.key)),
        };
        let _ = request.respond.send(result);
    }
}

pub struct UbiHomePlatform {
    config: LambdaConfig,
    components: Vec<UbiComponent>,
    sensors: HashMap<String, LambdaSensorConfig>,
    binary_sensors: HashMap<String, LambdaBinarySensorConfig>,
    text_sensors: HashMap<String, LambdaTextSensorConfig>,
}

impl Module for UbiHomePlatform {
    fn new(config_string: &str, config_path: &str) -> Result<Self, String> {
        let config =
            ubihome_core::validation::validate_config::<CoreConfig>(config_string, config_path)?;
        debug!("Lambda config: {:?}", config);
        let mut components: Vec<UbiComponent> = Vec::new();

        let mut sensors: HashMap<String, LambdaSensorConfig> = HashMap::new();
        for (_, sensor) in config.sensor.clone().unwrap_or_default() {
            let id = sensor.get_object_id();
            check_syntax(&id, &sensor.lambda)?;
            components.push(UbiComponent::Sensor(UbiSensor {
                platform: "lambda".to_string(),
                icon: sensor.icon.clone(),
                device_class: sensor.device_class.clone(),
                state_class: sensor.state_class.clone(),
                unit_of_measurement: sensor.unit_of_measurement.clone(),
                accuracy_decimals: sensor.accuracy_decimals,
                name: sensor.name.clone().unwrap_or_default(),
                internal: sensor.internal,
                id: id.clone(),
                filters: sensor.filters.clone(),
            }));
            sensors.insert(id.clone(), sensor);
        }

        let mut binary_sensors: HashMap<String, LambdaBinarySensorConfig> = HashMap::new();
        for (_, binary_sensor) in config.binary_sensor.clone().unwrap_or_default() {
            let id = binary_sensor.get_object_id();
            check_syntax(&id, &binary_sensor.lambda)?;
            components.push(UbiComponent::BinarySensor(UbiBinarySensor {
                platform: "lambda".to_string(),
                icon: binary_sensor.icon.clone(),
                device_class: binary_sensor.device_class.clone(),
                name: binary_sensor.name.clone().unwrap_or_default(),
                internal: binary_sensor.internal,
                id: id.clone(),
                on_press: binary_sensor.on_press.clone(),
                on_release: binary_sensor.on_release.clone(),
                filters: binary_sensor.filters.clone(),
            }));
            binary_sensors.insert(id.clone(), binary_sensor);
        }

        let mut text_sensors: HashMap<String, LambdaTextSensorConfig> = HashMap::new();
        for (_, text_sensor) in config.text_sensor.clone().unwrap_or_default() {
            let id = text_sensor.get_object_id();
            check_syntax(&id, &text_sensor.lambda)?;
            components.push(UbiComponent::TextSensor(UbiTextSensor {
                platform: "lambda".to_string(),
                icon: text_sensor.icon.clone(),
                name: text_sensor.name.clone().unwrap_or_default(),
                internal: text_sensor.internal,
                id: id.clone(),
                device_class: text_sensor.device_class.clone(),
            }));
            text_sensors.insert(id.clone(), text_sensor);
        }

        Ok(UbiHomePlatform {
            config: config.lambda,
            components,
            sensors,
            binary_sensors,
            text_sensors,
        })
    }

    fn components(&mut self) -> Vec<UbiComponent> {
        self.components.clone()
    }

    fn run(
        &self,
        sender: Sender<ChangedMessage>,
        _receiver: Receiver<PublishedMessage>,
        state: StateStore,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let timeout = self.config.timeout;
        let sensors = self.sensors.clone();
        let binary_sensors = self.binary_sensors.clone();
        let text_sensors = self.text_sensors.clone();
        Box::pin(async move {
            // Other entities' states (including other lambdas' outputs) are read
            // from the global `StateStore`, which the main application keeps up
            // to date from the message bus.
            let mut lambdas: HashMap<String, (LambdaKind, String)> = HashMap::new();
            for (key, sensor) in &sensors {
                lambdas.insert(key.clone(), (LambdaKind::Sensor, sensor.lambda.clone()));
            }
            for (key, binary_sensor) in &binary_sensors {
                lambdas.insert(
                    key.clone(),
                    (LambdaKind::BinarySensor, binary_sensor.lambda.clone()),
                );
            }
            for (key, text_sensor) in &text_sensors {
                lambdas.insert(
                    key.clone(),
                    (LambdaKind::TextSensor, text_sensor.lambda.clone()),
                );
            }

            let (request_sender, request_receiver) = std::sync::mpsc::channel::<EvalRequest>();
            std::thread::spawn(move || engine_thread(lambdas, state, timeout, request_receiver));

            for (key, sensor) in sensors {
                spawn_lambda_loop(
                    key,
                    sensor.update_interval,
                    request_sender.clone(),
                    sender.clone(),
                    |key, value| match value {
                        LambdaValue::Number(value) => {
                            Some(ChangedMessage::SensorValueChange { key, value })
                        }
                        _ => None,
                    },
                );
            }
            for (key, binary_sensor) in binary_sensors {
                spawn_lambda_loop(
                    key,
                    binary_sensor.update_interval,
                    request_sender.clone(),
                    sender.clone(),
                    |key, value| match value {
                        LambdaValue::Bool(value) => {
                            Some(ChangedMessage::BinarySensorValueChange { key, value })
                        }
                        _ => None,
                    },
                );
            }
            for (key, text_sensor) in text_sensors {
                spawn_lambda_loop(
                    key,
                    text_sensor.update_interval,
                    request_sender.clone(),
                    sender.clone(),
                    |key, value| match value {
                        LambdaValue::Text(value) => {
                            Some(ChangedMessage::TextSensorValueChange { key, value })
                        }
                        _ => None,
                    },
                );
            }
            Ok(())
        })
    }
}

fn spawn_lambda_loop(
    key: String,
    update_interval: Option<Duration>,
    request_sender: std::sync::mpsc::Sender<EvalRequest>,
    sender: Sender<ChangedMessage>,
    to_message: impl Fn(String, LambdaValue) -> Option<ChangedMessage> + Send + 'static,
) {
    let Some(duration) = update_interval else {
        debug!("Lambda {} has no update interval", key);
        return;
    };
    tokio::spawn(async move {
        let mut interval = time::interval(duration);
        loop {
            interval.tick().await;
            let (respond, response) = tokio::sync::oneshot::channel();
            if request_sender
                .send(EvalRequest {
                    key: key.clone(),
                    respond,
                })
                .is_err()
            {
                error!("Lambda engine is no longer running, stopping '{}'", key);
                return;
            }
            match response.await {
                Ok(Ok(Some(value))) => {
                    debug!("Lambda '{}' returned {:?}", key, value);
                    if let Some(message) = to_message(key.clone(), value) {
                        _ = sender.send(message);
                    }
                }
                Ok(Ok(None)) => {
                    debug!("Lambda '{}' returned no value", key);
                }
                Ok(Err(e)) => {
                    error!("Lambda '{}' failed: {}", key, e);
                }
                Err(_) => {
                    error!("Lambda engine dropped the request for '{}'", key);
                    return;
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use ubihome_core::state::StateStoreWriter;

    const CONFIG: &str = r#"
ubihome:
  name: "Test Lambda Config"

lambda:

sensor:
  - platform: lambda
    name: "Total Power"
    update_interval: 10s
    lambda: |
      return get_sensor('power_a') + get_sensor('power_b');

binary_sensor:
  - platform: lambda
    name: "High Usage"
    update_interval: 5s
    lambda: |
      return get_sensor('power_a') > 40;

text_sensor:
  - platform: lambda
    name: "Power Status"
    update_interval: 5s
    lambda: |
      var total = get_sensor('power_a');
      return total > 40 ? "high (" + total + " W)" : "normal";
"#;

    #[test]
    fn test_lambda_config_parsing() {
        let module = UbiHomePlatform::new(CONFIG, "config.yaml").unwrap();
        assert_eq!(module.sensors.len(), 1);
        assert!(module.sensors.contains_key("total_power"));
        assert_eq!(module.binary_sensors.len(), 1);
        assert!(module.binary_sensors.contains_key("high_usage"));
        assert_eq!(module.text_sensors.len(), 1);
        assert!(module.text_sensors.contains_key("power_status"));
    }

    #[test]
    fn test_invalid_lambda_is_rejected_at_parse_time() {
        let config = r#"
ubihome:
  name: "Test Lambda Config"

lambda:

sensor:
  - platform: lambda
    name: "Broken"
    lambda: "return 1 +;"
"#;
        let error = UbiHomePlatform::new(config, "config.yaml").err().unwrap();
        assert!(error.contains("Invalid lambda"), "got: {}", error);
    }

    fn eval_on_engine(
        lambdas: HashMap<String, (LambdaKind, String)>,
        state: StateStore,
        timeout: Duration,
        key: &str,
    ) -> Result<Option<LambdaValue>, String> {
        let (request_sender, request_receiver) = std::sync::mpsc::channel::<EvalRequest>();
        let handle =
            std::thread::spawn(move || engine_thread(lambdas, state, timeout, request_receiver));
        let (respond, response) = tokio::sync::oneshot::channel();
        request_sender
            .send(EvalRequest {
                key: key.to_string(),
                respond,
            })
            .unwrap();
        let result = response.blocking_recv().unwrap();
        drop(request_sender);
        handle.join().unwrap();
        result
    }

    #[test]
    fn test_lambda_reads_state_and_returns_number() {
        let (writer, state) = StateStoreWriter::new(Vec::new());
        writer.set("power_a".to_string(), EntityState::Sensor(20.0));
        writer.set("power_b".to_string(), EntityState::Sensor(22.0));

        let mut lambdas = HashMap::new();
        lambdas.insert(
            "total".to_string(),
            (
                LambdaKind::Sensor,
                "return get_sensor('power_a') + get_sensor('power_b');".to_string(),
            ),
        );

        let result = eval_on_engine(lambdas, state, Duration::from_secs(5), "total").unwrap();
        assert_eq!(result, Some(LambdaValue::Number(42.0)));
    }

    #[test]
    fn test_lambda_returning_null_publishes_nothing() {
        let (_writer, state) = StateStoreWriter::new(Vec::new());
        let mut lambdas = HashMap::new();
        lambdas.insert(
            "silent".to_string(),
            (LambdaKind::Sensor, "return null;".to_string()),
        );

        let result = eval_on_engine(lambdas, state, Duration::from_secs(5), "silent").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_unknown_sensor_reads_as_null() {
        let (_writer, state) = StateStoreWriter::new(Vec::new());
        let mut lambdas = HashMap::new();
        lambdas.insert(
            "guard".to_string(),
            (
                LambdaKind::Sensor,
                "return get_sensor('does_not_exist') === null ? 1 : 2;".to_string(),
            ),
        );

        let result = eval_on_engine(lambdas, state, Duration::from_secs(5), "guard").unwrap();
        assert_eq!(result, Some(LambdaValue::Number(1.0)));
    }

    #[test]
    fn test_runaway_lambda_is_interrupted() {
        let (_writer, state) = StateStoreWriter::new(Vec::new());
        let mut lambdas = HashMap::new();
        lambdas.insert(
            "loop".to_string(),
            (LambdaKind::Sensor, "while (true) {}".to_string()),
        );

        let result = eval_on_engine(lambdas, state, Duration::from_millis(100), "loop");
        assert!(result.is_err(), "runaway lambda must be interrupted");
    }
}
