use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Component {
    Button(HAButton),
    Sensor(HASensor),
    BinarySensor(HABinarySensor),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct HAButton {
    pub name: String,
    pub icon: Option<String>,
    pub platform: String,
    pub unique_id: Option<String>,
    pub object_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct HASensor {
    pub name: String,
    pub platform: String,
    pub icon: Option<String>,
    pub device_class: Option<String>,
    pub unit_of_measurement: Option<String>,
    pub unique_id: Option<String>,
    pub object_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct HABinarySensor {
    pub name: String,
    pub platform: String,
    pub icon: Option<String>,
    pub device_class: Option<String>,
    pub unique_id: Option<String>,
    pub object_id: String,
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