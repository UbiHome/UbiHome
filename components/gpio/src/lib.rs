use log::debug;
use oshome_core::{binary_sensor::BinarySensorKind, home_assistant::sensors::Component, sensor::SensorKind, CoreConfig, Message, Module};
use saphyr::Yaml;
use serde::Deserialize;
use std::{future::Future, pin::Pin, str, time::Duration};
use tokio::{
    sync::broadcast::{Receiver, Sender},
    time,
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

#[derive(Clone, Debug)]
pub struct Default {

} 

impl Module for Default {

    fn validate(&mut self, config: &Yaml) -> Result<(), String> {
        Ok(())
    }


    fn init(&mut self, config: &CoreConfig) -> Result<Vec<Component>, String> {
        let mut components: Vec<Component> = Vec::new();

        Ok(components)
    }

    fn run(&self,
    sender: Sender<Option<Message>>,
    mut receiver: Receiver<Option<Message>>,
) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>{
        Box::pin(async { 
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            {
                panic!("GPIO is not supported on macOS.");
            }
            #[cfg(target_os = "linux")]
            {
                use rppal::gpio::{Gpio, Trigger};
        
                // Handle Button Presses
                let cloned_config = config.clone();
                let cloned_shell_config = shell_config.clone();
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
                        match binary_sensor.kind {
                            BinarySensorKind::Gpio(gpio_config) => {
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
