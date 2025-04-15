use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct ButtonBase {
    pub id: Option<String>,
    pub name: String,
}


#[derive(Clone, Deserialize, Debug)]
pub struct UnknownButton{}

#[macro_export]
macro_rules! template_button {
    ($component_name:ident, $button_extension:ident) => {
        use $crate::button::ButtonBase;
        use $crate::button::UnknownButton;

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