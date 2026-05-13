use garde::Validate;
use serde::Deserialize;

use crate::constants::is_id_string_option;
use crate::constants::is_readable_string;
use crate::with_base_entity_properties;

with_base_entity_properties! {
    #[derive(Clone, Deserialize, Debug, Validate)]
    pub struct LightBase {
        #[garde(skip)]
        pub disabled_by_default: Option<bool>,
    }
}
#[derive(Clone, Deserialize, Debug, Validate)]
pub struct UnknownLight {}

// #[macro_export]
// macro_rules! template_light {
//     ($component_name:ident, $light_extension:ident) => {
//         use $crate::light::LightBase;
//         use $crate::light::UnknownLight;

//         #[allow(non_camel_case_types)]
//         #[derive(Clone, Deserialize, Debug, Validate)]
//         #[serde(tag = "platform")]
//         #[serde(rename_all = "camelCase")]
//         pub enum LightKind {
//             $component_name(#[garde(dive)] $light_extension),
//             #[serde(untagged)]
//             Unknown(#[garde(dive)] UnknownLight),
//         }

//         #[derive(Clone, Debug, Deserialize, Validate)]
//         pub struct Light {
//             #[serde(flatten)]
//             #[garde(dive)]
//             pub default: LightBase,

//             #[serde(flatten)]
//             #[garde(dive)]
//             pub extra: LightKind,
//         }
//     };
// }
