use std::collections::HashMap;

use log::debug;
use oshome_core::{binary_sensor::BinarySensorBase, template_mapper, OSHome};
use serde::Deserialize;
use serde::Deserializer;

#[derive(Clone, Deserialize, Debug)]
pub enum LogLevel {
    #[serde(alias = "error", alias = "ERROR")]
    Error,
    #[serde(alias = "warn", alias = "WARN")]
    Warn,
    #[serde(alias = "info", alias = "INFO")]
    Info,
    #[serde(alias = "debug", alias = "DEBUG")]
    Debug,
    #[serde(alias = "trace", alias = "TRACE")]
    Trace,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Logger {
    pub level: LogLevel,
    pub directory: Option<String>,
    pub logs: Option<HashMap<String, LogLevel>>,
}

impl Logger {
    pub fn get_flexi_logger_spec(&self) -> String {
        let mut spec = String::new().to_owned();

        match self {
            Logger {
                level: LogLevel::Error,
                ..
            } => spec.push_str("error"),
            Logger {
                level: LogLevel::Warn,
                ..
            } => spec.push_str("warn"),
            Logger {
                level: LogLevel::Info,
                ..
            } => spec.push_str("info"),
            Logger {
                level: LogLevel::Debug,
                ..
            } => spec.push_str("debug"),
            Logger {
                level: LogLevel::Trace,
                ..
            } => spec.push_str("trace"),
        }
        
        let mut logs  = self.logs.clone().unwrap_or(HashMap::new());
        if !logs.contains_key("libmdns") {
            logs.insert("libmdns".to_string(), LogLevel::Info);
        }

        for (log, level) in logs.iter() {
            match level {
                LogLevel::Error => spec.push_str(&format!(",{}=error", log)),
                LogLevel::Warn => spec.push_str(&format!(",{}=warn", log)),
                LogLevel::Info => spec.push_str(&format!(",{}=info", log)),
                LogLevel::Debug => spec.push_str(&format!(",{}=debug", log)),
                LogLevel::Trace => spec.push_str(&format!(",{}=trace", log)),
            }
        }

        return spec;
    }
}


#[derive(Clone, Debug, Deserialize)]
pub struct BinarySensor {
    #[serde(flatten)]
    pub default: BinarySensorBase,
}

template_mapper!(map_binary_sensor, BinarySensor);


#[derive(Clone, Deserialize, Debug)]
pub struct CoreConfig {
    pub oshome: OSHome,
    pub logger: Option<Logger>,

    // #[serde(default, deserialize_with = "map_button")]
    // pub button: Option<HashMap<String, ButtonConfig>>,

    // #[serde(default, deserialize_with = "map_sensor")]
    // pub sensor: Option<HashMap<String, Sensor>>,

    
    #[serde(default, deserialize_with = "map_binary_sensor")]
    pub binary_sensor: Option<HashMap<String, BinarySensor>>,
}
