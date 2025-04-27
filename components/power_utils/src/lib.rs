use log::{debug, error};
use oshome_core::{
    ChangedMessage, Module, NoConfig, PublishedMessage, config_template,
    home_assistant::sensors::{Component, HAButton},
};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::{future::Future, pin::Pin, str};
use tokio::sync::broadcast::{Receiver, Sender};

use system_shutdown::shutdown;
use system_shutdown::reboot;


#[derive(Clone, Deserialize, Debug)]
pub struct PowerUtilsConfig {}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PowerAction {
    #[serde(alias = "reboot")]
    Reboot,
    #[serde(alias = "shutdown")]
    Shutdown,
}


#[derive(Clone, Deserialize, Debug)]
pub struct PowerUtilsButtonConfig {
    pub action: PowerAction
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
        _: Sender<ChangedMessage>,
        mut receiver: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let buttons = self.buttons.clone();
        Box::pin(async move {
            // Handle Button Presses
            tokio::spawn(async move {
                while let Ok(cmd) = receiver.recv().await {
                    match cmd {
                        PublishedMessage::ButtonPressed { key } => {
                            debug!("Button pressed1: {}", key);
                            if let Some(power_utils_button) = buttons.get(&key) {
                                debug!("Button pressed: {}", key);
                                debug!("Executing command: {:?}", power_utils_button.action);

                                match power_utils_button.action {
                                    PowerAction::Reboot => {
                                        debug!("Rebooting");
                                        match reboot() {
                                            Ok(_) => debug!("Rebooting, bye!"),
                                            Err(error) => error!("Failed to reboot: {}", error),
                                        }
                                    }
                                    PowerAction::Shutdown => {
                                        debug!("Shutting down");
                                        match shutdown() {
                                            Ok(_) => debug!("Shutting down, bye!"),
                                            Err(error) => error!("Failed to shut down: {}", error),
                                        }
                                    }
                                }
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
