use os_home_core::ButtonConfig;
use serde::Deserialize;
use os_home_mqtt::MqttConfig;
use os_home_shell::ShellConfig;

#[derive(Clone, Deserialize, Debug)]
pub struct OSHome {
    pub name: String
}

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub mqtt: Option<MqttConfig>,
    pub shell: Option<ShellConfig>,
    pub oshome: OSHome,
    pub button: Option<Vec<ButtonConfig>>
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub custom_directory: String,
    pub config: Config,
}
