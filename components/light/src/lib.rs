use duration_str::deserialize_duration;
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Deserializer};
use shell_exec::{Execution, Shell, ShellError};
use std::any;
use std::collections::HashMap;
use std::sync::Arc;
use std::{future::Future, pin::Pin, str, time::Duration};
use tokio::{
    sync::broadcast::{Receiver, Sender},
    time,
};
use ubihome_core::home_assistant::sensors::UbiLight;
use ubihome_core::internal::sensors::InternalLight;
use ubihome_core::{
    config_template,
    ChangedMessage, Module, PublishedMessage,
};

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CustomShell {
    Zsh,
    Bash,
    Sh,
    Cmd,
    Powershell,
    Wsl,
}

#[derive(Clone, Deserialize, Debug)]
pub struct LightConfig {
    #[serde(rename = "type")]
    pub kind: Option<CustomShell>,

    #[serde(default = "default_timeout")]
    #[serde(deserialize_with = "deserialize_duration")]
    pub timeout: Duration,
}

fn default_timeout() -> Duration {
    Duration::from_secs(5)
}

#[derive(Clone, Deserialize, Debug)]
pub struct LightLightConfig {
    pub command_on: String,
    pub command_off: String,
    pub command_state: Option<String>,
    pub command_brightness: Option<String>,
    pub command_rgb: Option<String>,
    
    pub supports_brightness: Option<bool>,
    pub supports_rgb: Option<bool>,
    pub supports_white_value: Option<bool>,
    pub supports_color_temperature: Option<bool>,

    #[serde(default = "default_timeout_none")]
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub update_interval: Option<Duration>,
}

fn default_timeout_none() -> Option<Duration> {
    None
}

config_template!(
    shell_light,
    LightConfig,
    NoButtonExtension,
    NoBinarySensorExtension,
    NoSensorExtension,
    NoSwitchExtension,
    LightLightConfig
);

// Dummy types for unused entity types in the light component
#[derive(Clone, Deserialize, Debug)]
pub struct NoButtonExtension {}

#[derive(Clone, Deserialize, Debug)]
pub struct NoBinarySensorExtension {}

#[derive(Clone, Deserialize, Debug)]
pub struct NoSensorExtension {}

#[derive(Clone, Deserialize, Debug)]
pub struct NoSwitchExtension {}

pub struct Default {
    config: LightConfig,
    components: Vec<InternalLight>,
    lights: HashMap<String, LightLightConfig>,
}

impl Module for Default {
    fn new(config_string: &String) -> Result<Self, String> {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();
        debug!("Light config: {:?}", config);
        let mut components: Vec<InternalLight> = Vec::new();

        let mut lights: HashMap<String, LightLightConfig> = HashMap::new();
        for (_, any_light) in config.light.clone().unwrap_or_default() {
            match any_light.extra {
                LightKind::shell_light(light_config) => {
                    let id = any_light.default.get_object_id();
                    components.push(InternalLight {
                        ha: UbiLight {
                            platform: "light".to_string(),
                            icon: any_light.default.icon.clone(),
                            name: any_light.default.name.clone(),
                            id: id.clone(),
                            supports_brightness: light_config.supports_brightness.unwrap_or(false),
                            supports_rgb: light_config.supports_rgb.unwrap_or(false),
                            supports_white_value: light_config.supports_white_value.unwrap_or(false),
                            supports_color_temperature: light_config.supports_color_temperature.unwrap_or(false),
                        },
                    });
                    lights.insert(id.clone(), light_config);
                }
                _ => {}
            }
        }
        Ok(Default {
            config: config.shell_light,
            components,
            lights,
        })
    }

    fn components(&mut self) -> Vec<ubihome_core::internal::sensors::InternalComponent> {
        self.components.clone().into_iter().map(|light| {
            ubihome_core::internal::sensors::InternalComponent::Light(light)
        }).collect()
    }

