use log::debug;
use os_home_core::{CoreConfig, Message, SensorKind};
use serde::Deserialize;
use shell_exec::{Execution, Shell, ShellError};
use std::{str, time::Duration};
use tokio::{sync::broadcast::{Receiver, Sender}, time};

#[derive(Debug, Copy, Clone, Deserialize)]
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
    pub _type: Option<CustomShell>,
}

pub async fn start(sender: Sender<Option<Message>>, mut receiver: Receiver<Option<Message>>, config: &CoreConfig, shell_config: &ShellConfig) {
    // Handle Button Presses
    let cloned_config = config.clone();
    tokio::spawn(async move {
        while let Ok(Some(cmd)) = receiver.recv().await {
            use Message::*;

            match cmd {
                ButtonPress { key } => {
                    if let Some(button) = &cloned_config.button.as_ref().and_then(|b| b.get(&key))
                        {
                            debug!("Button pressed: {}", key);
                            debug!("Executing command: {}", button.command);
                            // execute_command(&shell_config, &button.command).await.unwrap();
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
            match sensor.kind {
                SensorKind::Shell(shell) => {
                    debug!("Sensor {} is of type Shell", key);
                    tokio::spawn(async move {
                        let mut interval = time::interval(Duration::from_secs(10));
                
                        loop {
                            interval.tick().await;
                            let output = execute_command(&cloned_shell_config, shell.command.as_str()).await;
                            match output {
                                Ok(output) => {
                                    debug!("Sensor {} output: {}", key, output);
                                    _ = cloned_sender.send(Some(Message::SensorValueChange {
                                        key: key.clone(),
                                        value: output.clone(),
                                    }));
                                },
                                Err(e) => {
                                    debug!("Error executing command: {}", e);
                                    continue;
                                }
                            };
                        }
                    });
                }
            }
        }
    }
}

async fn execute_command(shell_config: &ShellConfig, command: &str) -> Result<String, ShellError> {
    debug!("config: {:?}", shell_config);
    let shell = match shell_config._type {
        Some(CustomShell::Zsh) => Shell::Zsh,
        Some(CustomShell::Bash) => Shell::Bash,
        Some(CustomShell::Sh) => Shell::Sh,
        Some(CustomShell::Cmd) => Shell::Cmd,
        Some(CustomShell::Powershell) => Shell::Powershell,
        Some(CustomShell::Wsl) => Shell::Wsl,
        None => Shell::default(),
    };

        
    let execution = Execution::builder()
        .shell(Shell::Bash)
        .cmd(command.to_string())
        .timeout(Duration::from_millis(10000))
        .build();

    let output = execution.execute(b"").await?;
    let output_string = str::from_utf8(&output).unwrap_or(""); 
    debug!("Command executed successfully: {}", output_string.clone());
    Ok(output_string.to_string())

}
