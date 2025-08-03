use log::{debug, warn};
use serde::{Deserialize, Deserializer};
use std::{collections::HashMap, future};
use std::{future::Future, pin::Pin, str};
use tokio::sync::broadcast::{Receiver, Sender};
use ubihome_core::{
    config_template,
    home_assistant::sensors::UbiBinarySensor,
    internal::sensors::{InternalBinarySensor, InternalComponent},
    ChangedMessage, Module, NoConfig, PublishedMessage,
};

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GpioDevice {
    RaspberryPi,
}

#[derive(Clone, Deserialize, Debug)]
pub struct GpioConfig {
    pub device: GpioDevice,
}

#[derive(Clone, Deserialize, Debug)]
pub struct GpioBinarySensorConfig {
    pub pin: u8, // TODO: Use GPIO types or library
    pub pull_up: Option<bool>,
}

config_template!(gpio, GpioConfig, NoConfig, GpioBinarySensorConfig, NoConfig, NoConfig, NoConfig);

#[derive(Clone, Debug)]
pub struct Default {
    components: Vec<InternalComponent>,
    binary_sensors: HashMap<String, GpioBinarySensorConfig>,
}


impl Module for Default {
    fn new(config_string: &String) -> Result<Self, String> {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();
        // info!("GPIO config: {:?}", config);
        let mut components: Vec<InternalComponent> = Vec::new();
        let mut binary_sensors: HashMap<String, GpioBinarySensorConfig> = HashMap::new();

        for (_, any_sensor) in config.binary_sensor.clone().unwrap_or_default() {
            match any_sensor.extra {
                BinarySensorKind::gpio(binary_sensor) => {
                    let id = any_sensor.default.get_object_id();
                    components.push(InternalComponent::BinarySensor(InternalBinarySensor {
                        ha: UbiBinarySensor {
                            platform: "sensor".to_string(),
                            icon: any_sensor.default.icon.clone(),
                            device_class: any_sensor.default.device_class.clone(),
                            name: any_sensor.default.name.clone(),
                            id: id.clone(),
                        },
                        base: any_sensor.default.clone(),
                    }));
                    binary_sensors.insert(
                        id,
                        GpioBinarySensorConfig {
                            pin: binary_sensor.pin,
                            pull_up: binary_sensor.pull_up,
                        },
                    );
                }
                _ => {}
            }
        }

        Ok(Default {
            components,
            binary_sensors,
        })
    }

    fn components(&mut self) -> Vec<InternalComponent> {
        self.components.clone()
    }

    fn run(
        &self,
        sender: Sender<ChangedMessage>,
        _: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let binary_sensors = self.binary_sensors.clone();
        Box::pin(async move {
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            {
                warn!("GPIO is not supported on this platform.");
                return Ok(());
            }
            #[cfg(target_os = "linux")]
            {
                use rppal::gpio::{Gpio, Trigger};

                let gpio = Gpio::new();
                match gpio {
                    Err(e) => {
                        warn!("Error initializing GPIO: {}", e);
                        return Ok(());
                    }
                    Ok(gpio) => {
                        // Handle Button Presses
                        // let cloned_config = self.config.clone();
                        // tokio::spawn(async move {
                        //     while let Ok(Some(cmd)) = receiver.recv().await {
                        //         use Message::*;

                        //         match cmd {
                        //             ButtonPress { key } => {
                        //                 if let Some(button) = &cloned_config.button.as_ref().and_then(|b| b.get(&key))
                        //                     {
                        //                         debug!("Button pressed: {}", key);
                        //                         debug!("Executing command: {}", button.command);
                        //                         println!("Button '{}' pressed.", key);

                        //                         let output = execute_command(&cloned_shell_config, &button.command, &cloned_shell_config.timeout).await.unwrap();
                        //                         // If output is empty report status code
                        //                         if output.is_empty() {
                        //                             println!("Command executed successfully with no output.");
                        //                         } else {
                        //                             println!("Command executed successfully with output: {}", output);
                        //                         }
                        //                     } else {
                        //                         debug!("Button pressed: {}", key);
                        //                     }
                        //             }
                        //             _ => {
                        //                 debug!("Ignored message type: {:?}", cmd);
                        //             }
                        //         }
                        //     }
                        // });

                        for (key, binary_sensor) in binary_sensors {
                            let gpio_pin =
                                gpio.get(binary_sensor.pin).expect("GPIO pin not found?");
                            let mut pin: rppal::gpio::InputPin;
                            let pull_up = binary_sensor.pull_up.unwrap_or(true);
                            if pull_up {
                                debug!("pullup");
                                pin = gpio_pin.into_input_pullup();
                            } else {
                                debug!("pulldown");
                                pin = gpio_pin.into_input_pulldown();
                            }

                            // Errors?
                            // cat /sys/kernel/debug/gpio
                            let cloned_key = key.clone();
                            let cloned_sender = sender.clone();
                            pin.set_async_interrupt(Trigger::Both, None, move |event| {
                                debug!("Event: {:?}", event);
                                debug!("BinarySensor {} triggered.", cloned_key);

                                match event.trigger {
                                    Trigger::RisingEdge => {
                                        _ = cloned_sender.send(ChangedMessage::BinarySensorValueChange {
                                            key: cloned_key.clone(),
                                            value: true,
                                        });
                                    }
                                    Trigger::FallingEdge => {
                                        _ = cloned_sender.send(ChangedMessage::BinarySensorValueChange {
                                            key: cloned_key.clone(),
                                            value: false,
                                        });
                                    }
                                    _ => {
                                        debug!("Unknown trigger detected {:?}", event.trigger);
                                    }
                                }
                            })
                            .expect("failed to set async interrupt");
                            debug!("Waiting for interrupts.");

                            // Wait indefinitely for the interrupts
                            let future = future::pending();
                            let () = future.await;
                            debug!("Interrupts stopped.");
                        }
                    }
                }
            }
            debug!("GPIO module stopped");
            Ok(())
        })
    }
}
