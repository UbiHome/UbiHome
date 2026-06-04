use std::collections::HashMap;

use garde::Validate;
use serde::Deserialize;
use ubihome_core::UbiHome;

#[derive(Clone, Deserialize, Debug, Validate)]
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

#[derive(Clone, Deserialize, Debug, Validate)]
pub struct Logger {
    #[garde(dive)]
    pub level: LogLevel,
    #[garde(ascii)]
    pub directory: Option<String>,

    #[garde(skip)]
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

        let mut logs = self.logs.clone().unwrap_or_default();
        if !logs.contains_key("libmdns") {
            logs.insert("libmdns".to_string(), LogLevel::Info);
        }
        if !logs.contains_key("neli") {
            logs.insert("neli".to_string(), LogLevel::Info);
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

        spec
    }
}

// Base configuration structure
#[derive(Clone, Deserialize, Debug, Validate)]
pub struct CoreConfig {
    #[garde(dive)]
    pub ubihome: UbiHome,

    #[garde(dive)]
    pub logger: Option<Logger>,
}
