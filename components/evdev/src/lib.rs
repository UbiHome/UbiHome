use log::{debug, info};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::{future::Future, pin::Pin, str};
use tokio::sync::broadcast::{Receiver, Sender};
use ubihome_core::internal::sensors::UbiComponent;
use ubihome_core::{
    config_template, state::StateStore, ChangedMessage, Module, NoConfig, PublishedMessage,
};

#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct EvdevConfig {
    // pub device: GpioDevice,
}

// TODO: Add events?
config_template!(
    evdev,
    EvdevConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig
);

#[derive(Clone, Debug)]
pub struct UbiHomePlatform {
    config: CoreConfig,
}

impl Module for UbiHomePlatform {
    fn new(config_string: &str, config_path: &str) -> Result<Self, String> {
        let config =
            ubihome_core::validation::validate_config::<CoreConfig>(config_string, config_path)?;

        info!("Evdev config: {:?}", config);
        Ok(UbiHomePlatform { config: config })
    }

    fn components(&mut self) -> Vec<UbiComponent> {
        // TODO: Add events? binary_sensor is currently wired to NoConfig
        // (see config_template! below), so there is no evdev-specific config
        // to build entities from yet.
        Vec::new()
    }

    fn run(
        &self,
        _sender: Sender<ChangedMessage>,
        _: Receiver<PublishedMessage>,
        _state: StateStore,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        Box::pin(async move {
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            {
                panic!("EVDEV is not supported.");
            }
            #[cfg(target_os = "linux")]
            {
                // use evdev::{Device, KeyCode};
                // let device = Device::open("/dev/input/event0")?;
                // // check if the device has an ENTER key
                // if device.supported_keys().map_or(false, |keys| keys.contains(KeyCode::KEY_ENTER)) {
                //     println!("are you prepared to ENTER the world of evdev?");
                // } else {
                //     println!(":(");
                // }

                // TODO: Add events? binary_sensor is currently wired to
                // NoConfig (see config_template! below), so there is no
                // evdev-specific config to read here yet.
            }
            Ok(())
        })
    }
}
