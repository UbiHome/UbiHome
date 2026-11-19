pub mod configuration;
pub mod constants;
pub mod features;
pub mod global_value;
pub mod internal;
pub mod platform;
pub mod state;
pub mod text_sensor;
pub mod utils;
#[cfg(feature = "validation")]
pub mod validation;
pub extern crate serde_value;
#[doc(hidden)]
pub use paste;

use garde::Validate;
use internal::sensors::UbiComponent;
use serde::Deserialize;
use std::{collections::HashMap, future::Future, pin::Pin};
use tokio::sync::broadcast::{Receiver, Sender};

use crate::configuration::binary_sensor::Trigger;
use crate::constants::{is_readable_string, is_readable_string_option};
use crate::state::StateStore;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
pub type ModuleRunFuture = BoxFuture<'static, Result<(), Box<dyn std::error::Error>>>;

/// A running platform module. Construction from config happens through
/// [`platform::ConfigurableModule`].
pub trait Module
where
    Self: Send,
{
    /// The entities this module provides. Called once after construction and
    /// before [`Module::run`]. Do not do any heavy lifting in this function,
    /// as it will block the main thread.
    fn components(&mut self) -> Vec<UbiComponent>;

    // This will be called in a separate Thread to run the module and its functionality.
    fn run(
        &self,
        sender: Sender<ChangedMessage>,
        receiver: Receiver<PublishedMessage>,
        state: StateStore,
    ) -> ModuleRunFuture;
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
    TextSensorValueChange {
        key: String,
        value: String,
    },
    BluetoothProxyMessage(BluetoothProxyMessage),
    /// Combined so a single message can carry any subset of playback/volume/mute
    /// changes instead of needing one enum variant per field.
    MediaPlayerStateChange {
        key: String,
        playing: Option<bool>,
        volume: Option<f32>,
        muted: Option<bool>,
    },
}

#[derive(Debug, Clone)]
pub enum PublishedMessage {
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
    TextSensorValueChanged {
        key: String,
        value: String,
    },
    BluetoothProxyMessage(BluetoothProxyMessage),
    MediaPlayerStateChanged {
        key: String,
        playing: Option<bool>,
        volume: Option<f32>,
        muted: Option<bool>,
    },
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

/// `NoConfig` marks an entity kind the platform does not provide; validation
/// rejects any entity of that kind pointing at the platform.
impl platform::EntityConfig for NoConfig {
    const SUPPORTED: bool = false;

    fn platform(&self) -> &str {
        &self.platform
    }
    fn object_id(&self) -> String {
        String::new()
    }
    fn triggers(&self) -> Vec<(&'static str, &Trigger)> {
        Vec::new()
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
    /// Actions to run once, when UbiHome starts up.
    #[garde(dive)]
    pub on_startup: Option<Trigger>,
}

/// Declares the config types of a platform crate and generates its
/// `CoreConfig` (the platform's slice of the configuration file).
///
/// `CoreConfig` is not deserialized by the platform crate: the main binary
/// parses and validates the whole file once and builds it via
/// [`platform::PlatformConfigTypes::from_parts`].
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
        $number_extension:ident,
        $text_sensor_extension:ident,
        $media_player_extension:ident) => {
        use duration_str::deserialize_option_duration;
        use garde::Validate;
        use ubihome_core::UbiHome;

        #[derive(Clone, Debug)]
        pub struct CoreConfig {
            pub ubihome: UbiHome,
            pub $component_name: $component_config,
            pub button: Option<HashMap<String, $button_extension>>,
            pub sensor: Option<HashMap<String, $sensor_extension>>,
            pub binary_sensor: Option<HashMap<String, $binary_sensor_extension>>,
            pub switch: Option<HashMap<String, $switch_extension>>,
            pub light: Option<HashMap<String, $light_extension>>,
            pub number: Option<HashMap<String, $number_extension>>,
            pub text_sensor: Option<HashMap<String, $text_sensor_extension>>,
            pub media_player: Option<HashMap<String, $media_player_extension>>,
        }

        impl ::ubihome_core::platform::PlatformConfigTypes for CoreConfig {
            const NAME: &'static str = stringify!($component_name);
            type Section = $component_config;
            type Button = $button_extension;
            type BinarySensor = $binary_sensor_extension;
            type Sensor = $sensor_extension;
            type Switch = $switch_extension;
            type Light = $light_extension;
            type Number = $number_extension;
            type TextSensor = $text_sensor_extension;
            type MediaPlayer = $media_player_extension;

            fn from_parts(parts: ::ubihome_core::platform::PlatformParts<Self>) -> Self {
                use ::ubihome_core::platform::into_entity_map;
                CoreConfig {
                    ubihome: parts.ubihome,
                    $component_name: parts.section,
                    button: into_entity_map(parts.button),
                    sensor: into_entity_map(parts.sensor),
                    binary_sensor: into_entity_map(parts.binary_sensor),
                    switch: into_entity_map(parts.switch),
                    light: into_entity_map(parts.light),
                    number: into_entity_map(parts.number),
                    text_sensor: into_entity_map(parts.text_sensor),
                    media_player: into_entity_map(parts.media_player),
                }
            }
        }
    };
}
