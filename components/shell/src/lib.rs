use log::debug;
use serde::Deserialize;
use shell_exec::{Execution, Shell};
use std::{str, time::Duration};

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

pub async fn execute_command(config: &ShellConfig, command: &str) {
    debug!("config: {:?}", config);
    let execution = Execution::builder()
        .shell(Shell::Bash)
        .cmd(command.to_string())
        .timeout(Duration::from_millis(10000))
        .build();

    let data = execution.execute(b"").await.unwrap();
    let command_output = str::from_utf8(&data).unwrap_or("");
    debug!("data: {:?}", command_output);
}
