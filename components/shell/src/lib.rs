use duration_str::deserialize_duration;
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Deserializer};
use shell_exec::{Execution, Shell, ShellError};
use std::collections::HashMap;
use std::{future::Future, pin::Pin, str, time::Duration};
use tokio::{
    sync::broadcast::{Receiver, Sender},
    time,
};
use ubihome_core::home_assistant::sensors::{UbiLight, UbiSwitch};
use ubihome_core::internal::sensors::{InternalLight, InternalSwitch};
use ubihome_core::{
    config_template,
    home_assistant::sensors::{UbiBinarySensor, UbiButton, UbiSensor},
    internal::sensors::{InternalBinarySensor, InternalButton, InternalComponent, InternalSensor},
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
pub struct ShellConfig {
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
pub struct ShellBinarySensorConfig {
    #[serde(default = "default_timeout_none")]
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub update_interval: Option<Duration>,
    pub command: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct ShellSensorConfig {
    pub command: String,

    #[serde(default = "default_timeout_none")]
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub update_interval: Option<Duration>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct ShellButtonConfig {
    pub command: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct ShellSwitchConfig {
    pub command_on: String,
    pub command_off: String,
    pub command_state: Option<String>,

    #[serde(default = "default_timeout_none")]
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub update_interval: Option<Duration>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct ShellLightConfig {
    pub command_on: String,
    pub command_off: String,
    pub command_state: Option<String>,
    // pub command_brightness: Option<String>,
    // pub command_rgb: Option<String>,

    // pub supports_brightness: Option<bool>,
    // pub supports_rgb: Option<bool>,
    // pub supports_white_value: Option<bool>,
    // pub supports_color_temperature: Option<bool>,
    #[serde(default = "default_timeout_none")]
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub update_interval: Option<Duration>,
}

fn default_timeout_none() -> Option<Duration> {
    None
}

config_template!(
    shell,
    ShellConfig,
    ShellButtonConfig,
    ShellBinarySensorConfig,
    ShellSensorConfig,
    ShellSwitchConfig,
    ShellLightConfig
);

pub struct Default {
    config: ShellConfig,
    components: Vec<InternalComponent>,
    binary_sensors: HashMap<String, ShellBinarySensorConfig>,
    buttons: HashMap<String, ShellButtonConfig>,
    sensors: HashMap<String, ShellSensorConfig>,
    switches: HashMap<String, ShellSwitchConfig>,
    lights: HashMap<String, ShellLightConfig>,
}

impl Module for Default {
    fn new(config_string: &String) -> Result<Self, String> {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();
        debug!("Shell config: {:?}", config);
        let mut components: Vec<InternalComponent> = Vec::new();

        let mut sensors: HashMap<String, ShellSensorConfig> = HashMap::new();
        for (_, any_sensor) in config.sensor.clone().unwrap_or_default() {
            match any_sensor.extra {
                SensorKind::shell(sensor) => {
                    let id = any_sensor.default.get_object_id();
                    components.push(InternalComponent::Sensor(InternalSensor {
                        ha: UbiSensor {
                            platform: "sensor".to_string(),
                            icon: any_sensor.default.icon.clone(),
                            device_class: any_sensor.default.device_class.clone(),
                            state_class: any_sensor.default.state_class.clone(),
                            unit_of_measurement: any_sensor.default.unit_of_measurement.clone(),
                            name: any_sensor.default.name.clone(),
                            id: id.clone(),
                        },
                        base: any_sensor.default.clone(),
                    }));
                    sensors.insert(id.clone(), sensor);
                }
                _ => {}
            }
        }

        let mut binary_sensors: HashMap<String, ShellBinarySensorConfig> = HashMap::new();
        for (_, any_sensor) in config.binary_sensor.clone().unwrap_or_default() {
            match any_sensor.extra {
                BinarySensorKind::shell(binary_sensor) => {
                    let id = any_sensor.default.get_object_id();
                    components.push(InternalComponent::BinarySensor(InternalBinarySensor {
                        ha: UbiBinarySensor {
                            platform: "sensor".to_string(),
                            icon: any_sensor.default.icon.clone(),
                            device_class: any_sensor.default.device_class.clone(),
                            name: any_sensor.default.name.clone(),
                            id: id.clone(),
                        },
                        base: any_sensor.default.clone(),
                    }));
                    binary_sensors.insert(id.clone(), binary_sensor);
                }
                _ => {}
            }
        }

        let mut buttons: HashMap<String, ShellButtonConfig> = HashMap::new();
        for (_, any_sensor) in config.button.clone().unwrap_or_default() {
            match any_sensor.extra {
                ButtonKind::shell(button) => {
                    let id = any_sensor.default.get_object_id();
                    components.push(InternalComponent::Button(InternalButton {
                        ha: UbiButton {
                            platform: "sensor".to_string(),
                            icon: any_sensor.default.icon.clone(),
                            name: any_sensor.default.name.clone(),
                            id: id.clone(),
                        },
                    }));
                    buttons.insert(id.clone(), button);
                }
                _ => {}
            }
        }

        let mut switches: HashMap<String, ShellSwitchConfig> = HashMap::new();
        for (_, any_sensor) in config.switch.clone().unwrap_or_default() {
            match any_sensor.extra {
                SwitchKind::shell(switch) => {
                    let id = any_sensor.default.get_object_id();
                    components.push(InternalComponent::Switch(InternalSwitch {
                        ha: UbiSwitch {
                            // TODO
                            platform: "sensor".to_string(),
                            icon: any_sensor.default.icon.clone(),
                            name: any_sensor.default.name.clone(),
                            id: id.clone(),
                            device_class: None,
                            assumed_state: !switch.command_state.is_some(),
                        },
                    }));
                    switches.insert(id.clone(), switch);
                }
                _ => {}
            }
        }

        let mut lights: HashMap<String, ShellLightConfig> = HashMap::new();
        for (_, any_light) in config.light.clone().unwrap_or_default() {
            match any_light.extra {
                LightKind::shell(light_config) => {
                    let id = any_light.default.get_object_id();
                    components.push(InternalComponent::Light(InternalLight {
                        ha: UbiLight {
                            platform: "light".to_string(),
                            icon: any_light.default.icon.clone(),
                            name: any_light.default.name.clone(),
                            id: id.clone(),
                            disabled_by_default: any_light
                                .default
                                .disabled_by_default
                                .unwrap_or(true),
                        },
                    }));
                    lights.insert(id.clone(), light_config);
                }
                _ => {}
            }
        }

        Ok(Default {
            config: config.shell,
            components,
            binary_sensors,
            buttons,
            sensors,
            switches,
            lights,
        })
    }

    fn components(&mut self) -> Vec<InternalComponent> {
        self.components.clone()
    }

    fn run(
        &self,
        sender: Sender<ChangedMessage>,
        mut receiver: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let config = self.config.clone();
        let binary_sensors = self.binary_sensors.clone();
        let buttons = self.buttons.clone();
        let switches = self.switches.clone();
        let sensors = self.sensors.clone();
        let lights = self.lights.clone();
        Box::pin(async move {
            let cloned_config = config.clone();
            let csender = sender.clone();

            let switches_clone = switches.clone();
            let lights_clone = lights.clone();
            // Handle Button Presses
            tokio::spawn(async move {
                let cloned_sender = csender.clone();

                while let Ok(cmd) = receiver.recv().await {
                    match cmd {
                        PublishedMessage::SwitchStateCommand { key, state } => {
                            debug!("SwitchStateChanged: {} {}", key, state);
                            if let Some(switch) = switches_clone.get(&key) {
                                // ButtonKind::shell(shell_button) => {
                                let command: String;
                                if state {
                                    debug!("Turning on switch: {}", key);
                                    command = switch.command_on.clone();
                                } else {
                                    debug!("Turning off switch: {}", key);
                                    command = switch.command_off.clone();
                                }
                                debug!("Executing command: {}", command);

                                let output = execute_command(
                                    &cloned_config,
                                    &command,
                                    &cloned_config.timeout,
                                )
                                .await
                                .unwrap();
                                // If output is empty report status code
                                if output.is_empty() {
                                    trace!("Command executed successfully with no output.");
                                } else {
                                    trace!("Command executed successfully with output: {}", output);
                                }

                                if let Some(command_state) = &switch.command_state {
                                    let output = execute_command(
                                        &cloned_config,
                                        &command_state,
                                        &cloned_config.timeout,
                                    )
                                    .await;

                                    match output {
                                        Ok(output) => {
                                            debug!("Switch {} output: {}", key, &output);
                                            let value = if output.trim().to_lowercase() == "true" {
                                                true
                                            } else if output.trim().to_lowercase() == "false" {
                                                false
                                            } else {
                                                debug!("Invalid switch sensor output: {}", output);
                                                continue;
                                            };

                                            _ = cloned_sender.send(
                                                ChangedMessage::SwitchStateChange {
                                                    key: key.clone(),
                                                    state: value,
                                                },
                                            );
                                        }
                                        Err(e) => {
                                            debug!("Error executing command: {}", e);
                                        }
                                    };
                                }
                            }
                        }
                        PublishedMessage::ButtonPressed { key } => {
                            debug!("Button pressed1: {}", key);
                            if let Some(shell_button) = buttons.get(&key) {
                                // ButtonKind::shell(shell_button) => {
                                debug!("Button pressed: {}", key);
                                debug!("Executing command: {}", shell_button.command);
                                println!("Button '{}' pressed.", key);

                                let output = execute_command(
                                    &cloned_config,
                                    &shell_button.command,
                                    &cloned_config.timeout,
                                )
                                .await
                                .unwrap();
                                // If output is empty report status code
                                if output.is_empty() {
                                    trace!("Command executed successfully with no output.");
                                } else {
                                    trace!("Command executed successfully with output: {}", output);
                                }
                            }
                        }
                        PublishedMessage::LightStateCommand {
                            key,
                            state,
                            brightness,
                            red,
                            green,
                            blue,
                        } => {
                            debug!(
                                "LightStateCommand: {} state:{} brightness:{:?} rgb:{:?},{:?},{:?}",
                                key, state, brightness, red, green, blue
                            );
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
                                // if let (Some(brightness_val), Some(brightness_cmd)) = (brightness, &light.command_brightness) {
                                //     if light.supports_brightness.unwrap_or(false) {
                                //         let brightness_command = brightness_cmd.replace("{brightness}", &brightness_val.to_string());
                                //         debug!("Executing brightness command: {}", brightness_command);
                                //         let _ = execute_command(
                                //             &cloned_config,
                                //             &brightness_command,
                                //             &cloned_config.timeout,
                                //         ).await;
                                //     }
                                // }

                                // Handle RGB color command if provided and supported
                                // if let (Some(r), Some(g), Some(b), Some(rgb_cmd)) = (red, green, blue, &light.command_rgb) {
                                //     if light.supports_rgb.unwrap_or(false) {
                                //         let rgb_command = rgb_cmd
                                //             .replace("{red}", &r.to_string())
                                //             .replace("{green}", &g.to_string())
                                //             .replace("{blue}", &b.to_string());
                                //         debug!("Executing RGB command: {}", rgb_command);
                                //         let _ = execute_command(
                                //             &cloned_config,
                                //             &rgb_command,
                                //             &cloned_config.timeout,
                                //         ).await;
                                //     }
                                // }

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

            for (key, sensor) in sensors {
                let cloned_config = config.clone();
                let cloned_sender = sender.clone();
                tokio::spawn(async move {
                    if let Some(duration) = sensor.update_interval {
                        let mut interval = time::interval(duration);
                        debug!("Sensor {} has update interval: {:?}", key, interval);
                        loop {
                            let output = execute_command(
                                &cloned_config,
                                sensor.command.as_str(),
                                &cloned_config.timeout,
                            )
                            .await;
                            // TODO: Handle long running commands (e.g. newline per value) and multivalued outputs (e.g. json)
                            match output {
                                Ok(output) => {
                                    debug!("Sensor {} output: {}", key, &output);
                                    let value = output;

                                    _ = cloned_sender.send(ChangedMessage::SensorValueChange {
                                        key: key.clone(),
                                        value: value.parse().unwrap(),
                                    });
                                }
                                Err(e) => {
                                    debug!("Error executing command: {}", e);
                                }
                            };
                            interval.tick().await;
                        }
                    } else {
                        debug!("Sensor {} has no update interval", key);
                    }
                });
            }

            for (key, switch) in switches {
                let switch = switch.clone();
                debug!("Switch State {:?}", switch);

                if let Some(command_state) = switch.command_state {
                    let cloned_config = config.clone();
                    let cloned_sender = sender.clone();
                    tokio::spawn(async move {
                        let duration = switch
                            .update_interval
                            .unwrap_or_else(|| Duration::from_secs(60));

                        let mut interval = time::interval(duration);
                        debug!(
                            "Switch {} has update interval: {:?}",
                            key,
                            interval.period()
                        );
                        loop {
                            let output = execute_command(
                                &cloned_config,
                                &command_state,
                                &cloned_config.timeout,
                            )
                            .await;
                            match output {
                                Ok(output) => {
                                    debug!("Switch {} output: {}", key, &output);
                                    let value = if output.trim().to_lowercase() == "true" {
                                        true
                                    } else if output.trim().to_lowercase() == "false" {
                                        false
                                    } else {
                                        debug!("Invalid switch sensor output: {}", output);
                                        interval.tick().await;
                                        continue;
                                    };

                                    _ = cloned_sender.send(ChangedMessage::SwitchStateChange {
                                        key: key.clone(),
                                        state: value,
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
                    warn!("Switch {} has no command_state", key);
                }
            }

            for (key, binary_sensor) in binary_sensors {
                let cloned_config = config.clone();
                let cloned_sender = sender.clone();
                debug!("Binary Sensor {}: {:?}", key, binary_sensor);

                tokio::spawn(async move {
                    if let Some(duration) = binary_sensor.update_interval {
                        let mut interval = time::interval(duration);
                        debug!("Sensor {} has update interval: {:?}", key, interval);
                        loop {
                            let output = execute_command(
                                &cloned_config,
                                binary_sensor.command.as_str(),
                                &cloned_config.timeout,
                            )
                            .await;
                            // TODO: Handle long running commands (e.g. newline per value) and multivalued outputs (e.g. json)
                            match output {
                                Ok(output) => {
                                    let value = if output.trim().to_lowercase() == "true" {
                                        true
                                    } else if output.trim().to_lowercase() == "false" {
                                        false
                                    } else {
                                        debug!("Invalid binary sensor output: {}", output);
                                        continue;
                                    };
                                    debug!("Binary Sensor '{}' output: {}", key, value);

                                    _ = cloned_sender.send(
                                        ChangedMessage::BinarySensorValueChange {
                                            key: key.clone(),
                                            value: value.clone(),
                                        },
                                    );
                                }
                                Err(e) => {
                                    debug!("Error executing command: {}", e);
                                }
                            };
                            interval.tick().await;
                        }
                    } else {
                        debug!("Sensor {} has no update interval", key);
                    }
                });
            }

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
    shell_config: &ShellConfig,
    command: &str,
    timeout: &Duration,
) -> Result<String, ShellError> {
    let shell = match shell_config.kind {
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
