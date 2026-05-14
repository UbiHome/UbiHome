use crate::components::{configure_platforms, initialize_platforms, run_platforms, Platform};
use crate::config::{get_platforms_from_config, BaseConfig, BaseConfigContext};
use flexi_logger::writers::FileLogWriter;
use flexi_logger::{detailed_format, Age, Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming};

use ubihome_core::configuration::binary_sensor::{ActionType, FilterType};
use ubihome_core::configuration::sensor::SensorFilterType;
use ubihome_core::internal::sensors::UbiComponent;
use ubihome_core::{ChangedMessage, PublishedMessage};

use futures_signals::signal::{Mutable, SignalExt};
use log::{debug, trace, warn};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::{runtime::Runtime, signal};

fn read_base_config(path: Option<String>) -> Result<String, String> {
    if let Some(path) = path {
        println!("Config: {}", &path);
        let config_file_path = fs::canonicalize(&path).unwrap();
        if let Ok(content) = fs::read_to_string(config_file_path) {
            return Ok(content);
        } else {
            warn!(
                "Failed to read the configuration file at '{}'.", //, falling back to default.",
                &path
            );
        }
    }

    // Fallback to the embedded default configuration
    // println!("Config file path: BUILTIN");
    // printlm!(DEFAULT_CONFIG);
    // DEFAULT_CONFIG
    panic!("oh no!");
}

