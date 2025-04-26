use serde::Deserialize;

use crate::utils::format_id;

#[derive(Clone, Deserialize, Debug)]
pub struct BinarySensorBase {
    pub id: Option<String>,
    pub name: String,
    pub icon: Option<String>,
    pub device_class: Option<String>,

    pub filters: Option<Vec<String>>,
}

// TODO implement as procedural macro
impl BinarySensorBase {
    pub fn get_object_id(&self, base_name: &String) -> String {
        format_id(base_name, &self.id, &self.name)
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







