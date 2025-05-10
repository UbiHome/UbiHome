pub mod binary_sensor;
pub mod home_assistant;
pub mod sensor;
pub mod sensor_mapper;
pub mod button;
pub mod event;
pub mod switch;
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
    BluetoothProxyMessage(BluetoothProxyMessage),
    EventChange { key: String , r#type: String },
    APISubscribeEntity { entity: String, attribute: String},
    APISubscribedEntityChange { entity: String, attribute: String, state: String },
}


#[derive(Debug, Clone)]
pub enum PublishedMessage {
    Components { components: Vec<Component> },
    ButtonPressed { key: String },
    SwitchStateChange { key: String, state: bool },
    SwitchStateCommand { key: String, state: bool },
    SensorValueChanged { key: String, value: f32 },
    BinarySensorValueChanged { key: String, value: bool },
    BluetoothProxyMessage (BluetoothProxyMessage),
    EventChange { key: String , r#type: String },
    APISubscribeEntity { entity: String, attribute: String},
    APISubscribedEntityChange { entity: String, attribute: String, state: String },
}

#[derive(Clone, Deserialize, Debug)]
pub struct NoConfig {}


#[derive(Clone, Deserialize, Debug)]
pub struct UbiHome {
    pub name: String,
    pub friendly_name: Option<String>,
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
        $event_extension:ident
    ) => {

        use duration_str::deserialize_option_duration;
        use ubihome_core::template_button;
        use ubihome_core::template_binary_sensor;
        use ubihome_core::template_sensor;
        use ubihome_core::template_switch;
        use ubihome_core::template_event;
        use ubihome_core::template_mapper;
        use ubihome_core::UbiHome;

        template_button!($component_name, $button_extension);
        template_binary_sensor!($component_name, $binary_sensor_extension);
        template_sensor!($component_name, $sensor_extension);
        template_switch!($component_name, $switch_extension);
        template_event!($component_name, $event_extension);

        template_mapper!(map_sensor, Sensor);
        template_mapper!(map_button, ButtonConfig);
        template_mapper!(map_binary_sensor, BinarySensor);
        template_mapper!(map_switch, Switch);
        template_mapper!(map_event, EventConfig);

        #[derive(Clone, Deserialize, Debug)]
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

            #[serde(default, deserialize_with = "map_event")]
            pub event: Option<HashMap<String, EventConfig>>,
        }
    };
}

