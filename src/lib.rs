use serde::Deserialize;

// Define a struct to represent the YAML configuration
#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub mqtt_broker: String,
    pub mqtt_port: u16,
}
#[derive(Clone, Debug)]
pub struct AppState {
    pub custom_directory: String,
    pub config: Config,
}
