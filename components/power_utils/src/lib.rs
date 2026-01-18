use log::{debug, error, info};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::{future::Future, pin::Pin, str};
use tokio::sync::broadcast::{Receiver, Sender};
use ubihome_core::constants::is_id_string_option;
use ubihome_core::constants::is_readable_string;
use ubihome_core::internal::sensors::UbiComponent;
use ubihome_core::template_binary_sensor;
use ubihome_core::template_button;
use ubihome_core::with_base_entity_properties;
use ubihome_core::{
    ChangedMessage, Module, NoConfig, PublishedMessage, config_template,
    internal::sensors::UbiButton,
};

use system_shutdown::hibernate;
use system_shutdown::logout;
use system_shutdown::reboot;
use system_shutdown::shutdown;
use system_shutdown::sleep;

#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct PowerUtilsConfig {}

#[derive(Debug, Copy, Clone, Deserialize, Validate)]
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

template_button! {
#[derive(Clone, Deserialize, Debug, Validate)]
pub struct PowerUtilsButtonConfig {
    #[garde(dive)]
    pub action: PowerAction,
}
}

// #[derive(Clone, Deserialize, Debug, Validate)]
// pub enum ButtonKind {
//     power_utils(#[garde(dive)] PowerUtilsButtonConfig),
//     nothing(#[garde(skip)] NoConfig),
// }

// #[derive(Clone, Deserialize, Debug, Validate)]
// pub struct TestButton {
//     #[serde(flatten)]
//     #[garde(dive)]
//     kind: ButtonKind,
// }

// impl TestButton {
//     pub fn is_configured(&self) -> bool {
//         true
//     }
//     pub fn get_object_id(&self) -> String {
//         unimplemented!();
//     }
// }

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
    components: Vec<UbiComponent>,
    buttons: HashMap<String, PowerAction>,
}

impl Module for Default {
    fn new(config_string: &String) -> Result<Self, String> {
        let config =
            serde_saphyr::from_str::<CoreConfig>(config_string).map_err(|e| e.to_string())?;
        info!("PowerUtils config: {:?}", config);
        let mut components: Vec<UbiComponent> = Vec::new();

        let mut buttons: HashMap<String, PowerAction> = HashMap::new();
        for (_, button) in config.button.clone().unwrap_or_default() {
            match button.kind {
                PowerUtilsButtonConfigKind::nothing(_) => {
                    continue;
                }
                PowerUtilsButtonConfigKind::parsed(button) => {
                    let id = button.get_object_id();
                    let button_component;
                    match button.action {
                        PowerAction::Reboot => {
                            button_component = UbiButton {
                                platform: "sensor".to_string(),
                                icon: Some(button.icon.unwrap_or("mdi:restart".to_string())),
                                name: button.name.clone(),
                                id: id.clone(),
                            };
                        }
                        PowerAction::Shutdown => {
                            button_component = UbiButton {
                                platform: "sensor".to_string(),
                                icon: Some(button.icon.unwrap_or("mdi:power".to_string())),
                                name: button.name.clone(),
                                id: id.clone(),
                            };
                        }
                        PowerAction::Hibernate => {
                            button_component = UbiButton {
                                platform: "sensor".to_string(),
                                icon: Some(button.icon.unwrap_or("mdi:snowflake".to_string())),
                                name: button.name.clone(),
                                id: id.clone(),
                            };
                        }
                        PowerAction::Logout => {
                            button_component = UbiButton {
                                platform: "sensor".to_string(),
                                icon: Some(button.icon.unwrap_or("mdi:logout".to_string())),
                                name: button.name.clone(),
                                id: id.clone(),
                            };
                        }
                        PowerAction::Sleep => {
                            button_component = UbiButton {
                                platform: "sensor".to_string(),
                                icon: Some(button.icon.unwrap_or("mdi:sleep".to_string())),
                                name: button.name.clone(),
                                id: id.clone(),
                            };
                        }
                    }

                    components.push(UbiComponent::Button(button_component));
                    buttons.insert(id.clone(), button.action);
                    // continue;
                }
            }
        }
        Ok(Default {
            config: config.power_utils,
            components,
            buttons,
        })
    }

    fn components(&mut self) -> Vec<UbiComponent> {
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
                            if let Some(action) = buttons.get(&key) {
                                debug!("Button pressed: {}", key);
                                debug!("Executing command: {:?}", action);

                                match action {
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
