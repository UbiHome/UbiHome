use garde::Validate;
use serde::Deserialize;

use crate::constants::is_id_string_option;
use crate::constants::is_readable_string;

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SensorFilterType {
    round(usize),
}

#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]

pub struct SensorFilter {
    #[serde(flatten)]
    pub filter: SensorFilterType,
}

use crate::with_base_entity_properties;

with_base_entity_properties! {
    #[derive(Clone, Deserialize, Debug, Validate)]
    pub struct SensorBase {
        #[garde(skip)]
        pub unit_of_measurement: Option<String>,
        #[garde(skip)]
        pub state_class: Option<String>,

        #[garde(skip)]
        pub filters: Option<Vec<SensorFilter>>,
    }
}

#[derive(Clone, Deserialize, Debug, Validate)]
pub struct UnknownSensor {}

#[macro_export]
macro_rules! template_sensor {
    ($component_name:ident, $sensor_extension:ident) => {
        use $crate::sensor::SensorBase;
        use $crate::sensor::UnknownSensor;

        #[allow(non_camel_case_types)]
        #[derive(Clone, Deserialize, Debug, Validate)]
        #[serde(tag = "platform")]
        #[serde(rename_all = "camelCase")]
        pub enum SensorKind {
            $component_name(#[garde(dive)] $sensor_extension),
            #[serde(untagged)]
            Unknown(#[garde(dive)] UnknownSensor),
        }

        #[derive(Clone, Deserialize, Debug, Validate)]
        pub struct Sensor {
            #[garde(dive)]
            #[serde(flatten)]
            pub default: SensorBase,

            #[garde(dive)]
            #[serde(flatten)]
            pub extra: SensorKind,
        }
    };
}
