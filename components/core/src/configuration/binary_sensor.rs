use duration_str::deserialize_duration;
use garde::Validate;
use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;

// use crate::constants::is_id_string_option;
// use crate::constants::is_readable_string;
// use crate::{utils::format_id, with_base_entity_properties};

#[derive(Clone, Serialize, Deserialize, Debug, Validate)]
#[serde(rename_all = "lowercase")]
pub enum FilterType {
    Invert(#[garde(required)] Option<String>),

    #[serde(deserialize_with = "deserialize_duration")]
    DelayedOff(#[garde(skip)] Duration),

    #[serde(deserialize_with = "deserialize_duration")]
    DelayedOn(#[garde(skip)] Duration),
}

#[derive(Clone, Serialize, Deserialize, Debug, Validate)]
#[serde(deny_unknown_fields)]
pub struct BinarySensorFilter {
    #[serde(flatten)]
    #[garde(skip)]
    pub filter: FilterType,
}

#[derive(Clone, Serialize, Deserialize, Debug, Validate)]
pub enum ActionType {
    #[serde(rename = "switch.turn_on")]
    SwitchTurnOn(#[garde(ascii)] String),

    #[serde(rename = "switch.turn_off")]
    SwitchTurnOff(#[garde(ascii)] String),
}

#[derive(Clone, Serialize, Deserialize, Debug, Validate)]
#[serde(deny_unknown_fields)]

pub struct Action {
    #[serde(flatten)]
    #[garde(skip)]
    pub action: ActionType,
}

#[derive(Clone, Serialize, Deserialize, Debug, Validate)]
#[serde(deny_unknown_fields)]

pub struct Trigger {
    #[garde(dive)]
    pub then: Vec<Action>,
}

// with_base_entity_properties! {

//     #[derive(Clone, Deserialize, Debug, Validate)]
//     #[serde(deny_unknown_fields)]
//     pub struct BinarySensorBase {
//         #[garde(dive)]
//         pub filters: Option<Vec<BinarySensorFilter>>,
//         #[garde(dive)]
//         pub on_press: Option<Trigger>,
//         #[garde(dive)]
//         pub on_release: Option<Trigger>,
//     }
// }

// #[derive(Clone, Deserialize, Debug, Validate)]
// pub struct UnknownBinarySensor {}

// #[macro_export]
// macro_rules! template_binary_sensor {
//     ($component_name:ident, $binary_sensor_extension:ident) => {
//         use $crate::binary_sensor::BinarySensorBase;
//         use $crate::binary_sensor::UnknownBinarySensor;

//         #[allow(non_camel_case_types)]
//         #[derive(Clone, Deserialize, Debug, Validate)]
//         #[serde(tag = "platform")]
//         #[serde(rename_all = "camelCase")]
//         pub enum BinarySensorKind {
//             $component_name(#[garde(dive)] $binary_sensor_extension),
//             #[serde(untagged)]
//             Unknown(#[garde(dive)] UnknownBinarySensor),
//         }

//         #[derive(Clone, Debug, Deserialize, Validate)]
//         pub struct BinarySensor {
//             #[serde(flatten)]
//             #[garde(dive)]
//             pub default: BinarySensorBase,

//             #[serde(flatten)]
//             #[garde(dive)]
//             pub extra: BinarySensorKind,
//         }
//     };
// }
