#[macro_export]
macro_rules! template_binary_sensor {
    ($component_name:ident, $binary_sensor_extension:ident) => {
    
        #[derive(Clone, Deserialize, Debug)]
        pub struct UnknownBinarySensor{}

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
            pub id: Option<String>,
            pub name: String,
            pub icon: Option<String>,
            pub device_class: Option<String>,

            pub filters: Option<Vec<String>>,

            #[serde(flatten)]
            pub extra: BinarySensorKind,
        }
    }
}







