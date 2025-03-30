use oshome_core::{binary_sensor::BinarySensor, button::ButtonConfig};
use oshome_web_server::WebServerConfig;
use serde::Deserialize;
use oshome_mqtt::MqttConfig;
use oshome_shell::ShellConfig;

#[derive(Clone, Deserialize, Debug)]
pub struct OSHome {
    pub name: String
}

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub oshome: OSHome,

    pub mqtt: Option<MqttConfig>,
    pub shell: Option<ShellConfig>,
    pub button: Option<Vec<ButtonConfig>>,
    pub binary_sensor: Option<Vec<BinarySensor>>,
    pub web_server: Option<WebServerConfig>,
}

