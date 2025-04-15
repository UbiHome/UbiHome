use log::{debug, warn};
use oshome_core::{config_template, home_assistant::sensors::Component, Message, Module};
use std::{env, future::Future, pin::Pin, str, time::Duration};
use tokio::{
    sync::broadcast::{Receiver, Sender},
    time,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;



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
pub struct NoConfig {
    // pub bla: String
}

#[derive(Clone, Deserialize, Debug)]
pub struct GpioBinarySensorConfig {
    pub pin: u8, // TODO: Use GPIO types or library
    pub pull_up: Option<bool>,
}

config_template!(gpio, GpioConfig, NoConfig, GpioBinarySensorConfig, NoConfig);


#[derive(Clone, Debug)]
pub struct Default {
    config: CoreConfig
} 


impl Default {
    pub fn new(config_string: &String) -> Self {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();

        Default {
            config: config,
        }
    }
}

impl Module for Default {
    fn validate(&mut self) -> Result<(), String> {
        Ok(())
    }


    fn init(&mut self) -> Result<Vec<Component>, String> {
        let mut components: Vec<Component> = Vec::new();

        Ok(components)
    }

    fn run(&self,
    sender: Sender<Option<Message>>,
    mut receiver: Receiver<Option<Message>>,
) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>{
        let config = self.config.clone();
        Box::pin(async move {

            #[cfg(any(target_os = "macos", target_os = "windows"))]
            {
                panic!("GPIO is not supported.");
            }
            #[cfg(target_os = "linux")]
            {
                use rppal::gpio::{Gpio, Trigger};

                let result = Gpio::new();
                match result {
                    Err(e) => {
                        warn!("Error initializing GPIO: {}", e);
                        return Ok(())
                    }
                    _ => {}
                }
        
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
        
                if let Some(binary_sensors) = config.binary_sensor.clone() {
                    for (key, binary_sensor) in binary_sensors {
                        let cloned_sender = sender.clone();
                        match binary_sensor.extra {
                            BinarySensorKind::gpio(gpio_config) => {
                                debug!("BinarySensor {} is of type Gpio", key);
        
                                let gpio = Gpio::new().unwrap().get(gpio_config.pin).expect("GPIO pin not found?");
                                let mut pin: rppal::gpio::InputPin;
                                if let Some(pullup) = gpio_config.pull_up {
                                    if pullup {
                                        pin = gpio.into_input_pullup();
                                    } else {
                                        pin = gpio.into_input_pulldown();
                                    }
                                } else {
                                    pin = gpio.into_input_pullup();
                                }
        
                                pin.set_async_interrupt(
                                    Trigger::Both,
                                    None,
                                    move |event| {
                                        debug!("BinarySensor {} triggered.", key);
                                        println!("Binary Sensor '{}' triggered.", key);
                                        match event.trigger {
                                            Trigger::RisingEdge => {
                                                debug!("RisingEdge triggered.");
                                            }
                                            Trigger::FallingEdge => {
                                                debug!("FallingEdge triggered.");
                                            }
                                            Trigger::Both => {
                                                debug!("Both triggered.");
                                            }
                                            _ => {
                                                debug!("Unknown trigger.");
                                            }
                                        }
        
        
                                        _ = cloned_sender.send(Some(Message::BinarySensorValueChange {
                                            key: key.clone(),
                                            value: true,
                                        }));
                                        let cloned_sender2 = cloned_sender.clone();
        
                                        let cloned_key = key.clone();
                                        _ = tokio::spawn(async move {
                                            tokio::time::sleep(Duration::from_secs(5)).await;
                                            _ = &cloned_sender2.send(Some(
                                                Message::BinarySensorValueChange {
                                                    key: cloned_key,
                                                    value: false,
                                                },
                                            ));
                                        });
                                    },
                                )
                                .unwrap();
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

// pub async fn start(

// ) {
    
// }
