use log::debug;
use oshome_core::{binary_sensor::BinarySensorKind, sensor::SensorKind, CoreConfig, Message};
use serde::Deserialize;
use shell_exec::{Execution, Shell, ShellError};
use std::{str, time::Duration};
use tokio::{sync::broadcast::{Receiver, Sender}, time};
use duration_str::deserialize_duration;

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

pub async fn start(sender: Sender<Option<Message>>, mut receiver: Receiver<Option<Message>>, config: &CoreConfig, shell_config: &ShellConfig) {
    // Handle Button Presses
    let cloned_config = config.clone();
    let cloned_shell_config = shell_config.clone();
    tokio::spawn(async move {
        while let Ok(Some(cmd)) = receiver.recv().await {
            use Message::*;

            match cmd {
                ButtonPress { key } => {
                    if let Some(button) = &cloned_config.button.as_ref().and_then(|b| b.get(&key))
                        {
                            debug!("Button pressed: {}", key);
                            debug!("Executing command: {}", button.command);
                            println!("Button '{}' pressed.", key);
                            
                            let output = execute_command(&cloned_shell_config, &button.command, &cloned_shell_config.timeout).await.unwrap();
                            // If output is empty report status code
                            if output.is_empty() {
                                println!("Command executed successfully with no output.");
                            } else {
                                println!("Command executed successfully with output: {}", output);
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
        for (key, sensor) in sensors {
            let cloned_shell_config = shell_config.clone();
            let cloned_sender = sender.clone();
            let cloned_sensor = sensor.clone();
            match sensor.kind {
                SensorKind::Shell(shell) => {
                    debug!("Sensor {} is of type Shell", key);
                    tokio::spawn(async move {
                        if let Some(duration) = cloned_sensor.update_interval {
                            let mut interval = time::interval(duration);
                            debug!("Sensor {} has update interval: {:?}", key, interval);
                            loop {
                                interval.tick().await;
                                let output = execute_command(&cloned_shell_config, shell.command.as_str(), &duration).await;
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
                _ => { }
            }
        }
    }

    if let Some(binary_sensors) = config.binary_sensor.clone() {
        for (key, binary_sensor) in binary_sensors {
            let cloned_shell_config = shell_config.clone();
            let cloned_sender = sender.clone();
            let cloned_sensor = binary_sensor.clone();
            match binary_sensor.kind {
                BinarySensorKind::Shell(shell) => {
                    debug!("BinarySensor {} is of type Shell", key);
                    tokio::spawn(async move {
                        if let Some(duration) = shell.update_interval {
                            let mut interval = time::interval(duration);
                            debug!("Sensor {} has update interval: {:?}", key, interval);
                            loop {
                                interval.tick().await;
                                let output = execute_command(&cloned_shell_config, shell.command.as_str(), &duration).await;
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
                _ => { }
            }
        }
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
