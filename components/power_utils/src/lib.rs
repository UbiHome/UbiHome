use log::{debug, error, info};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::{future::Future, pin::Pin, str};
use tokio::sync::broadcast::{Receiver, Sender};
use ubihome_core::internal::sensors::UbiComponent;
use ubihome_core::state::StateStore;
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

/// Restarts UbiHome by re-executing the current binary. On Unix the image is
/// replaced in place; on a Windows terminal a new process is spawned; as a
/// Windows service it exits so the service controller restarts it. On failure
/// it logs the error and keeps running.
fn restart_process() {
    info!("Restart requested - restarting UbiHome process");

    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(err) => {
            error!("Could not restart UbiHome: failed to resolve current executable: {err}");
            return;
        }
    };
    let args: Vec<String> = std::env::args().skip(1).collect();

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // `exec` replaces the image in place; it only returns on failure.
        let err = std::process::Command::new(&exe).args(&args).exec();
        error!("Could not restart UbiHome: {err}");
    }

    #[cfg(not(unix))]
    {
        // Exit so the Windows service controller restarts us (the only way).
        if args
            .iter()
            .any(|arg| arg.as_str() == "--as-windows-service")
        {
            std::process::exit(0);
        }

        match std::process::Command::new(&exe).args(&args).spawn() {
            Ok(_) => std::process::exit(0),
            Err(err) => error!("Could not restart UbiHome: failed to spawn new process: {err}"),
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub enum PowerAction {
    #[serde(alias = "reboot", alias = "restart")]
    Reboot,
    /// Restarts the UbiHome process itself (not the machine).
    #[serde(rename = "restart_service")]
    RestartService,
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

config_template!(
    power_utils,
    PowerUtilsConfig,
    PowerUtilsButtonConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig
);

pub struct UbiHomePlatform {
    components: Vec<UbiComponent>,
    buttons: HashMap<String, PowerAction>,
}

impl Module for UbiHomePlatform {
    fn new(config_string: &str, config_path: &str) -> Result<Self, String> {
        let config =
            ubihome_core::validation::validate_config::<CoreConfig>(config_string, config_path)?;

        info!("PowerUtils config: {:?}", config);
        let mut components: Vec<UbiComponent> = Vec::new();

        let mut buttons: HashMap<String, PowerAction> = HashMap::new();
        for (_, button) in config.button.clone().unwrap_or_default() {
            let id = button.get_object_id();
            let name = button.name.clone().unwrap_or_default();
            let internal = button.internal;
            let button_component = match button.action {
                PowerAction::Reboot => UbiButton {
                    platform: "sensor".to_string(),
                    icon: Some(button.icon.unwrap_or("mdi:restart".to_string())),
                    name,
                    internal,
                    id: id.clone(),
                },
                PowerAction::RestartService => UbiButton {
                    platform: "sensor".to_string(),
                    icon: Some(button.icon.unwrap_or("mdi:reload".to_string())),
                    name,
                    internal,
                    id: id.clone(),
                },
                PowerAction::Shutdown => UbiButton {
                    platform: "sensor".to_string(),
                    icon: Some(button.icon.unwrap_or("mdi:power".to_string())),
                    name,
                    internal,
                    id: id.clone(),
                },
                PowerAction::Hibernate => UbiButton {
                    platform: "sensor".to_string(),
                    icon: Some(button.icon.unwrap_or("mdi:snowflake".to_string())),
                    name,
                    internal,
                    id: id.clone(),
                },
                PowerAction::Logout => UbiButton {
                    platform: "sensor".to_string(),
                    icon: Some(button.icon.unwrap_or("mdi:logout".to_string())),
                    name,
                    internal,
                    id: id.clone(),
                },
                PowerAction::Sleep => UbiButton {
                    platform: "sensor".to_string(),
                    icon: Some(button.icon.unwrap_or("mdi:sleep".to_string())),
                    name,
                    internal,
                    id: id.clone(),
                },
            };

            components.push(UbiComponent::Button(button_component));
            buttons.insert(id.clone(), button.action);
        }
        Ok(UbiHomePlatform {
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
        _state: StateStore,
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
                                    PowerAction::RestartService => {
                                        restart_process();
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
