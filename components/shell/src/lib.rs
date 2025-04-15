use log::debug;
use oshome_core::{config_template, home_assistant::sensors::Component,Message, Module};
use serde::{Deserialize, Deserializer};
use shell_exec::{Execution, Shell, ShellError};
use std::{future::Future, pin::Pin, str, time::Duration};
use tokio::{sync::broadcast::{Receiver, Sender}, time};
use duration_str::deserialize_duration;
use std::collections::HashMap;


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
    pub command: String
}

#[derive(Clone, Deserialize, Debug)]
pub struct ShellSensorConfig {
    pub command: String
}

#[derive(Clone, Deserialize, Debug)]
pub struct ShellButtonConfig {
    pub command: String,
}

config_template!(shell, ShellConfig, ShellButtonConfig, ShellBinarySensorConfig, ShellBinarySensorConfig);


pub struct Default {
    config: CoreConfig
    
}

impl Default {
    pub fn new(config_string: &String) -> Self {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();

        Default {
            config
        }
    }
}

impl Module for Default {


    fn validate(&mut self) -> Result<(), String> {
        Ok(())
    }

    
    fn init(&mut self) -> Result<Vec<Component>, String> {
        let components: Vec<Component> = Vec::new();

        Ok(components)
    }

    fn run(
        &self,
        sender: Sender<Option<Message>>,
        mut receiver: Receiver<Option<Message>>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let config = self.config.clone();
        Box::pin(async move {
             // Handle Button Presses
            let cloned_config = config.clone();
            tokio::spawn(async move {
                while let Ok(Some(cmd)) = receiver.recv().await {
                    use Message::*;

                    match cmd {
                        ButtonPress { key } => {
                            if let Some(button) = cloned_config.button.as_ref().and_then(|b| b.get(&key))
                                {
                                    match &button.extra {
                                        ButtonKind::shell(shell_button) => {
                                            debug!("Button pressed: {}", key);
                                            debug!("Executing command: {}", shell_button.command);
                                            println!("Button '{}' pressed.", key);
                                            
                                            let output = execute_command(&cloned_config.shell, &shell_button.command, &cloned_config.shell.timeout).await.unwrap();
                                            // If output is empty report status code
                                            if output.is_empty() {
                                                println!("Command executed successfully with no output.");
                                            } else {
                                                println!("Command executed successfully with output: {}", output);
                                            }
                                        }
                                        _ => {}
                                    }
                                } else {
                                    debug!("Button pressed: {}", key);
                                }
                        }
                        _ => {
                            debug!("Ignored message type: {:?}", cmd);
                        }
                    }
                }
            });


            
            if let Some(sensors) = config.sensor.clone() {
                for (key, any_sensor) in sensors {
                    let cloned_config = config.clone();
                    let cloned_sender = sender.clone();
                    let cloned_sensor = any_sensor.clone();
                    debug!("Sensor {} is of type Shell", key);
                    match any_sensor.extra {
                        SensorKind::shell(sensor) => {
                            tokio::spawn(async move {
                                if let Some(duration) = cloned_sensor.update_interval {
                                    let mut interval = time::interval(duration);
                                    debug!("Sensor {} has update interval: {:?}", key, interval);
                                    loop {
                                        interval.tick().await;
                                        let output = execute_command(&cloned_config.shell, sensor.command.as_str(), &duration).await;
                                        // TODO: Handle long running commands (e.g. newline per value) and multivalued outputs (e.g. json)
                                        match output {
                                            Ok(output) => {
                                                debug!("Sensor {} output: {}", key, &output);

                                                let value = output;
                                                println!("Sensor '{}' output: {}", key, &value);

                                                _ = cloned_sender.send(Some(Message::SensorValueChange {
                                                    key: key.clone(),
                                                    value: value,
                                                }));
                                            },
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
                        _ => {}
                    }
                }
            }

            if let Some(binary_sensors) = config.binary_sensor.clone() {
                for (key, binary_sensor) in binary_sensors {
                    let cloned_config = config.clone();
                    let cloned_sender = sender.clone();

                    match binary_sensor.extra {
                        BinarySensorKind::shell(sensor) => {

                            debug!("BinarySensor {} is of type Shell", key);
                            tokio::spawn(async move {
                                if let Some(duration) = sensor.update_interval {
                                    let mut interval = time::interval(duration);
                                    debug!("Sensor {} has update interval: {:?}", key, interval);
                                    loop {
                                        interval.tick().await;
                                        let output = execute_command(&cloned_config.shell, sensor.command.as_str(), &duration).await;
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
                                                println!("Binary Sensor '{}' output: {}", key, value);

                                                _ = cloned_sender.send(Some(Message::BinarySensorValueChange {
                                                    key: key.clone(),
                                                    value: value.clone(),
                                                }));
                                            },
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
                        _ => {}
                    }
                    }
                }
            Ok(())
        })
    }
}


async fn execute_command(shell_config: &ShellConfig, command: &str, timeout: &Duration) -> Result<String, ShellError> {
    let shell = match shell_config.kind {
        Some(CustomShell::Zsh) => Shell::Zsh,
        Some(CustomShell::Bash) => Shell::Bash,
        Some(CustomShell::Sh) => Shell::Sh,
        Some(CustomShell::Cmd) => Shell::Cmd,
        Some(CustomShell::Powershell) => Shell::Powershell,
        Some(CustomShell::Wsl) => Shell::Wsl,
        None => Shell::default(),
    };
    debug!("config: {:?}", shell);

        
    let execution = Execution::builder()
        .shell(shell)
        .cmd(command.to_string())
        .timeout(*timeout)
        .build();

    let output = execution.execute(b"").await?;
    let output_string = str::from_utf8(&output).unwrap_or(""); 
    debug!("Command executed successfully: {}", output_string);
    Ok(output_string.to_string())

}
