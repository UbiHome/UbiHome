use serde::Deserialize;

use crate::utils::format_id;

#[derive(Clone, Deserialize, Debug)]
pub struct TextSensorBase {
    pub id: Option<String>,
    pub name: String,
    pub icon: Option<String>,
    pub device_class: Option<String>,
}

// TODO implement as procedural macro
impl TextSensorBase {
    pub fn get_object_id(&self) -> String {
        format_id(&self.id, &self.name)
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct UnknownTextSensor {}

#[macro_export]
macro_rules! template_text_sensor {
    ($component_name:ident, $text_sensor_extension:ident) => {
        use $crate::text_sensor::TextSensorBase;
        use $crate::text_sensor::UnknownTextSensor;

        #[allow(non_camel_case_types)]
        #[derive(Clone, Deserialize, Debug)]
        #[serde(tag = "platform")]
        #[serde(rename_all = "camelCase")]
        pub enum TextSensorKind {
            $component_name($text_sensor_extension),
            #[serde(untagged)]
            Unknown(UnknownTextSensor),
        }

        #[derive(Clone, Debug, Deserialize)]
        pub struct TextSensor {
            #[serde(flatten)]
            pub default: TextSensorBase,

            #[serde(flatten)]
            pub extra: TextSensorKind,
        }
    };
}
