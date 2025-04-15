#[macro_export]
macro_rules! template_sensor {
    ($component_name:ident, $sensor_extension:ident) => {

        #[derive(Clone, Deserialize, Debug)]
        pub struct UnknownSensor{}

        #[derive(Clone, Deserialize, Debug)]
        #[serde(tag = "platform")]
        #[serde(rename_all = "camelCase")]
        pub enum SensorKind {
            $component_name($sensor_extension),
            #[serde(untagged)]
            Unknown(UnknownSensor),
        }

        #[derive(Clone, Deserialize, Debug)]
        pub struct Sensor {
            pub id: Option<String>,
            pub name: String,
            pub icon: Option<String>,
            pub device_class: Option<String>,
            pub unit_of_measurement: String,
            pub state_class: String,
            #[serde(deserialize_with = "deserialize_option_duration")]
            pub update_interval: Option<Duration>,
        
            #[serde(flatten)]
            pub extra: SensorKind,
        }
    }
}