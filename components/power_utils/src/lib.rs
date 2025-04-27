use log::debug;
use oshome_core::{
    ChangedMessage, Module, NoConfig, PublishedMessage, config_template,
    home_assistant::sensors::{Component, HAButton},
};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::{future::Future, pin::Pin, str};
use tokio::sync::broadcast::{Receiver, Sender};

#[derive(Clone, Deserialize, Debug)]
pub struct PowerUtilsConfig {}

#[derive(Clone, Deserialize, Debug)]
pub struct PowerUtilsButtonConfig {
    pub command: String,
}

config_template!(
    power_utils,
    PowerUtilsConfig,
    PowerUtilsButtonConfig,
    NoConfig,
    NoConfig
);

pub struct Default {
    config: PowerUtilsConfig,
    components: Vec<Component>,
    buttons: HashMap<String, PowerUtilsButtonConfig>,
}

impl Default {
    pub fn new(config_string: &String) -> Self {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();
        // info!("PowerUtils config: {:?}", config);
        let mut components: Vec<Component> = Vec::new();

        let mut buttons: HashMap<String, PowerUtilsButtonConfig> = HashMap::new();
        for (_, any_sensor) in config.button.clone().unwrap_or_default() {
            match any_sensor.extra {
                ButtonKind::power_utils(button) => {
                    let id = any_sensor.default.get_object_id(&config.oshome.name);
                    components.push(Component::Button(HAButton {
                        platform: "sensor".to_string(),
                        icon: any_sensor.default.icon.clone(),
                        unique_id: Some(id.clone()),
                        name: any_sensor.default.name.clone(),
                        object_id: id.clone(),
                    }));
                    buttons.insert(id.clone(), button);
                }
                _ => {}
            }
        }
        Default {
            config: config.power_utils,
            components,
            buttons,
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
        mut receiver: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let config = self.config.clone();
        let buttons = self.buttons.clone();
        Box::pin(async move {
            let cloned_config = config.clone();
            // Handle Button Presses
            tokio::spawn(async move {
                while let Ok(cmd) = receiver.recv().await {
                    match cmd {
                        PublishedMessage::ButtonPressed { key } => {
                            debug!("Button pressed1: {}", key);
                            if let Some(PowerUtils_button) = buttons.get(&key) {
                                // ButtonKind::PowerUtils(PowerUtils_button) => {
                                debug!("Button pressed: {}", key);
                                debug!("Executing command: {}", PowerUtils_button.command);
                                println!("Button '{}' pressed.", key);

                                // let output = execute_command(
                                //     &cloned_config,
                                //     &PowerUtils_button.command,
                                //     &cloned_config.timeout,
                                // )
                                // .await
                                // .unwrap();
                                // If output is empty report status code
                                // if output.is_empty() {
                                //     println!("Command executed successfully with no output.");
                                // } else {
                                //     println!(
                                //         "Command executed successfully with output: {}",
                                //         output
                                //     );
                                // }
                            } else {
                                debug!("Button pressed2: {}", key);
                            }
                        }
                        _ => {
                            debug!("Ignored message type: {:?}", cmd);
                        }
                    }
                }
            });

            Ok(())
        })
    }
}
