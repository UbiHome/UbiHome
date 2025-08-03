use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Component {
    Button(UbiButton),
    Sensor(UbiSensor),
    BinarySensor(UbiBinarySensor),
    Switch(UbiSwitch),
    Light(UbiLight),
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

// https://developers.home-assistant.io/docs/core/entity/binary-sensor
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UbiBinarySensor {
    pub name: String,
    pub platform: String,
    pub icon: Option<String>,
    pub device_class: Option<String>,
    pub id: String,
}

// https://developers.home-assistant.io/docs/core/entity/switch
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UbiSwitch {
    pub name: String,
    pub platform: String,
    pub icon: Option<String>,
    pub device_class: Option<String>,
    pub id: String,
    // If the state must be assumed or can be determined
    pub assumed_state: bool,
}

// https://developers.home-assistant.io/docs/core/entity/light
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UbiLight {
    pub name: String,
    pub platform: String,
    pub icon: Option<String>,
    pub id: String,
    pub disabled_by_default: bool,
    // Light capabilities
    // pub supports_brightness: bool,
    // pub supports_rgb: bool,
    // pub supports_white_value: bool,
    // pub supports_color_temperature: bool,
}
