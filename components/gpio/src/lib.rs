use log::{debug, warn};
use oshome_core::{
    config_template,
    home_assistant::sensors::{Component, HABinarySensor},
    ChangedMessage, Module, NoConfig, PublishedMessage,
};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::{future::Future, pin::Pin, str, time::Duration};
use tokio::sync::broadcast::{Receiver, Sender};

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
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub update_interval: Option<Duration>,
}

config_template!(gpio, GpioConfig, NoConfig, GpioBinarySensorConfig, NoConfig);

#[derive(Clone, Debug)]
pub struct InternalBinarySensor {
    pub key: String,
    pub pin: u8,
    pub pull_up: Option<bool>,
}

#[derive(Clone, Debug)]
pub struct Default {
    components: Vec<Component>,
    binary_sensors: Vec<InternalBinarySensor>,
}

impl Default {
    pub fn new(config_string: &String) -> Self {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();
        // info!("GPIO config: {:?}", config);
        let mut components: Vec<Component> = Vec::new();
        let mut binary_sensors: Vec<InternalBinarySensor> = Vec::new();

        for (_, any_sensor) in config.binary_sensor.clone().unwrap_or_default() {
            match any_sensor.extra {
                BinarySensorKind::gpio(binary_sensor) => {
                    let object_id = any_sensor.default.get_object_id(&config.oshome.name);
                    let id = any_sensor.default.id.unwrap_or(object_id.clone());
                    components.push(Component::BinarySensor(HABinarySensor {
                        platform: "sensor".to_string(),
                        icon: any_sensor.default.icon.clone(),
                        unique_id: Some(id.clone()),
                        device_class: any_sensor.default.device_class.clone(),
                        name: any_sensor.default.name.clone(),
                        object_id: object_id.clone(),
                    }));
                    binary_sensors.push(InternalBinarySensor {
                        key: id.clone(),
                        pin: binary_sensor.pin,
                        pull_up: binary_sensor.pull_up,
                    });
                }
                _ => {}
            }
        }

        Default {
            config,
            components,
            binary_sensors,
        }
    }
}

impl Module for Default {
    fn validate(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn init(&mut self) -> Result<Vec<Component>, String> {
        Ok(self.components.clone())
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
                use rppal::gpio::Event;
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

                        for binary_sensor in binary_sensors {
                            let cloned_sender = sender.clone();
                            // tokio::spawn(async move {
                            //     let duration = gpio_config
                            //         .update_interval
                            //         .unwrap_or(Duration::from_secs(30));
                            //     let mut interval = time::interval(duration);

                            //     loop {
                            //         interval.tick().await;

                            //     }
                            // });
                            let gpio_pin = gpio.get(binary_sensor.pin).expect("GPIO pin not found?");
                            let mut pin: rppal::gpio::InputPin;
                            if let Some(pullup) = binary_sensor.pull_up {
                                if pullup {
                                    debug!("pullup");
                                    pin = gpio_pin.into_input_pullup();
                                } else {
                                    debug!("pulldown");
                                    pin = gpio_pin.into_input_pulldown();
                                }
                            } else {
                                debug!("pullup");
                                pin = gpio_pin.into_input_pullup();
                            }

                            // Errors?
                            // cat /sys/kernel/debug/gpio

                            fn input_callback(event: Event) {
                                println!("Event: {:?}", event);
                            }

                            pin.set_async_interrupt(
                                Trigger::Both,
                                Some(Duration::from_millis(50)),
                                move |event| {
                                    // Note: you could add more parameters here!
                                    input_callback(event);
                                },
                            )
                            .expect("failed to set async interrupt");
                            pin.set_interrupt(Trigger::Both, None).unwrap();
                            pin.poll_interrupt(true, None).unwrap();
                            debug!("BinarySensor {} triggered.", binary_sensor.key);
                            println!("Binary Sensor '{}' triggered.", binary_sensor.key);

                            _ = cloned_sender.send(ChangedMessage::BinarySensorValueChange {
                                key: binary_sensor.key.clone(),
                                value: true,
                            });
                            let cloned_sender2 = cloned_sender.clone();

                            let cloned_key = binary_sensor.key.clone();
                            _ = tokio::spawn(async move {
                                tokio::time::sleep(Duration::from_secs(5)).await;
                                _ = &cloned_sender2.send(ChangedMessage::BinarySensorValueChange {
                                    key: cloned_key,
                                    value: false,
                                });
                            });
                        }
                    }
                }
            }
            Ok(())
        })
    }
}
