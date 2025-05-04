use log::{debug, info};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::{future::Future, pin::Pin, str};
use tokio::sync::broadcast::{Receiver, Sender};
use ubihome_core::{
    config_template,
    home_assistant::sensors::HABinarySensor,
    internal::sensors::{InternalBinarySensor, InternalComponent},
    ChangedMessage, Module, NoConfig, PublishedMessage,
};

#[derive(Clone, Deserialize, Debug)]
pub struct EvdevConfig {
    // pub device: GpioDevice,
}

// TODO: Add events?
config_template!(evdev, EvdevConfig, NoConfig, NoConfig, NoConfig);

#[derive(Clone, Debug)]
pub struct Default {
    config: CoreConfig,
}

impl Default {
    pub fn new(config_string: &String) -> Self {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();
        info!("Evdev config: {:?}", config);
        Default { config: config }
    }
}

impl Module for Default {
    fn validate(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn init(&mut self) -> Result<Vec<InternalComponent>, String> {
        let mut components: Vec<InternalComponent> = Vec::new();

        for (_, any_sensor) in self.config.binary_sensor.clone().unwrap_or_default() {
            match any_sensor.extra {
                BinarySensorKind::evdev(_) => {
                    let object_id = format!(
                        "{}_{}",
                        self.config.ubihome.name,
                        any_sensor.default.name.clone()
                    );
                    let id = any_sensor.default.id.unwrap_or(object_id.clone());
                    components.push(InternalComponent::BinarySensor(InternalBinarySensor {
                        ha: HABinarySensor {
                            platform: "sensor".to_string(),
                            icon: any_sensor.default.icon.clone(),
                            unique_id: Some(id),
                            device_class: any_sensor.default.device_class.clone(),
                            name: any_sensor.default.name.clone(),
                            object_id: object_id.clone(),
                        },
                        filters: None,
                    }));
                }
                _ => {}
            }
        }
        Ok(components)
    }

    fn run(
        &self,
        sender: Sender<ChangedMessage>,
        _: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let config = self.config.clone();
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

                if let Some(binary_sensors) = config.binary_sensor.clone() {
                    for (key, binary_sensor) in binary_sensors {
                        let cloned_sender = sender.clone();
                        match binary_sensor.extra {
                            BinarySensorKind::evdev(gpio_config) => {
                                debug!("BinarySensor {} is of type evdev", key);

                                // tokio::spawn(async move {
                                //     let duration = gpio_config
                                //         .update_interval
                                //         .unwrap_or(Duration::from_secs(30));
                                //     let mut interval = time::interval(duration);

                                //     loop {
                                //         interval.tick().await;

                                //     }
                                // });
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(())
        })
    }
}
