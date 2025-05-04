use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum MqttComponent {
    Button(HAMqttButton),
    Sensor(HAMqttSensor),
    BinarySensor(HAMqttBinarySensor),
    Switch(HAMqttSwitch)
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct HAMqttSwitch {
    #[serde(rename = "p")]
    pub(crate) platform: String,
    pub(crate) unique_id: String,
    pub(crate) command_topic: String,
    pub(crate) name: String,
    pub(crate) object_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct HAMqttButton {
    #[serde(rename = "p")]
    pub(crate) platform: String,
    pub(crate) unique_id: String,
    pub(crate) command_topic: String,
    pub(crate) name: String,
    pub(crate) object_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct HAMqttSensor {
    #[serde(rename = "p")]
    pub(crate) platform: String,
    #[serde(rename = "ic")]
    pub(crate) icon: Option<String>,
    pub(crate) name: String,
    pub(crate) device_class: String,
    pub(crate) unit_of_measurement: String,
    pub(crate) unique_id: String,
    pub(crate) object_id: String,
    pub(crate) state_topic: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct HAMqttBinarySensor {
    #[serde(rename = "p")]
    pub(crate) platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ic")]
    pub(crate) icon: Option<String>,
    pub(crate) name: String,
    pub(crate) device_class: String,
    pub(crate) unique_id: String,
    pub(crate) object_id: String,
    pub(crate) state_topic: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct Device {
    pub(crate) identifiers: Vec<String>,
    pub(crate) manufacturer: String,
    pub(crate) name: String,
    pub(crate) model: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct Origin {
    pub(crate) name: String,
    pub(crate) sw: String,
    pub(crate) url: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct MqttDiscoveryMessage {
    pub(crate) device: Device,
    pub(crate) origin: Origin,
    pub(crate) components: HashMap<String, MqttComponent>,
}