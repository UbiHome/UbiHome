use serde::Deserialize;

use crate::utils::format_id;

#[derive(Clone, Deserialize, Debug)]
pub struct ButtonBase {
    pub id: Option<String>,
    pub icon: Option<String>,
    pub name: String,
}

impl ButtonBase {
    pub fn get_object_id(&self, base_name: &String) -> String {
        format_id(base_name, &self.id, &self.name)
    }
}


#[derive(Clone, Deserialize, Debug)]
pub struct UnknownButton{}

#[macro_export]
macro_rules! template_button {
    ($component_name:ident, $button_extension:ident) => {
        use $crate::button::ButtonBase;
        use $crate::button::UnknownButton;

        #[allow(non_camel_case_types)]
        #[derive(Clone, Deserialize, Debug)]
        #[serde(tag = "platform")]
        #[serde(rename_all = "camelCase")]
        pub enum ButtonKind {
            $component_name($button_extension),
            #[serde(untagged)]
            Unknown(UnknownButton),
        }

        #[derive(Clone, Deserialize, Debug)]
        pub struct ButtonConfig {
            #[serde(flatten)]
            pub default: ButtonBase,

            #[serde(flatten)]
            pub extra: ButtonKind,
        }
    }
}