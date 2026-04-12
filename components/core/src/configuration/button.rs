// use crate::constants::is_id_string_option;
// use crate::constants::is_readable_string;
// use crate::with_base_entity_properties;
// use garde::Validate;
// use serde::Deserialize;

// with_base_entity_properties! {
//     #[derive(Clone, Deserialize, Debug, Validate)]
//     pub struct ButtonBase {
//     }
// }

// #[derive(Clone, Deserialize, Debug, Validate)]
// pub struct UnknownButton {}

// #[macro_export]
// macro_rules! template_button {
//     ($component_name:ident, $button_extension:ident) => {
//         use $crate::button::ButtonBase;
//         use $crate::button::UnknownButton;

//         #[allow(non_camel_case_types)]
//         #[derive(Clone, Deserialize, Debug, Validate)]
//         #[serde(tag = "platform")]
//         #[serde(rename_all = "camelCase")]
//         pub enum ButtonKind {
//             $component_name(#[garde(dive)] $button_extension),
//             #[serde(untagged)]
//             Unknown(#[garde(dive)] UnknownButton),
//         }

//         #[derive(Clone, Deserialize, Debug, Validate)]
//         pub struct ButtonConfig {
//             #[serde(flatten)]
//             #[garde(dive)]
//             pub default: ButtonBase,

//             #[serde(flatten)]
//             #[garde(dive)]
//             pub extra: ButtonKind,
//         }
//     };
// }
