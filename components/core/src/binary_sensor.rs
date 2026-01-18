use duration_str::deserialize_duration;
use garde::Validate;
use std::{collections::HashMap, time::Duration};

use serde::Deserialize;

use crate::constants::is_id_string_option;
use crate::constants::is_readable_string;
use crate::{utils::format_id, with_base_entity_properties};
#[derive(Clone, Deserialize, Debug, Validate)]
pub enum FilterType {
    invert(#[garde(required)] Option<String>),

    #[serde(deserialize_with = "deserialize_duration")]
    delayed_off(#[garde(skip)] Duration),

    #[serde(deserialize_with = "deserialize_duration")]
    delayed_on(#[garde(skip)] Duration),
}

#[derive(Clone, Deserialize, Debug, Validate)]
#[serde(deny_unknown_fields)]

pub struct BinarySensorFilter {
    #[serde(flatten)]
    #[garde(skip)]
    pub filter: FilterType,
}

#[derive(Clone, Deserialize, Debug, Validate)]
pub enum ActionType {
    #[serde(rename = "switch.turn_on")]
    switch_turn_on(#[garde(ascii)] String),

    #[serde(rename = "switch.turn_off")]
    switch_turn_off(#[garde(ascii)] String),
}

#[derive(Clone, Deserialize, Debug, Validate)]
#[serde(deny_unknown_fields)]

pub struct Action {
    #[serde(flatten)]
    #[garde(skip)]
    pub action: ActionType,
}

#[derive(Clone, Deserialize, Debug, Validate)]
#[serde(deny_unknown_fields)]

pub struct Trigger {
    #[garde(dive)]
    pub then: Vec<Action>,
}

with_base_entity_properties! {

    #[derive(Clone, Deserialize, Debug, Validate)]
    #[serde(deny_unknown_fields)]
    pub struct BinarySensorBase {
        #[garde(dive)]
        pub filters: Option<Vec<BinarySensorFilter>>,
        #[garde(dive)]
        pub on_press: Option<Trigger>,
        #[garde(dive)]
        pub on_release: Option<Trigger>,
    }
}

// // TODO implement as procedural macro
// impl BinarySensorBase {
//     pub fn get_object_id(&self) -> String {
//         format_id(&self.id, &self.name)
//     }
// }

#[derive(Clone, Deserialize, Debug, Validate)]
pub struct UnknownBinarySensor {}

#[macro_export]
macro_rules! template_binary_sensor {
    ($component_name:ident, $binary_sensor_extension:ident) => {
        use $crate::binary_sensor::BinarySensorBase;
        use $crate::binary_sensor::UnknownBinarySensor;

        #[allow(non_camel_case_types)]
        #[derive(Clone, Deserialize, Debug, Validate)]
        #[serde(tag = "platform")]
        #[serde(rename_all = "camelCase")]
        pub enum BinarySensorKind {
            $component_name(#[garde(dive)] $binary_sensor_extension),
            #[serde(untagged)]
            Unknown(#[garde(dive)] UnknownBinarySensor),
        }

        #[derive(Clone, Debug, Deserialize, Validate)]
        pub struct BinarySensor {
            #[serde(flatten)]
            #[garde(dive)]
            pub default: BinarySensorBase,

            #[serde(flatten)]
            #[garde(dive)]
            pub extra: BinarySensorKind,
        }
    };
}
