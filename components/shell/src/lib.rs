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
use ubihome_core::home_assistant::sensors::UbiSwitch;
use ubihome_core::internal::sensors::InternalSwitch;
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

fn default_timeout_none() -> Option<Duration>{
    None
}



config_template!(
    shell,
    ShellConfig,
    ShellButtonConfig,
    ShellBinarySensorConfig,
    ShellSensorConfig,
    ShellSwitchConfig
);

pub struct Default {
    config: ShellConfig,
    components: Vec<InternalComponent>,
    binary_sensors: HashMap<String, ShellBinarySensorConfig>,
    buttons: HashMap<String, ShellButtonConfig>,
    sensors: HashMap<String, ShellSensorConfig>,
    switches: HashMap<String, ShellSwitchConfig>,
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
                        },
                    }));
                    switches.insert(id.clone(), switch);
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
        Box::pin(async move {
            let cloned_config = config.clone();
            let switches_clone = switches.clone();
            // Handle Button Presses
            tokio::spawn(async move {
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
                            let output =
                                execute_command(&cloned_config, sensor.command.as_str(), &cloned_config.timeout)
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
                        debug!("Switch {} has update interval: {:?}", key, interval);
                        loop {
                            let output = execute_command(
                                &cloned_config,
                                    &command_state,
                                    &cloned_config.timeout,
                            )
                            .await;
                            // TODO: Handle long running commands (e.g. newline per value) and multivalued outputs (e.g. json)
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

    let output = execution.execute(b"").await?;
    let output_string = str::from_utf8(&output).unwrap_or("");
    debug!("Command '{}' executed: {}", command, output_string);
    Ok(output_string.to_string())
}
