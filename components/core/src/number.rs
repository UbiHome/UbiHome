use serde::Deserialize;

use crate::utils::format_id;

#[derive(Clone, Deserialize, Debug)]
pub struct NumberBase {
    pub id: Option<String>,
    pub name: String,
    pub icon: Option<String>,
    pub device_class: Option<String>,
    pub unit_of_measurement: Option<String>,
    pub state_class: Option<String>,
    pub min_value: Option<f32>,
    pub max_value: Option<f32>,
    pub step: Option<f32>,
}

// TODO implement as procedural macro
impl NumberBase {
    pub fn get_object_id(&self) -> String {
        format_id(&self.id, &self.name)
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct UnknownNumber {}

#[macro_export]
macro_rules! template_number {
    ($component_name:ident, $number_extension:ident) => {
        use $crate::number::NumberBase;
        use $crate::number::UnknownNumber;

        #[allow(non_camel_case_types)]
        #[derive(Clone, Deserialize, Debug)]
        #[serde(tag = "platform")]
        #[serde(rename_all = "camelCase")]
        pub enum NumberKind {
            $component_name($number_extension),
            #[serde(untagged)]
            Unknown(UnknownNumber),
        }

        #[derive(Clone, Debug, Deserialize)]
        pub struct Number {
            #[serde(flatten)]
            pub default: NumberBase,

            #[serde(flatten)]
            pub extra: NumberKind,
        }
    };
}
