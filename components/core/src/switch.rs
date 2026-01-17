use duration_str::deserialize_duration;
use std::time::Duration;

use serde::Deserialize;

use crate::utils::format_id;

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

#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SwitchBase {
    pub id: Option<String>,
    pub name: String,
    pub icon: Option<String>,
    pub device_class: Option<String>,
    // pub filters: Option<Vec<BinarySensorFilter>>,
}

// TODO implement as procedural macro
impl SwitchBase {
    pub fn get_object_id(&self) -> String {
        format_id(&self.id, &self.name)
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct UnknownSwitch {}

#[macro_export]
macro_rules! template_switch {
    ($component_name:ident, $switch_extension:ident) => {
        use $crate::switch::SwitchBase;
        use $crate::switch::UnknownSwitch;

        #[allow(non_camel_case_types)]
        #[derive(Clone, Deserialize, Debug)]
        #[serde(tag = "platform")]
        #[serde(rename_all = "camelCase")]
        pub enum SwitchKind {
            $component_name($switch_extension),
            #[serde(untagged)]
            Unknown(UnknownSwitch),
        }

        #[derive(Clone, Debug, Deserialize)]
        pub struct Switch {
            #[serde(flatten)]
            pub default: SwitchBase,

            #[serde(flatten)]
            pub extra: SwitchKind,
        }
    };
}
