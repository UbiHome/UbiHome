use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Component {
    Button(UbiButton),
    Sensor(UbiSensor),
    BinarySensor(UbiBinarySensor),
    Switch(UbiSwitch),
    Light(UbiLight),
    Number(UbiNumber),
    TextSensor(UbiTextSensor),
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
    pub accuracy_decimals: Option<i32>,
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

// https://developers.home-assistant.io/docs/core/entity/number
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UbiNumber {
    pub name: String,
    pub platform: String,
    pub icon: Option<String>,
    pub id: String,
    pub min_value: f32,
    pub max_value: f32,
    pub step: f32,
    pub unit_of_measurement: Option<String>,
    pub device_class: Option<String>,
    pub mode: i32,
}

// https://developers.home-assistant.io/docs/core/entity/text-sensor
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UbiTextSensor {
    pub name: String,
    pub platform: String,
    pub icon: Option<String>,
    pub id: String,
    pub device_class: Option<String>,
}
