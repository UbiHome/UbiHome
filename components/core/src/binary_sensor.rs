use std::time::Duration;

use serde::Deserialize;
use duration_str::deserialize_option_duration;

#[derive(Clone, Debug, Deserialize)]
pub struct BinarySensor {
    // pub platform: String,
    pub id: Option<String>,
    pub name: String,
    pub icon: Option<String>,
    pub device_class: Option<String>,
    
    pub filters: Option<Vec<String>>,

    #[serde(flatten)]
    pub kind: BinarySensorKind,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "platform")]
#[serde(rename_all = "camelCase")]
pub enum BinarySensorKind {
    Shell(ShellBinarySensorConfig),
    Gpio(GpioBinarySensorConfig),
}

#[derive(Clone, Deserialize, Debug)]
pub struct ShellBinarySensorConfig {
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub update_interval: Option<Duration>,
    pub command: String
}

#[derive(Clone, Deserialize, Debug)]
pub struct GpioBinarySensorConfig {
    pub pin: u8, // TODO: Use GPIO types or library
    pub pull_up: Option<bool>,
}