    fn run(
        &self,
        sender: Sender<ChangedMessage>,
        mut receiver: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let config = self.config.clone();
        let lights = self.lights.clone();
        Box::pin(async move {
            let cloned_config = config.clone();
            let csender = sender.clone();

            let lights_clone = lights.clone();
            // Handle Light Commands
            tokio::spawn(async move {
                let cloned_sender = csender.clone();

                while let Ok(cmd) = receiver.recv().await {
                    match cmd {
                        PublishedMessage::LightStateCommand { key, state, brightness, red, green, blue } => {
                            debug!("LightStateCommand: {} state:{} brightness:{:?} rgb:{:?},{:?},{:?}", 
                                key, state, brightness, red, green, blue);
                            if let Some(light) = lights_clone.get(&key) {
                                let command: String;
                                if state {
                                    debug!("Turning on light: {}", key);
                                    command = light.command_on.clone();
                                } else {
                                    debug!("Turning off light: {}", key);
                                    command = light.command_off.clone();
                                }
                                debug!("Executing command: {}", command);

                                let output = execute_command(
                                    &cloned_config,
                                    &command,
                                    &cloned_config.timeout,
                                )
                                .await
                                .unwrap();
                                
                                if output.is_empty() {
                                    trace!("Command executed successfully with no output.");
                                } else {
                                    trace!("Command executed successfully with output: {}", output);
                                }

                                // Handle brightness command if provided and supported
                                if let (Some(brightness_val), Some(brightness_cmd)) = (brightness, &light.command_brightness) {
                                    if light.supports_brightness.unwrap_or(false) {
                                        let brightness_command = brightness_cmd.replace("{brightness}", &brightness_val.to_string());
                                        debug!("Executing brightness command: {}", brightness_command);
                                        let _ = execute_command(
                                            &cloned_config,
                                            &brightness_command,
                                            &cloned_config.timeout,
                                        ).await;
                                    }
                                }

                                // Handle RGB color command if provided and supported
                                if let (Some(r), Some(g), Some(b), Some(rgb_cmd)) = (red, green, blue, &light.command_rgb) {
                                    if light.supports_rgb.unwrap_or(false) {
                                        let rgb_command = rgb_cmd
                                            .replace("{red}", &r.to_string())
                                            .replace("{green}", &g.to_string())
                                            .replace("{blue}", &b.to_string());
                                        debug!("Executing RGB command: {}", rgb_command);
                                        let _ = execute_command(
                                            &cloned_config,
                                            &rgb_command,
                                            &cloned_config.timeout,
                                        ).await;
                                    }
                                }

                                // Check state after command if state command is available
                                if let Some(command_state) = &light.command_state {
                                    let output = execute_command(
                                        &cloned_config,
                                        &command_state,
                                        &cloned_config.timeout,
                                    )
                                    .await;
                                
                                    match output {
                                        Ok(output) => {
                                            debug!("Light {} state output: {}", key, &output);
                                            let value = if output.trim().to_lowercase() == "true" {
                                                true
                                            } else if output.trim().to_lowercase() == "false" {
                                                false
                                            } else {
                                                debug!("Invalid light state output: {}", output);
                                                continue;
                                            };

                                            _ = cloned_sender.send(
                                                ChangedMessage::LightStateChange {
                                                    key: key.clone(),
                                                    state: value,
                                                    brightness,
                                                    red,
                                                    green,
                                                    blue,
                                                },
                                            );
                                        }
                                        Err(e) => {
                                            debug!("Error executing state command: {}", e);
                                        }
                                    };
                                }
                            }
                        }
                        _ => {}
                    }
                }
            });

            // Monitor light states with update intervals
            for (key, light) in lights {
                let light = light.clone();
                debug!("Light State monitor for: {:?}", light);

                if let Some(command_state) = light.command_state {
                    let cloned_config = config.clone();
                    let cloned_sender = sender.clone();
                    tokio::spawn(async move {
                        let duration = light
                            .update_interval
                            .unwrap_or_else(|| Duration::from_secs(60));

                        let mut interval = time::interval(duration);
                        debug!("Light {} has update interval: {:?}", key, interval.period());
                        loop {
                            let output = execute_command(
                                &cloned_config,
                                &command_state,
                                &cloned_config.timeout,
                            )
                            .await;
                            match output {
                                Ok(output) => {
                                    debug!("Light {} state: {}", key, &output);
                                    let value = if output.trim().to_lowercase() == "true" {
                                        true
                                    } else if output.trim().to_lowercase() == "false" {
                                        false
                                    } else {
                                        debug!("Invalid light state output: {}", output);
                                        interval.tick().await;
                                        continue;
                                    };

                                    _ = cloned_sender.send(ChangedMessage::LightStateChange {
                                        key: key.clone(),
                                        state: value,
                                        brightness: None, // TODO: Parse from command output if needed
                                        red: None,
                                        green: None,
                                        blue: None,
                                    });
                                }
                                Err(e) => {
                                    debug!("Error executing command: {}", e);
                                }
                            };

                            interval.tick().await;
                        }
                    });
                } else {
                    warn!("Light {} has no command_state", key);
                }
            }
            Ok(())
        })
    }
}

async fn execute_command(
    light_config: &LightConfig,
    command: &str,
    timeout: &Duration,
) -> Result<String, ShellError> {
    let shell = match light_config.kind {
        Some(CustomShell::Zsh) => Shell::Zsh,
        Some(CustomShell::Bash) => Shell::Bash,
        Some(CustomShell::Sh) => Shell::Sh,
        Some(CustomShell::Cmd) => Shell::Cmd,
        Some(CustomShell::Powershell) => Shell::Powershell,
        Some(CustomShell::Wsl) => Shell::Wsl,
        None => Shell::default(),
    };

    let execution = Execution::builder()
        .shell(shell)
        .cmd(command.to_string())
        .timeout(*timeout)
        .build();

    trace!("Executing command: {}", command);
    let output = execution.execute(b"").await?;
    let output_string = str::from_utf8(&output).unwrap_or("");
    trace!("Command '{}' executed: {}", command, output_string);
    Ok(output_string.to_string())
}