pub(crate) fn run(
    config_path: Option<String>,
    validate_only: bool,
    shutdown_signal: Option<mpsc::Receiver<()>>,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(debug_assertions))]
    use directories::BaseDirs;
    #[cfg(not(debug_assertions))]
    let base_dirs = BaseDirs::new().expect("Failed to get base directories");
    #[cfg(not(debug_assertions))]
    let log_directory = base_dirs.data_local_dir();

    #[cfg(debug_assertions)]
    let log_directory = Path::new("./logs");

    #[cfg(not(debug_assertions))]
    let log_level = "info";

    #[cfg(debug_assertions)]
    let log_level = "debug";

    let mut logger_builder = Logger::try_with_env_or_str(log_level).unwrap();

    logger_builder = logger_builder
        .format_for_files(detailed_format)
        .log_to_file(FileSpec::default().directory(log_directory)) // write logs to file
        // .write_mode(WriteMode::BufferAndFlush)
        .append()
        .rotate(
            Criterion::AgeOrSize(Age::Day, 10 * 1024 * 1024),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(7),
        );

    // if cfg!(debug_assertions) {
    logger_builder = logger_builder.duplicate_to_stdout(Duplicate::Debug);
    // }

    let mut logger = logger_builder.start().unwrap();

    println!("LogDirectory: {}", log_directory.display());

    let config_string: String =
        read_base_config(config_path).expect("Failed to load base configuration");

    let platforms = get_platforms_from_config(&config_string);
    debug!("Configured modules: {:?}", &platforms);

    let no_snippet = serde_saphyr::Options {
        with_snippet: false,
        ..Default::default()
    };
    let ctx = BaseConfigContext {
        allowed_platforms: Some(platforms.clone()),
    };
    let validation_result = serde_saphyr::from_str_with_options_context_valid::<BaseConfig>(
        &config_string,
        no_snippet.clone(),
        &ctx,
    );

    if let Err(errors) = validation_result {
        let report = serde_saphyr::miette::to_miette_report(&errors, &config_string, "config.yml");
        return Err(format!("{:?}", report).into());
    }
    let config = validation_result.unwrap();

    if let Some(logger_config) = config.logger.clone() {
        logger
            .reset_flw(&FileLogWriter::builder(
                FileSpec::default().directory(
                    logger_config
                        .clone()
                        .directory
                        .unwrap_or(log_directory.to_string_lossy().to_string()),
                ),
            ))
            .unwrap();

        logger
            .parse_and_push_temp_spec(logger_config.get_flexi_logger_spec())
            .unwrap();
    };

    debug!("BaseConfiguration: {:?}", config);

    let mut platforms_to_load: BTreeSet<Platform> = BTreeSet::new();
    println!("Platforms to load: {:?}", platforms);
    for platform in platforms.iter() {
        if let Ok(platform_enum) = Platform::from_str(platform) {
            platforms_to_load.insert(platform_enum);
        } else {
            return Err(format!(
                r#"Unknown platform specified: {}
Remove the "{}:" entry from your configuration or install the cargo crate containing the platform."#,
                platform, platform
            ).into());
        }
    }
    let configuration_result = configure_platforms(&config_string, &platforms_to_load);
    if let Err(e) = configuration_result {
        return Err(e.into());
    }
    let mut configured_platforms = configuration_result.unwrap();
    log::info!("Loaded {} modules", configured_platforms.len());
    let initialized_platforms = initialize_platforms(&mut configured_platforms).unwrap();

    if validate_only {
        return Ok(());
    }

    // Spawn the root task
    let rt = Runtime::new().unwrap();
    rt.block_on(async {


        let (internal_tx, modules_rx) = broadcast::channel::<PublishedMessage>(16);
        let (modules_tx, mut internal_rx) = broadcast::channel::<ChangedMessage>(16);

        // Double Option Workaround for https://github.com/Pauan/rust-signals/issues/75
        let mut signal_map_binary_sensor: HashMap<String, Mutable<Option<Option<bool>>>> = HashMap::new();
        let mut signal_map_sensor: HashMap<String, Mutable<Option<Option<f32>>>> = HashMap::new();

        for component in initialized_platforms.clone() {
            match component {
                UbiComponent::Button(_button) => {
                    // println!("Button: {:?}", button);
                }
                UbiComponent::Sensor(sensor) => {
                    let mutable: Mutable<Option<Option<f32>>> = Mutable::new(Option::None);
                    signal_map_sensor
                        .insert(sensor.id.clone(), mutable.clone());
                    let internal_tx_clone = internal_tx.clone();

                    let mutable_clone = mutable.clone();
                    tokio::spawn(async move {
                        // println!("Filters: {:?}", binary_sensor.filters);

                        let mut signal = mutable_clone.signal_cloned().boxed();
                        for filter in sensor.filters.unwrap_or_default() {
                            match filter.filter {
                                SensorFilterType::Round(decimals) => {
                                    trace!("round");
                                    signal = signal
                                    .map(move |value| {
                                        if let Some(v) = value.and_then(|v| v) {
                                            // let number: f64 = v.parse().unwrap();
                                            let output: f32 = format!("{:.1$}", v, decimals).parse().unwrap();
                                            debug!("Round: {}", output);
                                            Some(Some(output))
                                        } else {
                                            value
                                        }
                                    })
                                    .boxed();
                                }
                            }
                        }

                        // React to signal changes
                        signal
                            .for_each(|value| {
                                let signal_tx_clone = internal_tx_clone.clone();

                                let key = sensor.id.clone();
                                if let Some(value) = value.and_then(|v| v) {
                                    let pcmd = PublishedMessage::SensorValueChanged {
                                        key,
                                        value,
                                    };
                                    debug!("Publishing command from signal: {:?}", pcmd);

                                    signal_tx_clone.send(pcmd).unwrap();
                                }

                                async move {
                                }
                            })
                            .await;
                    });
                }
                UbiComponent::Switch(_switch) => {
                    // println!("Switch: {:?}", switch);
                }
                UbiComponent::Light(_light) => {
                    // println!("Light: {:?}", light);
                }
                UbiComponent::Number(_number) => {
                    // Numbers don't have filters, state changes are forwarded directly
                }
                UbiComponent::BinarySensor(binary_sensor) => {
                    let mutable: Mutable<Option<Option<bool>>> = Mutable::new(Option::None);
                    signal_map_binary_sensor
                        .insert(binary_sensor.id.clone(), mutable.clone());
                    let internal_tx_clone = internal_tx.clone();

                    let mutable_clone = mutable.clone();
                    tokio::spawn(async move {
                        // println!("Filters: {:?}", binary_sensor.filters);

                        let mut signal = mutable_clone.signal().boxed();
                        for filter in binary_sensor.filters.unwrap_or_default() {
                            match filter.filter {
                                FilterType::DelayedOn(time) => {
                                    trace!("delayed_on");
                                    signal = signal
                                        .map_future(move |value| {
                                            Box::pin(async move {
                                                let value = value.and_then(|v| v);
                                                if let Some(v) = value {
                                                    // Delay on (true) values
                                                    if v {
                                                        tokio::time::sleep(time).await;
                                                        value
                                                    } else {
                                                        value
                                                    }
                                                } else {
                                                    value
                                                }
                                            })
                                        })
                                        .boxed();
                                }
                                FilterType::DelayedOff(time) => {
                                    trace!("delayed_off");
                                    signal = signal
                                        .map_future(move |value| {
                                            Box::pin(async move {
                                                let value = value.and_then(|v| v);
                                                if let Some(v) = value {
                                                    // Delay off (false) values
                                                    if !v {
                                                        tokio::time::sleep(time).await;
                                                        value
                                                    } else {
                                                        value
                                                    }
                                                } else {
                                                    value
                                                }
                                            })
                                        })
                                        .boxed();
                                }
                                FilterType::Invert(_) => {
                                    signal = signal
                                        .map(|value| {
                                            trace!("invert");
                                            if value.is_some() {
                                                Some(Some(!value.and_then(|v| v).unwrap()))
                                            } else {
                                                value
                                            }
                                        })
                                        .boxed();
                                }
                            }
                        }

                        // React to signal changes
                        signal
                            .for_each(|value| {
                                let signal_tx_clone = internal_tx_clone.clone();

                                let key = binary_sensor.id.clone();
                                if let Some(value) = value.and_then(|v| v) {
                                    if value {
                                        if let Some(on_press) = binary_sensor.on_press.clone() {
                                            for action in on_press.then {
                                                match &action.action {
                                                    ActionType::SwitchTurnOn(key) => {
                                                        let pcmd = PublishedMessage::SwitchStateCommand {
                                                            key: key.clone(),
                                                            state: true,
                                                        };
                                                        debug!("Publishing command from action {:?}: {:?}", action.clone(), pcmd);
                                                        internal_tx_clone.send(pcmd).unwrap();
                                                    }
                                                    ActionType::SwitchTurnOff(key) => {
                                                        let pcmd = PublishedMessage::SwitchStateCommand {
                                                            key: key.clone(),
                                                            state: false,
                                                        };
                                                        debug!("Publishing command from action {:?}: {:?}", action.clone(), pcmd);
                                                        internal_tx_clone.send(pcmd).unwrap();
                                                    }
                                                }
                                            }
                                        }
                                    }else{
                                        if let Some(on_release) = binary_sensor.on_release.clone() {
                                            for action in on_release.then {
                                                match &action.action {
                                                    ActionType::SwitchTurnOn(key) => {
                                                        let pcmd = PublishedMessage::SwitchStateCommand {
                                                            key: key.clone(),
                                                            state: true,
                                                        };
                                                        debug!("Publishing command from action {:?}: {:?}", action.clone(), pcmd);
                                                        internal_tx_clone.send(pcmd).unwrap();
                                                    }
                                                    ActionType::SwitchTurnOff(key) => {
                                                        let pcmd = PublishedMessage::SwitchStateCommand {
                                                            key: key.clone(),
                                                            state: false,
                                                        };
                                                        debug!("Publishing command from action {:?}: {:?}", action.clone(), pcmd);
                                                        internal_tx_clone.send(pcmd).unwrap();
                                                    }
                                                }
                                            }
                                        }
                                    }


                                    let pcmd = PublishedMessage::BinarySensorValueChanged {
                                        key,
                                        value,
                                    };
                                    debug!("Publishing command from signal: {:?}", pcmd);

                                    signal_tx_clone.send(pcmd).unwrap();
                                }

                                async move {
                                }
                            })
                            .await;

                        // // Debounce future for "off" values
                        // future
                        //     .for_each(|value| {
                        //         // This code is run for the current value of my_state,
                        //         // and also every time my_state changes
                        //         println!("From signal: {}", value);
                        //         async {}
                        //     })
                        //     .await;
                    });
                }
            }
        }

        let internal_tx_clone = internal_tx.clone();
        tokio::spawn({
            async move {
                while let Ok(cmd) = internal_rx.recv().await {
                    debug!("Received command: {:?}", cmd);
                    let publish_cmd: Option<PublishedMessage> = match cmd {
                        ChangedMessage::SwitchStateChange { key, state } => {
                            Some(PublishedMessage::SwitchStateChange { key, state })
                        }
                        ChangedMessage::SwitchStateCommand { key, state } => {
                            Some(PublishedMessage::SwitchStateCommand { key, state })
                        }
                        ChangedMessage::LightStateChange { key, state, brightness, red, green, blue } => {
                            Some(PublishedMessage::LightStateChange { key, state, brightness, red, green, blue })
                        }
                        ChangedMessage::LightStateCommand { key, state, brightness, red, green, blue } => {
                            Some(PublishedMessage::LightStateCommand { key, state, brightness, red, green, blue })
                        }
                        ChangedMessage::ButtonPress { key } => {
                            Some(PublishedMessage::ButtonPressed { key })
                        }
                        ChangedMessage::SensorValueChange { key, value } => {
                            if let Some(signal) = signal_map_sensor.get(&key) {
                                signal.set(Some(Some(value)));
                            }
                            None
                        }
                        ChangedMessage::BinarySensorValueChange { key, value } => {
                            debug!("BinarySensorValueChange: {}", value);
                            if let Some(signal) = signal_map_binary_sensor.get(&key) {
                                signal.set(Some(Some(value)));
                            }
                            None
                        }
                        ChangedMessage::BluetoothProxyMessage(msg) => {
                            Some(PublishedMessage::BluetoothProxyMessage(msg))
                        }
                        ChangedMessage::NumberValueChange { key, value } => {
                            Some(PublishedMessage::NumberValueChanged { key, value })
                        }
                        ChangedMessage::NumberValueCommand { key, value } => {
                            Some(PublishedMessage::NumberValueCommand { key, value })
                        }
                        ChangedMessage::TextSensorValueChange { key, value } => {
                            Some(PublishedMessage::TextSensorValueChanged { key, value })
                        }
                    };
                    if let Some(pcmd) = publish_cmd {
                        debug!("Publishing command: {:?}", pcmd);
                        internal_tx_clone.send(pcmd).unwrap();
                    }
                }
            }
        });

        run_platforms(configured_platforms, modules_tx.clone(), modules_rx).await;

        println!("Platforms: {:?}", initialized_platforms);
        internal_tx
            .send(PublishedMessage::Components {
                components: initialized_platforms
                    .iter()
                    .map(|c| match c {
                        UbiComponent::Switch(switch) => UbiComponent::Switch(switch.clone()),
                        UbiComponent::Button(button) => UbiComponent::Button(button.clone()),
                        UbiComponent::Sensor(sensor) => UbiComponent::Sensor(sensor.clone()),
                        UbiComponent::BinarySensor(binary_sensor) => {
                            UbiComponent::BinarySensor(binary_sensor.clone())
                        }
                        UbiComponent::Light(light) => UbiComponent::Light(light.clone()),
                        UbiComponent::Number(number) => {
                            UbiComponent::Number(number.clone())
                        }
                        UbiComponent::TextSensor(text_sensor) => {
                            UbiComponent::TextSensor(text_sensor.clone())
                        }
                    })
                    .collect(),
            })
            .unwrap();

        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        if let Some(some_shutdown_signal) = shutdown_signal {
            let shutdown_event = async {
                loop {
                    // Poll shutdown event.
                    match some_shutdown_signal.recv_timeout(Duration::from_secs(1)) {
                        // Break the loop either upon stop or channel disconnect
                        Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => break,

                        // Continue work if no events were received within the timeout
                        Err(mpsc::RecvTimeoutError::Timeout) => (),
                    };
                }
            };
            tokio::select! {
                _ = ctrl_c => {},
                _ = terminate => {},
                _ = shutdown_event => {},
            }
        } else {
            tokio::select! {
                _ = ctrl_c => {},
                _ = terminate => {},
            }
        }
    });
    debug!("Shutdown complete");
    Ok(())
}
