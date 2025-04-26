use oshome_core::OSHome;
use serde::Deserialize;

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
}

impl Logger {
    pub fn get_flexi_logger_spec(&self) -> String {
        match self {
            Logger {
                level: LogLevel::Error,
                ..
            } => "error".to_string(),
            Logger {
                level: LogLevel::Warn,
                ..
            } => "warn".to_string(),
            Logger {
                level: LogLevel::Info,
                ..
            } => "info".to_string(),
            Logger {
                level: LogLevel::Debug,
                ..
            } => "debug".to_string(),
            Logger {
                level: LogLevel::Trace,
                ..
            } => "trace".to_string(),
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct CoreConfig {
    pub oshome: OSHome,
    pub logger: Option<Logger>,
}
