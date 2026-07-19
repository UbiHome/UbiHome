use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use garde::Validate;
use serde::Deserialize;
use tokio::sync::broadcast;

/// Supported global variable types.
///
/// Values are stored as strings at runtime; the type is used to validate the
/// `initial_value`/`globals.set` values so configuration mistakes are caught
/// early. Lambdas are intentionally not supported yet, so globals are written
/// via the `globals.set` action and are not readable in expressions.
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

    #[garde(skip)]
    pub initial_value: Option<String>,

    /// Accepted for compatibility with ESPHome. Persisting values across
    /// restarts is not implemented yet, so this currently has no effect.
    #[allow(dead_code)]
    #[serde(default)]
    #[garde(skip)]
    pub restore_value: bool,
}

impl GlobalConfig {
    /// The initial value used to seed the runtime store. Falls back to a
    /// type-appropriate default when no `initial_value` is configured.
    pub fn initial(&self) -> String {
        if let Some(value) = &self.initial_value {
            return value.clone();
        }
        match self.value_type {
            GlobalType::Bool => "false".to_string(),
            GlobalType::Int | GlobalType::Float => "0".to_string(),
            GlobalType::String => String::new(),
        }
    }
}

/// Validate that `value` is parseable as `value_type`.
pub fn validate_value(value_type: &GlobalType, value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    match value_type {
        GlobalType::Bool => match trimmed.to_lowercase().as_str() {
            "true" | "false" => Ok(()),
            _ => Err(format!("expected a boolean (true/false), got '{}'", value)),
        },
        GlobalType::Int => trimmed
            .parse::<i64>()
            .map(|_| ())
            .map_err(|_| format!("expected an integer, got '{}'", value)),
        GlobalType::Float => trimmed
            .parse::<f64>()
            .map(|_| ())
            .map_err(|_| format!("expected a float, got '{}'", value)),
        GlobalType::String => Ok(()),
    }
}

/// Shared runtime store for global variables. Cloned into every task that can
/// execute a `globals.set` action or read a global (e.g. a template switch
/// whose `lambda` is `globals.get`).
#[derive(Clone)]
pub struct Globals {
    values: Arc<Mutex<HashMap<String, String>>>,
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
    pub fn set(&self, id: &str, value: &str) {
        let Some(value_type) = self.types.get(id) else {
            log::warn!("globals.set: unknown global id '{}'", id);
            return;
        };
        if let Err(e) = validate_value(value_type, value) {
            log::warn!("globals.set: invalid value for global '{}': {}", id, e);
            return;
        }
        {
            let mut values = self.values.lock().unwrap();
            log::debug!("globals.set: {} = {}", id, value);
            values.insert(id.to_string(), value.to_string());
        }
        // Notify readers; an error just means nobody is currently subscribed.
        let _ = self.changes.send(id.to_string());
    }

    /// Current value of a global, if it exists.
    #[allow(dead_code)]
    pub fn get(&self, id: &str) -> Option<String> {
        self.values.lock().unwrap().get(id).cloned()
    }

    /// Current value of a global interpreted as a boolean (`globals.get` on a
    /// `bool` global). Returns `None` for unknown ids or non-boolean values.
    pub fn get_bool(&self, id: &str) -> Option<bool> {
        match self.get(id)?.trim().to_lowercase().as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    }

    /// Subscribe to global-change notifications (yields the id of each changed
    /// global).
    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.changes.subscribe()
    }
}
