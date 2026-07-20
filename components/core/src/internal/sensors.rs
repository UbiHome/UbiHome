use serde::{Deserialize, Serialize};

use crate::{
    configuration::binary_sensor::{BinarySensorFilter, Trigger},
    configuration::sensor::SensorFilter,
};

#[derive(Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum UbiComponent {
    Button(UbiButton),
    Sensor(UbiSensor),
    BinarySensor(UbiBinarySensor),
    Switch(UbiSwitch),
    Light(UbiLight),
    Number(UbiNumber),
    TextSensor(UbiTextSensor),
}

impl UbiComponent {
    /// An internal component is configured with an `id` but no `name`. It is
    /// wired up internally (filters, actions, state routing) but must not be
    /// exposed by connectivity components such as api, mqtt or http.
    pub fn is_internal(&self) -> bool {
        match self {
            UbiComponent::Button(c) => c.internal,
            UbiComponent::Sensor(c) => c.internal,
            UbiComponent::BinarySensor(c) => c.internal,
            UbiComponent::Switch(c) => c.internal,
            UbiComponent::Light(c) => c.internal,
            UbiComponent::Number(c) => c.internal,
            UbiComponent::TextSensor(c) => c.internal,
        }
    }
}

// Icons: https://pictogrammers.com/library/mdi/

macro_rules! with_base_properties {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident : $field_type:ty
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name {
            pub name: String,
            pub icon: Option<String>,
            pub platform: String,
            pub id: String,
            /// Set when the component was configured with only an `id` (no
            /// `name`). Internal components are hidden from connectivity
            /// components (api, mqtt, http).
            #[serde(default)]
            pub internal: bool,
        // pub state_class: Option<String>,
        // pub device_class: Option<String>,

            $(
                $(#[$field_meta])*
                $field_vis $field_name : $field_type,
            )*
        }
    };
}

with_base_properties! {
    #[derive(Clone, Serialize, Deserialize, Debug)]
    pub struct UbiButton {
    }
}

with_base_properties! {
// https://developers.home-assistant.io/docs/core/entity/sensor/
    #[derive(Clone, Deserialize, Debug)]
    pub struct UbiSensor {
        pub state_class: Option<String>,
        pub device_class: Option<String>,
        pub unit_of_measurement: Option<String>,
        pub accuracy_decimals: Option<i32>,

        pub filters: Option<Vec<SensorFilter>>,
    }
}

with_base_properties! {
// https://developers.home-assistant.io/docs/core/entity/binary-sensor
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UbiBinarySensor {
    pub device_class: Option<String>,
    pub filters: Option<Vec<BinarySensorFilter>>,
    pub on_press: Option<Trigger>,
    pub on_release: Option<Trigger>,
}
}

with_base_properties! {
// https://developers.home-assistant.io/docs/core/entity/switch
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UbiSwitch {
    pub device_class: Option<String>,
    // If the state must be assumed or can be determined
    pub assumed_state: bool,
}
}

with_base_properties! {

// https://developers.home-assistant.io/docs/core/entity/light
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UbiLight {
    pub disabled_by_default: bool,
    // Light capabilities
    // pub supports_brightness: bool,
    // pub supports_rgb: bool,
    // pub supports_white_value: bool,
    // pub supports_color_temperature: bool,
}
}

with_base_properties! {
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UbiNumber {
    pub min_value: f32,
    pub max_value: f32,
    pub step: f32,
    pub unit_of_measurement: Option<String>,
    pub device_class: Option<String>,
    pub mode: i32,
}
}

with_base_properties! {
// https://developers.home-assistant.io/docs/core/entity/text-sensor
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UbiTextSensor {
    pub device_class: Option<String>,
}
}
