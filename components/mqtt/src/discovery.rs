use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_tuple::{Deserialize_tuple, Serialize_tuple};

// For the abbreviations look here:
// https://www.home-assistant.io/integrations/mqtt/#supported-abbreviations-in-mqtt-discovery-messages

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum HAMqttComponent {
    Button(HAMqttButton),
    Sensor(HAMqttSensor),
    BinarySensor(HAMqttBinarySensor),
    Switch(HAMqttSwitch),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct HAMqttSwitch {
    #[serde(rename = "p")]
    pub(crate) platform: String,

    #[serde(rename = "uniq_id")]
    pub(crate) unique_id: String,

    #[serde(rename = "name")]
    pub(crate) name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ic")]
    pub(crate) icon: Option<String>,

    #[serde(rename = "obj_id")]
    pub(crate) object_id: String,

    #[serde(rename = "stat_t")]
    pub(crate) state_topic: String,

    #[serde(rename = "cmd_t")]
    pub(crate) command_topic: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct HAMqttButton {
    #[serde(rename = "p")]
    pub(crate) platform: String,

    #[serde(rename = "uniq_id")]
    pub(crate) unique_id: String,

    #[serde(rename = "name")]
    pub(crate) name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ic")]
    pub(crate) icon: Option<String>,

    #[serde(rename = "obj_id")]
    pub(crate) object_id: String,

    #[serde(rename = "cmd_t")]
    pub(crate) command_topic: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct HAMqttSensor {
    #[serde(rename = "p")]
    pub(crate) platform: String,

    #[serde(rename = "uniq_id")]
    pub(crate) unique_id: String,

    #[serde(rename = "name")]
    pub(crate) name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ic")]
    pub(crate) icon: Option<String>,

    #[serde(rename = "obj_id")]
    pub(crate) object_id: String,

    #[serde(rename = "stat_t")]
    pub(crate) state_topic: String,

    #[serde(rename = "dev_cla")]
    pub(crate) device_class: String,

    #[serde(rename = "unit_of_meas")]
    pub(crate) unit_of_measurement: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct HAMqttBinarySensor {
    #[serde(rename = "p")]
    pub(crate) platform: String,

    #[serde(rename = "uniq_id")]
    pub(crate) unique_id: String,

    #[serde(rename = "name")]
    pub(crate) name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ic")]
    pub(crate) icon: Option<String>,

    #[serde(rename = "obj_id")]
    pub(crate) object_id: String,

    #[serde(rename = "stat_t")]
    pub(crate) state_topic: String,

    #[serde(rename = "dev_cla")]
    pub(crate) device_class: String,
}

#[derive(Clone, Serialize_tuple, Deserialize_tuple, Debug)]
pub(crate) struct HAMqttConnection {
    pub(crate) r#type: String,
    pub(crate) value: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct HAMqttDevice {
    #[serde(rename = "ids")]
    pub(crate) identifiers: Vec<String>,
    #[serde(rename = "mf")]
    pub(crate) manufacturer: String,
    #[serde(rename = "name")]
    pub(crate) name: String,
    #[serde(rename = "mdl")]
    pub(crate) model: String,
    #[serde(rename = "cns")]
    pub(crate) connections: Vec<HAMqttConnection>,
    // sw_version
    // NEXT: Show http web server?
    // configuration_url
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct HAMqttOrigin {
    pub(crate) name: String,
    pub(crate) sw: String,
    pub(crate) url: String,
}

/// https://www.home-assistant.io/integrations/mqtt/#discovery-messages
#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct HAMqttDiscoveryMessage {
    pub(crate) device: HAMqttDevice,
    pub(crate) origin: HAMqttOrigin,
    pub(crate) components: HashMap<String, HAMqttComponent>,
}
