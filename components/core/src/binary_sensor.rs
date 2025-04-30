use std::time::Duration;
use duration_str::deserialize_duration;

use serde::Deserialize;

use crate::utils::format_id;

#[derive(Clone, Deserialize, Debug)]
pub enum FilterType {
    invert(Option<String>),

    #[serde(deserialize_with = "deserialize_duration")]
    delayed_off(Duration),

    #[serde(deserialize_with = "deserialize_duration")]
    delayed_on(Duration),
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum FilterTypeEnum {
    invert,
    delayed_off,
    delayed_on,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]

pub struct BinarySensorFilter {
    #[serde(flatten)]
    pub filter: FilterType,

}

#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct BinarySensorBase {
    pub id: Option<String>,
    pub name: String,
    pub icon: Option<String>,
    pub device_class: Option<String>,

    pub filters: Option<Vec<BinarySensorFilter>>,
}

// TODO implement as procedural macro
impl BinarySensorBase {
    pub fn get_object_id(&self) -> String {
        format_id( &self.id, &self.name)
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct UnknownBinarySensor{}



#[macro_export]
macro_rules! template_binary_sensor {
    ($component_name:ident, $binary_sensor_extension:ident) => {
    
        use $crate::binary_sensor::BinarySensorBase;
        use $crate::binary_sensor::UnknownBinarySensor;

        #[allow(non_camel_case_types)]
        #[derive(Clone, Deserialize, Debug)]
        #[serde(tag = "platform")]
        #[serde(rename_all = "camelCase")]
        pub enum BinarySensorKind {
            $component_name($binary_sensor_extension),
            #[serde(untagged)]
            Unknown(UnknownBinarySensor),
        }

        #[derive(Clone, Debug, Deserialize)]
        pub struct BinarySensor {
            #[serde(flatten)]
            pub default: BinarySensorBase,

            #[serde(flatten)]
            pub extra: BinarySensorKind,
        }
    }
}







