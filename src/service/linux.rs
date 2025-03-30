use log::info;
use std::{env, fs};
use std::{path::Path, time::Duration};
use log::debug;
use shell_exec::{Execution, Shell};
use std::str;

fn service_file() -> String {
    use crate::constants::SERVICE_NAME;
    format!("{}.service", SERVICE_NAME)
}

pub const SYSTEMD_FILE_PATH: &str = "/etc/systemd/system";


pub async fn install(location: &str){
    use crate::constants::SERVICE_DESCRIPTION;

    println!("Installing OSHome to {}", location);
    println!(" - Creating Folder at {}", location);
    fs::create_dir_all(location).expect("Unable to create directory");

    let new_path = Path::new(location).join("oshome");
    println!(" - Copying Binary to {}", new_path.display());
    fs::copy(env::current_exe().unwrap(), new_path).expect("Unable to copy file");

    let service_file = service_file();

    let systemd_file_path = Path::new(SYSTEMD_FILE_PATH).join(&service_file);
    println!(" - Creating Systemd Service file {}", systemd_file_path.to_string_lossy().to_string());
    let systemd_file: String = format!("[Unit]
Description={}
After=network-online.target

[Service]
Type=simple
Restart=always
RestartSec=1
ExecStart={}/oshome run
StandardOutput=journal
WorkingDirectory={}


[Install]
WantedBy=multi-user.target", SERVICE_DESCRIPTION, location, location);

    fs::write(systemd_file_path, systemd_file).expect("Unable to write file");
    println!("- Installing Systemd Service");
    execute_command("systemctl daemon-reload").await;
    execute_command(format!("systemctl enable {}", service_file).as_str()).await;
    execute_command(format!("systemctl start {}", service_file).as_str()).await;

    println!("Successfully installed!");
    println!("Query the status via `systemctl status {}`", service_file);
    println!("Or follow the log with `journalctl -u {}`", service_file);

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

    if location.chars().filter(|c| *c == '/').count() < 2 {
        println!("To not shoot yourself in the foot, please provide a longer path");
        return;
    }

    println!("Uninstalling OSHome at {}", location);
    println!(" - Remove Folder at {}", location);
    fs::remove_dir(location).unwrap();

    println!("- Removing Systemd Service");
    let service_file = service_file();
    execute_command(format!("systemctl stop {}", service_file).as_str()).await;
    execute_command(format!("systemctl disable {}", service_file).as_str()).await;
    let systemd_file_path = Path::new(SYSTEMD_FILE_PATH).join(&service_file);
    fs::remove_file(systemd_file_path).expect("Unable to remove file");
    
    execute_command("systemctl daemon-reload").await;
    println!("TODO: remove log files?");


}
