#[macro_export]
macro_rules! template_button {
    ($component_name:ident, $button_extension:ident) => {

        #[derive(Clone, Deserialize, Debug)]
        #[serde(tag = "platform")]
        #[serde(rename_all = "camelCase")]
        pub enum ButtonKind {
            $component_name($button_extension),
        }

        #[derive(Clone, Deserialize, Debug)]
        pub struct ButtonConfig {
            pub platform: String,
            pub id: Option<String>,
            pub name: String,
            pub command: String,

            #[serde(flatten)]
            pub extra: $button_extension,
        }
    }
}