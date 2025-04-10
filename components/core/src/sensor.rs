use std::time::Duration;

use serde::Deserialize;
use duration_str::deserialize_option_duration;

// TODO: Provide standard way to create a sensor
// macro_rules! SensorBase {
//     (#[derive($($derive:meta),*)] $pub:vis struct $name:ident { $($fpub:vis $field:ident : $type:ty,)* }) => {
//         #[derive($($derive),*)]
//         $pub struct $name {
//             // required for all sensors
//             pub id: Option<String>,
//             pub name: String,
//             pub icon: Option<String>,
//             pub device_class: Option<String>,
//             pub unit_of_measurement: String,
//             pub state_class: String,
//             #[serde(deserialize_with = "deserialize_option_duration")]
//             pub update_interval: Option<Duration>,
//             $($fpub $field : $type,)*
//         }
//         // impl $name {
//         //     $pub fn new(age:i64,$($field:$type,)*) -> Self{
//         //         Self{
//         //             age,
//         //             $($field,)*
//         //         }
//         //     }
//         // }
//     }
// }

// SensorBase! {
//     #[derive(Debug)]
//     pub struct Sensor {
//         #[serde(flatten)]
//         pub kind: SensorKind,
//     }
// }

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
    Shell(ShellSensorConfig),
    BME280(BME280SensorConfig)
}

#[derive(Clone, Deserialize, Debug)]
pub struct ShellSensorConfig {
    pub command: String
}

#[derive(Clone, Deserialize, Debug)]
pub struct BME280SensorConfig {
    pub bla: String
}