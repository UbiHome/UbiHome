use std::time::Duration;

use serde::Deserialize;
use duration_str::deserialize_option_duration;

#[derive(Clone, Debug, Deserialize)]
pub struct Sensor {
    pub id: Option<String>,
    pub name: String,
    pub icon: Option<String>,
    pub device_class: Option<String>,
    pub unit_of_measurement: String,
    pub state_class: String,
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub update_interval: Option<Duration>,

    #[serde(flatten)]
    pub kind: SensorKind,
}


#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "platform")]
#[serde(rename_all = "camelCase")]
pub enum SensorKind {
    Shell(ShellSensorConfig)
}

#[derive(Clone, Deserialize, Debug)]
pub struct ShellSensorConfig {
    pub command: String
}