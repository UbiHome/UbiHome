use log::debug;
use oshome_core::{binary_sensor::BinarySensorKind, sensor::SensorKind, CoreConfig, Message};
use serde::Deserialize;
use std::{str, time::Duration};
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

pub async fn start(
    sender: Sender<Option<Message>>,
    mut receiver: Receiver<Option<Message>>,
    config: &CoreConfig,
    shell_config: &GpioConfig,
) {
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
                    BinarySensorKind::Gpio(gpio) => {
                        debug!("BinarySensor {} is of type Gpio", key);

                        let gpio = Gpio::new().unwrap();
                        let mut pin = gpio
                            .get(23)
                            .expect("GPIO pin not found?")
                            .into_input_pullup();

                        pin.set_async_interrupt(
                            Trigger::RisingEdge,             // TODO: configurable!
                            Some(Duration::from_millis(50)), // TOOD: configurable!
                            move |event| {
                                debug!("BinarySensor {} triggered.", key);
                                println!("Binary Sensor '{}' triggered.", key);

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
}
