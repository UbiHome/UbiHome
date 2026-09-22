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
    /// The initial value used to seed the runtime store. An `initial_value`
    /// that doesn't match `value_type` is a configuration error already
    /// caught by [`validate_initial_value`] during config validation, so
    /// reaching a mismatch here means validation has a gap - fail loudly
    /// instead of silently falling back to a default that would mask it.
    pub fn initial(&self) -> GlobalValue {
        match self.initial_value.clone() {
            Some(value) => coerce_value(&self.value_type, value).unwrap_or_else(|e| {
                panic!(
                    "global '{}': invalid initial_value should have been rejected during config validation: {}",
                    self.id, e
                )
            }),
            None => default_value(&self.value_type),
        }
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
/// whose `lambda` reads it via `id()`).
#[derive(Clone)]
pub struct Globals {
    values: Arc<Mutex<HashMap<String, GlobalValue>>>,
    types: Arc<HashMap<String, GlobalType>>,
    /// Notifies readers (like template switch/number `lambda`s) that *some*
    /// global changed, so they re-evaluate. The changed id/value isn't
    /// carried: a `lambda` is arbitrary JS that may read any number of
    /// globals (or none), so a reader can't tell in advance which changes
    /// are relevant to it and just re-evaluates on every notification.
    changes: broadcast::Sender<()>,
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

    /// Set a global's value. Unknown ids and type mismatches are logged as
    /// errors (so they surface during testing) and ignored so a single bad
    /// action never brings the runtime down.
    pub fn set(&self, id: &str, value: GlobalValue) {
        let Some(value_type) = self.types.get(id) else {
            log::error!("globals.set: unknown global id '{}'", id);
            return;
        };
        let value = match coerce_value(value_type, value) {
            Ok(value) => value,
            Err(e) => {
                log::error!("globals.set: invalid value for global '{}': {}", id, e);
                return;
            }
        };
        {
            let mut values = self.values.lock().unwrap();
            log::debug!("globals.set: {} = {}", id, value);
            values.insert(id.to_string(), value.clone());
        }
        // Notify readers; an error just means nobody is currently subscribed.
        let _ = self.changes.send(());
    }

    /// Current value of a global, if it exists.
    pub fn get(&self, id: &str) -> Option<GlobalValue> {
        self.values.lock().unwrap().get(id).cloned()
    }

    /// The declared `type:` of a global, if it exists. Used by the script
    /// engine's `set_global` to reconcile a JS number into `Int` vs `Float`.
    pub fn type_of(&self, id: &str) -> Option<GlobalType> {
        self.types.get(id).cloned()
    }

    /// Subscribe to global-change notifications.
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.changes.subscribe()
    }
}
