use log::info;
use std::{env, fs};
use std::{path::Path, time::Duration};
use log::debug;
use shell_exec::{Execution, Shell};
use std::str;

pub async fn install(location: &str){
    use crate::constants::{SERVICE_DESCRIPTION, SERVICE_NAME};

    info!("Installing OSHome to {}", location);
    info!(" - Creating Folder at {}", location);
    fs::create_dir_all(location).expect("Unable to create directory");

    let new_path = Path::new(location).join("oshome");
    info!(" - Copying Binary to {}", new_path.display());
    fs::copy(env::current_exe().unwrap(), new_path).expect("Unable to copy file");

    let service_file = format!("{}.service", SERVICE_NAME);

    let systemd_file_path = Path::new("/etc/systemd/system").join(&service_file);
    info!(" - Creating Systemd Service at {}", systemd_file_path.to_string_lossy().to_string());
    let systemd_file: String = format!("[Unit]
Description={}
After=network-online.target

[Service]
Type=simple
Restart=always
RestartSec=1
ExecStart={}oshome
StandardOutput=journal

[Install]
WantedBy=multi-user.target", SERVICE_DESCRIPTION, location);

    fs::write(systemd_file_path, systemd_file).expect("Unable to write file");
    info!("- Running Commands for installation");
    execute_command("systemctl daemon-reload").await;
    execute_command(format!("systemctl enable {}", service_file).as_str()).await;
    execute_command(format!("systemctl start {}", service_file).as_str()).await;

}

async fn execute_command(command: &str) {
    let execution = Execution::builder()
    .shell(Shell::default())
    .timeout(Duration::from_secs(5))
    .cmd(command.to_string())
    .build();

    let output = execution.execute(b"").await;
    match output {
        Ok(output) => {
            let output_string = str::from_utf8(&output).unwrap_or(""); 
            debug!("Command executed successfully: {}", output_string);
        }
        Err(e) => {
            debug!("Error executing command: {}", e);
        }
    }
}

pub async fn uninstall(location: &str){

    info!("Uninstalling OSHome at {}", location);
    // "systemctl daemon-reload"
    // "systemctl enable oshome.service"
    // "systemctl start oshome.service"

}
