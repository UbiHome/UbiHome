use log::{debug, error};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::{future::Future, pin::Pin, str};
use tokio::sync::broadcast::{Receiver, Sender};
use ubihome_core::{
    ChangedMessage, Module, NoConfig, PublishedMessage, config_template,
    home_assistant::sensors::UbiButton,
    internal::sensors::{InternalButton, InternalComponent},
};

use system_shutdown::hibernate;
use system_shutdown::logout;
use system_shutdown::reboot;
use system_shutdown::shutdown;
use system_shutdown::sleep;

#[derive(Clone, Deserialize, Debug)]
pub struct PowerUtilsConfig {}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PowerAction {
    #[serde(alias = "reboot", alias = "restart")]
    Reboot,
    #[serde(alias = "shutdown")]
    Shutdown,
    #[serde(alias = "hibernate")]
    Hibernate,
    #[serde(alias = "logout")]
    Logout,
    #[serde(alias = "sleep")]
    Sleep,
}

#[derive(Clone, Deserialize, Debug)]
pub struct PowerUtilsButtonConfig {
    pub action: PowerAction,
}

config_template!(
    power_utils,
    PowerUtilsConfig,
    PowerUtilsButtonConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig
);

pub struct Default {
    config: PowerUtilsConfig,
    components: Vec<InternalComponent>,
    buttons: HashMap<String, PowerUtilsButtonConfig>,
}

impl Module for Default {
    fn new(config_string: &String) -> Result<Self, String> {
        let config = serde_saphyr::from_str::<CoreConfig>(config_string).unwrap();
        // info!("PowerUtils config: {:?}", config);
        let mut components: Vec<InternalComponent> = Vec::new();

        let mut buttons: HashMap<String, PowerUtilsButtonConfig> = HashMap::new();
        for (_, any_sensor) in config.button.clone().unwrap_or_default() {
            match any_sensor.extra {
                ButtonKind::power_utils(button) => {
                    let id = any_sensor.default.get_object_id();
                    let button_component;
                    match button.action {
                        PowerAction::Reboot => {
                            button_component = InternalButton {
                                ha: UbiButton {
                                    platform: "sensor".to_string(),
                                    icon: Some(
                                        any_sensor
                                            .default
                                            .icon
                                            .unwrap_or("mdi:restart".to_string()),
                                    ),
                                    name: any_sensor.default.name.clone(),
                                    id: id.clone(),
                                },
                            };
                        }
                        PowerAction::Shutdown => {
                            button_component = InternalButton {
                                ha: UbiButton {
                                    platform: "sensor".to_string(),
                                    icon: Some(
                                        any_sensor.default.icon.unwrap_or("mdi:power".to_string()),
                                    ),
                                    name: any_sensor.default.name.clone(),
                                    id: id.clone(),
                                },
                            };
                        }
                        PowerAction::Hibernate => {
                            button_component = InternalButton {
                                ha: UbiButton {
                                    platform: "sensor".to_string(),
                                    icon: Some(
                                        any_sensor
                                            .default
                                            .icon
                                            .unwrap_or("mdi:snowflake".to_string()),
                                    ),
                                    name: any_sensor.default.name.clone(),
                                    id: id.clone(),
                                },
                            };
                        }
                        PowerAction::Logout => {
                            button_component = InternalButton {
                                ha: UbiButton {
                                    platform: "sensor".to_string(),
                                    icon: Some(
                                        any_sensor.default.icon.unwrap_or("mdi:logout".to_string()),
                                    ),
                                    name: any_sensor.default.name.clone(),
                                    id: id.clone(),
                                },
                            };
                        }
                        PowerAction::Sleep => {
                            button_component = InternalButton {
                                ha: UbiButton {
                                    platform: "sensor".to_string(),
                                    icon: Some(
                                        any_sensor.default.icon.unwrap_or("mdi:sleep".to_string()),
                                    ),
                                    name: any_sensor.default.name.clone(),
                                    id: id.clone(),
                                },
                            };
                        }
                    }

                    components.push(InternalComponent::Button(button_component));
                    buttons.insert(id.clone(), button);
                }
                _ => {}
            }
        }
        Ok(Default {
            config: config.power_utils,
            components,
            buttons,
        })
    }

    fn components(&mut self) -> Vec<InternalComponent> {
        self.components.clone()
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
                                        debug!("Rebooting...");
                                        match reboot() {
                                            Ok(_) => debug!("Rebooting."),
                                            Err(error) => error!("Failed to reboot: {}", error),
                                        }
                                    }
                                    PowerAction::Shutdown => {
                                        debug!("Shutting down...");
                                        match shutdown() {
                                            Ok(_) => debug!("Shutting down."),
                                            Err(error) => error!("Failed to shut down: {}", error),
                                        }
                                    }
                                    PowerAction::Hibernate => {
                                        debug!("Hibernating...");
                                        match hibernate() {
                                            Ok(_) => debug!("Hibernating."),
                                            Err(error) => error!("Failed to hibernate: {}", error),
                                        }
                                    }
                                    PowerAction::Logout => {
                                        debug!("Logging out...");
                                        match logout() {
                                            Ok(_) => debug!("Logging out."),
                                            Err(error) => error!("Failed to log out: {}", error),
                                        }
                                    }
                                    PowerAction::Sleep => {
                                        debug!("Sleeping...");
                                        match sleep() {
                                            Ok(_) => debug!("Sleeping."),
                                            Err(error) => error!("Failed to sleep: {}", error),
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
