use duration_str::deserialize_duration;
use log::debug;
use ubihome_core::{
    config_template,
    home_assistant::sensors::{Component, HABinarySensor, HAButton, HASensor},
    ChangedMessage, Module, PublishedMessage,
};
use serde::{Deserialize, Deserializer};
use shell_exec::{Execution, Shell, ShellError};
use std::collections::HashMap;
use std::{future::Future, pin::Pin, str, time::Duration};
use tokio::{
    sync::broadcast::{Receiver, Sender},
    time,
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
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub update_interval: Option<Duration>,
    pub command: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct ShellSensorConfig {
    pub command: String,
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub update_interval: Option<Duration>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct ShellButtonConfig {
    pub command: String,
}

config_template!(
    shell,
    ShellConfig,
    ShellButtonConfig,
    ShellBinarySensorConfig,
    ShellSensorConfig
);

pub struct Default {
    config: ShellConfig,
    components: Vec<Component>,
    binary_sensors: HashMap<String, ShellBinarySensorConfig>,
    buttons: HashMap<String, ShellButtonConfig>,
    sensors: HashMap<String, ShellSensorConfig>,
}

impl Default {
    pub fn new(config_string: &String) -> Self {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();
        // info!("Shell config: {:?}", config);
        let mut components: Vec<Component> = Vec::new();

        let mut sensors: HashMap<String, ShellSensorConfig> = HashMap::new();
        for (_, any_sensor) in config.sensor.clone().unwrap_or_default() {
            match any_sensor.extra {
                SensorKind::shell(sensor) => {
                    let id = any_sensor.default.get_object_id();
                    components.push(Component::Sensor(HASensor {
                        platform: "sensor".to_string(),
                        icon: any_sensor.default.icon.clone(),
                        unique_id: Some(id.clone()),
                        device_class: any_sensor.default.device_class.clone(),
                        state_class: any_sensor.default.state_class.clone(),
                        unit_of_measurement: any_sensor.default.unit_of_measurement.clone(),
                        name: any_sensor.default.name.clone(),
                        object_id: id.clone(),
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
                    components.push(Component::BinarySensor(HABinarySensor {
                        platform: "sensor".to_string(),
                        icon: any_sensor.default.icon.clone(),
                        unique_id: Some(id.clone()),
                        device_class: any_sensor.default.device_class.clone(),
                        name: any_sensor.default.name.clone(),
                        object_id: id.clone(),
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
            config: config.shell,
            components,
            binary_sensors,
            buttons,
            sensors,
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
        let binary_sensors = self.binary_sensors.clone();
        let buttons = self.buttons.clone();
        let sensors = self.sensors.clone();
        Box::pin(async move {
            let cloned_config = config.clone();
            // Handle Button Presses
            tokio::spawn(async move {
                while let Ok(cmd) = receiver.recv().await {
                    match cmd {
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
                                    println!("Command executed successfully with no output.");
                                } else {
                                    println!(
                                        "Command executed successfully with output: {}",
                                        output
                                    );
                                }
                            }
                        }
                        _ => {
                            debug!("Ignored message type: {:?}", cmd);
                        }
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
                            interval.tick().await;
                            let output =
                                execute_command(&cloned_config, sensor.command.as_str(), &duration)
                                    .await;
                            // TODO: Handle long running commands (e.g. newline per value) and multivalued outputs (e.g. json)
                            match output {
                                Ok(output) => {
                                    debug!("Sensor {} output: {}", key, &output);
                                    let value = output;

                                    _ = cloned_sender.send(ChangedMessage::SensorValueChange {
                                        key: key.clone(),
                                        value: value,
                                    });
                                }
                                Err(e) => {
                                    debug!("Error executing command: {}", e);
                                    continue;
                                }
                            };
                        }
                    } else {
                        debug!("Sensor {} has no update interval", key);
                    }
                });
            }

            for (key, binary_sensor) in binary_sensors {
                let cloned_config = config.clone();
                let cloned_sender = sender.clone();

                tokio::spawn(async move {
                    if let Some(duration) = binary_sensor.update_interval {
                        let mut interval = time::interval(duration);
                        debug!("Sensor {} has update interval: {:?}", key, interval);
                        loop {
                            interval.tick().await;
                            let output = execute_command(
                                &cloned_config,
                                binary_sensor.command.as_str(),
                                &duration,
                            )
                            .await;
                            // TODO: Handle long running commands (e.g. newline per value) and multivalued outputs (e.g. json)
                            match output {
                                Ok(output) => {
                                    debug!("Sensor {} output: {}", key, output);

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
                                    continue;
                                }
                            };
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
