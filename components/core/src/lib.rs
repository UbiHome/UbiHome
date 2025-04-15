pub mod binary_sensor;
pub mod home_assistant;
pub mod sensor;
pub mod sensor_mapper;
pub mod button;
pub mod mapper;
pub extern crate paste;

use home_assistant::sensors::Component;
use std::pin::Pin;
use tokio::sync::broadcast::{Receiver, Sender};
use serde::{Deserialize, Deserializer};


pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait Module
where
    Self: Send,
{
    fn validate(&mut self) -> Result<(), String>;

    fn init(&mut self) -> Result<Vec<Component>, String>;
    fn run(
        &self,
        sender: Sender<ChangedMessage>,
        receiver: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>;
}



#[derive(Debug, Clone)]
pub enum ChangedMessage {
    ButtonPress { key: String },
    SensorValueChange { key: String, value: String },
    BinarySensorValueChange { key: String, value: bool },
}


#[derive(Debug, Clone)]
pub enum PublishedMessage {
    Components { components: Vec<Component> },
    ButtonPressed { key: String },
    SensorValueChanged { key: String, value: String },
    BinarySensorValueChanged { key: String, value: bool },
}

#[derive(Clone, Deserialize, Debug)]
pub struct NoConfig {}


#[derive(Clone, Deserialize, Debug)]
pub struct OSHome {
    pub name: String,
}


#[macro_export]
macro_rules! config_template {
    (
        $component_name:ident, 
        $component_config:ty, 
        $button_extension:ident, 
        $binary_sensor_extension:ident, 
        $sensor_extension:ident) => {

        use duration_str::deserialize_option_duration;
        use oshome_core::template_button;
        use oshome_core::template_binary_sensor;
        use oshome_core::template_sensor;
        use oshome_core::template_mapper;
        use oshome_core::OSHome;


        template_button!($component_name, $button_extension);
        template_binary_sensor!($component_name, $binary_sensor_extension);
        template_sensor!($component_name, $sensor_extension);
        
        #[derive(Clone, Deserialize, Debug)]
        pub struct Logger {
            pub level: LogLevel
        }
        
        #[derive(Clone, Deserialize, Debug)]
        #[serde(rename_all = "UPPERCASE")]
        pub enum LogLevel {
            Error,
            Warn,
            Info,
            Debug,
            Trace
        }



        template_mapper!(map_sensor, Sensor);
        template_mapper!(map_button, ButtonConfig);
        template_mapper!(map_binary_sensor, BinarySensor);


        #[derive(Clone, Deserialize, Debug)]
        pub struct CoreConfig {
            pub oshome: OSHome,
            pub logger: Option<Logger>,

            pub $component_name: $component_config,

            #[serde(default, deserialize_with = "map_button")]
            pub button: Option<HashMap<String, ButtonConfig>>,

            #[serde(default, deserialize_with = "map_sensor")]
            pub sensor: Option<HashMap<String, Sensor>>,

            #[serde(default, deserialize_with = "map_binary_sensor")]
            pub binary_sensor: Option<HashMap<String, BinarySensor>>,
        }
    };
}
