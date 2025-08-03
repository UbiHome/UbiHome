use serde::Deserialize;

use crate::utils::format_id;

#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct LightBase {
    pub id: Option<String>,
    pub name: String,
    pub icon: Option<String>,
    pub disabled_by_default: Option<bool>,
}

// TODO implement as procedural macro
impl LightBase {
    pub fn get_object_id(&self) -> String {
        format_id(&self.id, &self.name)
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct UnknownLight {}

#[macro_export]
macro_rules! template_light {
    ($component_name:ident, $light_extension:ident) => {
        use $crate::light::LightBase;
        use $crate::light::UnknownLight;

        #[allow(non_camel_case_types)]
        #[derive(Clone, Deserialize, Debug)]
        #[serde(tag = "platform")]
        #[serde(rename_all = "camelCase")]
        pub enum LightKind {
            $component_name($light_extension),
            #[serde(untagged)]
            Unknown(UnknownLight),
        }

        #[derive(Clone, Debug, Deserialize)]
        pub struct Light {
            #[serde(flatten)]
            pub default: LightBase,

            #[serde(flatten)]
            pub extra: LightKind,
        }
    };
}