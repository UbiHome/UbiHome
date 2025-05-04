use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Component {
    Button(UbiButton),
    Sensor(UbiSensor),
    BinarySensor(UbiBinarySensor),
    Switch(UbiSwitch),
}

// Icons: https://pictogrammers.com/library/mdi/

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UbiButton {
    pub name: String,
    pub icon: Option<String>,
    pub platform: String,
    pub id: String,
}

// https://developers.home-assistant.io/docs/core/entity/sensor/
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UbiSensor {
    pub name: String,
    pub platform: String,
    pub icon: Option<String>,
    pub state_class: Option<String>,
    pub device_class: Option<String>,
    pub unit_of_measurement: Option<String>,
    pub id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UbiBinarySensor {
    pub name: String,
    pub platform: String,
    pub icon: Option<String>,
    pub device_class: Option<String>,
    pub id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UbiSwitch {
    pub name: String,
    pub platform: String,
    pub icon: Option<String>,
    pub device_class: Option<String>,
    pub id: String,
}

// impl HABinarySensor {
//     pub fn new(name: String, platform: String, icon: Option<String>, device_class: Option<String>, unique_id: String, object_id: String) -> Self {
//         HABinarySensor {
//             name,
//             platform,
//             icon,
//             device_class,
//             unique_id,
//             object_id,
//         }
//     }
// }