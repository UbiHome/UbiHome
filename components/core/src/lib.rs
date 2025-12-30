pub mod binary_sensor;
pub mod home_assistant;
pub mod sensor;
pub mod sensor_mapper;
pub mod button;
pub mod switch;
pub mod light;
pub mod mapper;
pub mod utils;
pub mod internal;
pub mod features;
pub extern crate paste;

use home_assistant::sensors::Component;
use internal::sensors::InternalComponent;
use std::{collections::HashMap, pin::Pin};
use tokio::sync::broadcast::{Receiver, Sender};
use serde::{Deserialize};
use garde::Validate;


pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait Module
where
    Self: Send,
{
    /// This is the main entry point for the module.
    fn new(config_string: &String) -> Result<Self, String> where Self: Sized;

    /// This will be called to validate the module configuration and set the module up for the later run command.
    /// It is guaranteed to be be called before the run command.
    /// Do not do any heavy lifting in this function, as it will block the main thread.
    fn components(&mut self) -> Vec<InternalComponent>;

    // This will be called in a separate Thread to run the module and its functionality.
    fn run(
        &self,
        sender: Sender<ChangedMessage>,
        receiver: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>;
}



#[derive(Debug, Clone)]
pub struct BluetoothProxyMessage {
    pub reason: String,
    pub mac: String,
    pub rssi: i16, 
    pub name: String, 
    pub service_uuids: Vec<String>, 
    pub service_data: HashMap<String, Vec<u8>>, 
    pub manufacturer_data: HashMap<String, Vec<u8>>,
}


#[derive(Debug, Clone)]
pub enum ChangedMessage {
    ButtonPress { key: String },
    SwitchStateChange { key: String, state: bool },
    SwitchStateCommand { key: String, state: bool },
    SensorValueChange { key: String, value: f32 },
    BinarySensorValueChange { key: String, value: bool },
    LightStateChange { key: String, state: bool, brightness: Option<f32>, red: Option<f32>, green: Option<f32>, blue: Option<f32> },
    LightStateCommand { key: String, state: bool, brightness: Option<f32>, red: Option<f32>, green: Option<f32>, blue: Option<f32> },
    BluetoothProxyMessage(BluetoothProxyMessage),
}


#[derive(Debug, Clone)]
pub enum PublishedMessage {
    Components { components: Vec<Component> },
    ButtonPressed { key: String },
    SwitchStateChange { key: String, state: bool },
    SwitchStateCommand { key: String, state: bool },
    SensorValueChanged { key: String, value: f32 },
    BinarySensorValueChanged { key: String, value: bool },
    LightStateChange { key: String, state: bool, brightness: Option<f32>, red: Option<f32>, green: Option<f32>, blue: Option<f32> },
    LightStateCommand { key: String, state: bool, brightness: Option<f32>, red: Option<f32>, green: Option<f32>, blue: Option<f32> },
    BluetoothProxyMessage (BluetoothProxyMessage),
}

#[derive(Clone, Deserialize, Debug)]
pub struct NoConfig {}


#[derive(Clone, Deserialize, Debug, Validate)]
#[serde(deny_unknown_fields)]
pub struct UbiHome {
    #[garde(ascii, length(min=3, max=25))]
    pub name: String,
    #[garde(ascii, length(min=3, max=25))]
    pub friendly_name: Option<String>,
    #[garde(ascii, length(max=25))]
    pub area: Option<String>,
}


#[macro_export]
macro_rules! config_template {
    (
        $component_name:ident, 
        $component_config:ty, 
        $button_extension:ident, 
        $binary_sensor_extension:ident, 
        $sensor_extension:ident,
        $switch_extension:ident,
        $light_extension:ident) => {

        use duration_str::deserialize_option_duration;
        use ubihome_core::template_button;
        use ubihome_core::template_binary_sensor;
        use ubihome_core::template_sensor;
        use ubihome_core::template_switch;
        use ubihome_core::template_light;
        use ubihome_core::template_mapper;
        use ubihome_core::UbiHome;
        use garde::Validate;


        template_button!($component_name, $button_extension);
        template_binary_sensor!($component_name, $binary_sensor_extension);
        template_sensor!($component_name, $sensor_extension);
        template_switch!($component_name, $switch_extension);
        template_light!($component_name, $light_extension);
        




        template_mapper!(map_sensor, Sensor);
        template_mapper!(map_button, ButtonConfig);
        template_mapper!(map_binary_sensor, BinarySensor);
        template_mapper!(map_switch, Switch);
        template_mapper!(map_light, Light);


        #[derive(Clone, Deserialize, Debug, Validate)]
        #[garde(allow_unvalidated)]
        pub struct CoreConfig {
            pub ubihome: UbiHome,

            pub $component_name: $component_config,

            #[serde(default, deserialize_with = "map_button")]
            pub button: Option<HashMap<String, ButtonConfig>>,

            #[serde(default, deserialize_with = "map_sensor")]
            pub sensor: Option<HashMap<String, Sensor>>,

            #[serde(default, deserialize_with = "map_binary_sensor")]
            pub binary_sensor: Option<HashMap<String, BinarySensor>>,

            #[serde(default, deserialize_with = "map_switch")]
            pub switch: Option<HashMap<String, Switch>>,

            #[serde(default, deserialize_with = "map_light")]
            pub light: Option<HashMap<String, Light>>,
        }
    };
}

