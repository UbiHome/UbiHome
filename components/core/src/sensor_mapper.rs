#[macro_export]
macro_rules! template_sensor {
    ($component_name:ident, $sensor_extension:ident) => {
        use $crate::sensor::SensorBase;
        use $crate::sensor::UnknownSensor;

        #[allow(non_camel_case_types)]
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
            #[serde(flatten)]
            pub default: SensorBase,

            #[serde(flatten)]
            pub extra: SensorKind,
        }
    };
}
