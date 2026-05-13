pub mod configuration;
pub mod constants;
pub mod features;
pub mod internal;
pub mod light;
pub mod mapper;
pub mod utils;
pub extern crate paste;

use garde::Validate;
use internal::sensors::UbiComponent;
use serde::Deserialize;
use std::{collections::HashMap, pin::Pin};
use tokio::sync::broadcast::{Receiver, Sender};

use crate::constants::{is_readable_string, is_readable_string_option};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait Module
where
    Self: Send,
{
    /// This is the main entry point for the module.
    fn new(config_string: &String) -> Result<Self, String>
    where
        Self: Sized;

    /// This will be called to validate the module configuration and set the module up for the later run command.
    /// It is guaranteed to be be called before the run command.
    /// Do not do any heavy lifting in this function, as it will block the main thread.
    fn components(&mut self) -> Vec<UbiComponent>;

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
    ButtonPress {
        key: String,
    },
    SwitchStateChange {
        key: String,
        state: bool,
    },
    SwitchStateCommand {
        key: String,
        state: bool,
    },
    SensorValueChange {
        key: String,
        value: f32,
    },
    BinarySensorValueChange {
        key: String,
        value: bool,
    },
    LightStateChange {
        key: String,
        state: bool,
        brightness: Option<f32>,
        red: Option<f32>,
        green: Option<f32>,
        blue: Option<f32>,
    },
    LightStateCommand {
        key: String,
        state: bool,
        brightness: Option<f32>,
        red: Option<f32>,
        green: Option<f32>,
        blue: Option<f32>,
    },
    NumberValueChange {
        key: String,
        value: f32,
    },
    NumberValueCommand {
        key: String,
        value: f32,
    },
    BluetoothProxyMessage(BluetoothProxyMessage),
}

#[derive(Debug, Clone)]
pub enum PublishedMessage {
    Components {
        components: Vec<UbiComponent>,
    },
    ButtonPressed {
        key: String,
    },
    SwitchStateChange {
        key: String,
        state: bool,
    },
    SwitchStateCommand {
        key: String,
        state: bool,
    },
    SensorValueChanged {
        key: String,
        value: f32,
    },
    BinarySensorValueChanged {
        key: String,
        value: bool,
    },
    LightStateChange {
        key: String,
        state: bool,
        brightness: Option<f32>,
        red: Option<f32>,
        green: Option<f32>,
        blue: Option<f32>,
    },
    LightStateCommand {
        key: String,
        state: bool,
        brightness: Option<f32>,
        red: Option<f32>,
        green: Option<f32>,
        blue: Option<f32>,
    },
    NumberValueChanged {
        key: String,
        value: f32,
    },
    NumberValueCommand {
        key: String,
        value: f32,
    },
    BluetoothProxyMessage(BluetoothProxyMessage),
}

#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct NoConfig {
    pub platform: String,
}

impl NoConfig {
    pub fn is_configured(&self) -> bool {
        false
    }
    pub fn get_object_id(&self) -> String {
        unimplemented!();
    }
}

#[derive(Clone, Deserialize, Debug, Validate)]
#[serde(deny_unknown_fields)]
pub struct UbiHome {
    #[garde(custom(is_readable_string), length(min = 3, max = 100))]
    pub name: String,
    #[garde(custom(is_readable_string_option), length(min = 3, max = 100))]
    pub friendly_name: Option<String>,
    #[garde(custom(is_readable_string_option), length(min = 3, max = 100))]
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
        $light_extension:ident,
        $number_extension:ident) => {
        use duration_str::deserialize_option_duration;
        use garde::Validate;
        use ubihome_core::UbiHome;
        use ubihome_core::template_mapper_new;

        template_mapper_new!(map_light, $component_name, $light_extension);
        template_mapper_new!(map_switch, $component_name, $switch_extension);
        template_mapper_new!(map_number, $component_name, $number_extension);

        template_mapper_new!(map_sensor, $component_name, $sensor_extension);
        template_mapper_new!(map_button, $component_name, $button_extension);
        template_mapper_new!(map_binary_sensor, $component_name, $binary_sensor_extension);

        #[derive(Clone, Deserialize, Debug, Validate)]
        #[garde(allow_unvalidated)]
        pub struct CoreConfig {
            #[garde(dive)]
            pub ubihome: UbiHome,

            #[garde(dive)]
            pub $component_name: $component_config,

            #[serde(default, deserialize_with = "map_button")]
            #[garde(dive)]
            pub button: Option<HashMap<String, $button_extension>>,

            #[serde(default, deserialize_with = "map_sensor")]
            #[garde(dive)]
            pub sensor: Option<HashMap<String, $sensor_extension>>,

            #[serde(default, deserialize_with = "map_binary_sensor")]
            #[garde(dive)]
            pub binary_sensor: Option<HashMap<String, $binary_sensor_extension>>,

            #[serde(default, deserialize_with = "map_switch")]
            #[garde(dive)]
            pub switch: Option<HashMap<String, $switch_extension>>,

            #[serde(default, deserialize_with = "map_light")]
            #[garde(dive)]
            pub light: Option<HashMap<String, $light_extension>>,

            #[serde(default, deserialize_with = "map_number")]
            pub number: Option<HashMap<String, $number_extension>>,
        }
    };
}
