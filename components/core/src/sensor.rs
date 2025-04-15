use std::time::Duration;

use serde::Deserialize;
use duration_str::deserialize_option_duration;


#[derive(Clone, Deserialize, Debug)]
pub struct SensorBase {
    pub id: Option<String>,
    pub name: String,
    pub icon: Option<String>,
    pub device_class: Option<String>,
    pub unit_of_measurement: Option<String>,
    pub state_class: Option<String>,
    // #[serde(deserialize_with = "deserialize_option_duration")]
    // pub update_interval: Option<Duration>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct UnknownSensor{}
