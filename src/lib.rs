use serde::Deserialize;
use os_home_mqtt::MqttConfig;

#[derive(Clone, Deserialize, Debug)]
pub struct OSHome {
    pub name: String
}

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub mqtt: Option<MqttConfig>,
    pub oshome: OSHome
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub custom_directory: String,
    pub config: Config,
}
