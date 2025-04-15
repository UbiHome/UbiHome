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
    pub platform: String,
    pub unique_id: String,
    pub command_topic: String,
    pub name: String,
    pub object_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct HASensor {
    pub platform: String,
    pub icon: Option<String>,
    pub name: String,
    pub device_class: Option<String>,
    pub unit_of_measurement: String,
    pub unique_id: String,
    pub object_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct HABinarySensor {
    pub platform: String,
    pub icon: Option<String>,
    pub name: String,
    pub device_class: Option<String>,
    pub unique_id: String,
    pub object_id: String,
}