use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use garde::Validate;
use serde::Deserialize;
use tokio::sync::broadcast;
use ubihome_core::global_value::GlobalValue;

/// Supported global variable types.
///
/// The runtime value is a typed [`GlobalValue`]; this only declares which
/// variant a given global is meant to hold, so `initial_value`/`globals.set`
/// values can be reconciled against it (see [`coerce_value`]). Lambdas are
/// intentionally not supported yet, so globals are written via the
/// `globals.set` action and are not readable in expressions.
#[derive(Clone, Debug, Deserialize, Validate, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GlobalType {
    Bool,
    Int,
    Float,
    #[serde(alias = "std::string")]
    String,
}

/// Configuration of a single `globals:` entry.
///
/// Mirrors a subset of the ESPHome `globals` component
/// (<https://esphome.io/components/globals/>).
#[derive(Clone, Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct GlobalConfig {
    #[garde(length(min = 3, max = 100))]
    pub id: String,

    #[serde(rename = "type")]
    #[garde(skip)]
    pub value_type: GlobalType,

    /// Must be the YAML scalar matching `value_type` (unquoted for
    /// bool/int/float); see [`coerce_value`].
    #[serde(default)]
    #[garde(custom(validate_initial_value(&self.value_type)))]
    pub initial_value: Option<GlobalValue>,

    /// Accepted for compatibility with ESPHome. Persisting values across
    /// restarts is not implemented yet, so this currently has no effect.
    #[allow(dead_code)]
    #[serde(default)]
    #[garde(skip)]
    pub restore_value: bool,
}

/// Validates that `initial_value` (if given) matches the sibling
/// `value_type` field, reusing [`coerce_value`]. Wired up via `self` access
/// (see the `garde` crate's "Context/Self access" docs) so the error is
/// reported through the same `serde_saphyr`/`garde` pipeline as every other
/// field - with a source line/column - instead of a plain string surfaced
/// after deserialization.
fn validate_initial_value(
    value_type: &GlobalType,
) -> impl FnOnce(&Option<GlobalValue>, &()) -> garde::Result + '_ {
    move |value, _| {
        let Some(value) = value else {
            return Ok(());
        };
        coerce_value(value_type, value.clone())
            .map(|_| ())
            .map_err(garde::Error::new)
    }
}

impl GlobalConfig {
    /// The initial value used to seed the runtime store. Falls back to a
    /// type-appropriate default when no `initial_value` is configured. An
    /// `initial_value` that doesn't match `value_type` is a configuration
    /// error caught by [`validate_initial_value`] during config validation,
    /// so this only needs a defensive fallback.
    pub fn initial(&self) -> GlobalValue {
        if let Some(value) = self.initial_value.clone() {
            match coerce_value(&self.value_type, value) {
                Ok(value) => return value,
                Err(e) => {
                    log::warn!(
                        "global '{}': invalid initial_value, falling back to default: {}",
                        self.id,
                        e
                    );
                }
            }
        }
        default_value(&self.value_type)
    }
}

fn default_value(value_type: &GlobalType) -> GlobalValue {
    match value_type {
        GlobalType::Bool => GlobalValue::Bool(false),
        GlobalType::Int => GlobalValue::Int(0),
        GlobalType::Float => GlobalValue::Float(0.0),
        GlobalType::String => GlobalValue::String(String::new()),
    }
}

/// Reconcile a [`GlobalValue`] against a global's declared `value_type`.
///
/// A value that is already the right variant passes through unchanged (an
/// `int` is also accepted for a `float` global). Anything else - including a
/// string that merely looks like a bool/number, e.g. `"true"` - is a type
/// mismatch; it is not parsed. A `string`-typed global accepts any value,
/// stringified.
pub fn coerce_value(value_type: &GlobalType, value: GlobalValue) -> Result<GlobalValue, String> {
    match (value_type, value) {
        (GlobalType::Bool, GlobalValue::Bool(v)) => Ok(GlobalValue::Bool(v)),
        (GlobalType::Bool, other) => Err(format!("expected a boolean, got '{}'", other)),

        (GlobalType::Int, GlobalValue::Int(v)) => Ok(GlobalValue::Int(v)),
        (GlobalType::Int, other) => Err(format!("expected an integer, got '{}'", other)),

        (GlobalType::Float, GlobalValue::Float(v)) => Ok(GlobalValue::Float(v)),
        (GlobalType::Float, GlobalValue::Int(v)) => Ok(GlobalValue::Float(v as f64)),
        (GlobalType::Float, other) => Err(format!("expected a float, got '{}'", other)),

        (GlobalType::String, GlobalValue::String(v)) => Ok(GlobalValue::String(v)),
        (GlobalType::String, other) => Ok(GlobalValue::String(other.to_string())),
    }
}

/// Shared runtime store for global variables. Cloned into every task that can
/// execute a `globals.set` action or read a global (e.g. a template switch
/// whose `lambda` is `globals.get`).
#[derive(Clone)]
pub struct Globals {
    values: Arc<Mutex<HashMap<String, GlobalValue>>>,
    types: Arc<HashMap<String, GlobalType>>,
    /// Broadcasts the id of a global whenever its value changes, so readers
    /// (like template switch `globals.get` lambdas) can update live.
    changes: broadcast::Sender<String>,
}

impl Globals {
    pub fn new(configs: &[GlobalConfig]) -> Self {
        let mut values = HashMap::new();
        let mut types = HashMap::new();
        for config in configs {
            values.insert(config.id.clone(), config.initial());
            types.insert(config.id.clone(), config.value_type.clone());
        }
        let (changes, _) = broadcast::channel(64);
        Globals {
            values: Arc::new(Mutex::new(values)),
            types: Arc::new(types),
            changes,
        }
    }

    /// Set a global's value. Unknown ids and type mismatches are logged and
    /// ignored so a single bad action never brings the runtime down.
    pub fn set(&self, id: &str, value: GlobalValue) {
        let Some(value_type) = self.types.get(id) else {
            log::warn!("globals.set: unknown global id '{}'", id);
            return;
        };
        let value = match coerce_value(value_type, value) {
            Ok(value) => value,
            Err(e) => {
                log::warn!("globals.set: invalid value for global '{}': {}", id, e);
                return;
            }
        };
        {
            let mut values = self.values.lock().unwrap();
            log::debug!("globals.set: {} = {}", id, value);
            values.insert(id.to_string(), value);
        }
        // Notify readers; an error just means nobody is currently subscribed.
        let _ = self.changes.send(id.to_string());
    }

    /// Current value of a global, if it exists.
    #[allow(dead_code)]
    pub fn get(&self, id: &str) -> Option<GlobalValue> {
        self.values.lock().unwrap().get(id).cloned()
    }

    /// Current value of a global interpreted as a boolean (`globals.get` on a
    /// `bool` global). Returns `None` for unknown ids or non-boolean values.
    pub fn get_bool(&self, id: &str) -> Option<bool> {
        match self.get(id)? {
            GlobalValue::Bool(value) => Some(value),
            _ => None,
        }
    }

    /// Current value of a global interpreted as a float (`globals.get` on a
    /// `float` global). An `int` global is also accepted, widened to `f32`.
    /// Returns `None` for unknown ids or bool/string values.
    pub fn get_float(&self, id: &str) -> Option<f32> {
        match self.get(id)? {
            GlobalValue::Float(value) => Some(value as f32),
            GlobalValue::Int(value) => Some(value as f32),
            _ => None,
        }
    }

    /// Subscribe to global-change notifications (yields the id of each changed
    /// global).
    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.changes.subscribe()
    }
}
