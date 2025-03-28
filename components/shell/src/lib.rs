use std::time::Duration;
use log::debug;
use serde::Deserialize;
use shell_exec::{Execution, Shell};



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


pub async fn execute_command(config: &ShellConfig) {
    debug!("config: {:?}", config);
    let execution = Execution::builder()
        .shell(Shell::Bash)
        .cmd(
            r#"
            INPUT=`cat -`;
            echo "hello $INPUT"
            "#
            .to_string(),
        )
        .timeout(Duration::from_millis(10000))
        .build();

    let data = execution.execute(b"world").await.unwrap();
    debug!("data: {:?}", data);
}