use garde::Validate;

use serde::Deserialize;

use crate::constants::is_id_string_option;
use crate::constants::is_readable_string;

// #[derive(Clone, Deserialize, Debug)]
// pub enum FilterType {
//     invert(Option<String>),

//     #[serde(deserialize_with = "deserialize_duration")]
//     delayed_off(Duration),

//     #[serde(deserialize_with = "deserialize_duration")]
//     delayed_on(Duration),
// }

// #[derive(Clone, Deserialize, Debug)]
// #[serde(deny_unknown_fields)]

// pub struct BinarySensorFilter {
//     #[serde(flatten)]
//     pub filter: FilterType,

// }

use crate::with_base_entity_properties;

with_base_entity_properties! {
    #[derive(Clone, Deserialize, Debug, Validate)]
    pub struct SwitchBase {
        // #[garde(skip)]
        // pub filters: Option<Vec<BinarySensorFilter>>,
    }
}

#[derive(Clone, Deserialize, Debug, Validate)]
pub struct UnknownSwitch {}

#[macro_export]
macro_rules! template_switch {
    ($component_name:ident, $switch_extension:ident) => {
        use $crate::switch::SwitchBase;
        use $crate::switch::UnknownSwitch;

        #[allow(non_camel_case_types)]
        #[derive(Clone, Deserialize, Debug, Validate)]
        #[serde(tag = "platform")]
        #[serde(rename_all = "camelCase")]
        pub enum SwitchKind {
            $component_name(#[garde(dive)] $switch_extension),
            #[serde(untagged)]
            Unknown(#[garde(dive)] UnknownSwitch),
        }

        #[derive(Clone, Debug, Deserialize, Validate)]
        // #[garde(transparent)]
        pub struct Switch {
            #[serde(flatten)]
            #[garde(dive)]
            pub default: SwitchBase,

            #[serde(flatten)]
            #[garde(dive)]
            pub extra: SwitchKind,
        }
    };
}
