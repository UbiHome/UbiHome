use oshome_gpio::GpioConfig;
use oshome_web_server::WebServerConfig;
use serde::Deserialize;
use oshome_mqtt::MqttConfig;
use oshome_shell::ShellConfig;

#[derive(Clone, Deserialize, Debug)]
pub struct OSHome {
    pub name: String
}

#[derive(Clone, Deserialize, Debug)]
pub struct Logger {
    pub level: LogLevel
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace
}

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub oshome: OSHome,
    pub logger: Option<Logger>,

    // pub button: Option<Vec<ButtonConfig>>,
    // pub binary_sensor: Option<Vec<BinarySensor>>,
    // pub web_server: Option<WebServerConfig>,
    // pub gpio: Option<GpioConfig>,
}

