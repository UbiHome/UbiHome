mod service;

use inquire::Text;
use oshome_shell::start;

use clap::{Arg, Command};
use env_logger;
use env_logger::Env;
use log::{info, warn};
use oshome::Config;
use oshome_mqtt::start_mqtt_client;
use service::{install, uninstall};
use std::fs;
use tokio::signal;
use tokio::sync::broadcast;

#[cfg(target_os = "windows")]
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher, Result,
};
#[cfg(target_os = "windows")]
define_windows_service!(ffi_service_main, service_main);

// use oshome::
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const CARGO_PKG_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

fn cli() -> Command {
    Command::new("oshome")
        .about(format!("OSHome - {}\n\n{}\n{}" ,VERSION, DESCRIPTION, CARGO_PKG_HOMEPAGE))
        .version(VERSION)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .args([
            Arg::new("configuration_file")
                .long("configuration_file")
                .help("Optional configuration file. If not provided, the default configuration will be used.")
                .default_value("config.yaml")
                // .group("tests")
        ])
        .subcommand(
            Command::new("install")
                    .about("Insall OSHome")
                    .arg(
                        Arg::new("location")
                        .help( "The location to install OSHome")
                    )
        )
        .subcommand(
            Command::new("run")
                .about("Run OSHome")
        )
}


// Embed the default configuration file at compile time
// const DEFAULT_CONFIG: &str = include_str!("../config.yaml");

// Function to read the YAML configuration file
fn read_full_config(path: Option<&String>) -> Result<Config, serde_yaml::Error> {
    if let Some(path) = path {
        let config_file_path = fs::canonicalize(path).unwrap();
        if let Ok(content) = fs::read_to_string(config_file_path) {
            return serde_yaml::from_str(&content);
        } else {

            warn!(
                "Failed to read the configuration file at '{}', falling back to default.",
                path
            );
        }
    }

    // Fallback to the embedded default configuration
    // serde_yaml::from_str(DEFAULT_CONFIG)
    panic!("oh no!");
}

fn read_base_config(path: Option<&String>) -> Result<oshome_core::CoreConfig, serde_yaml::Error> {
    if let Some(path) = path {
        let config_file_path = fs::canonicalize(path).unwrap();
        if let Ok(content) = fs::read_to_string(config_file_path) {
            return serde_yaml::from_str(&content);
        } else {
            warn!(
                "Failed to read the configuration file at '{}', falling back to default.",
                path
            );
        }
    }

    // Fallback to the embedded default configuration
    // serde_yaml::from_str(DEFAULT_CONFIG)
    panic!("oh no!");
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    let matches = cli().get_matches();
    let config_file = matches.try_get_one::<String>("configuration_file").unwrap();
    #[cfg(target_os = "linux")]
    let default_installation_path = "/usr/bin/oshome";
    #[cfg(target_os = "windows")]
    let default_installation_path = "C:\\Program Files\\oshome";

    match matches.subcommand() {
        Some(("install", sub_matches)) => {
            let location = sub_matches.try_get_one::<String>("location").unwrap();
            if let Some(location) = location {
                // Perform installation logic here
                install(location).await;
            } else {

                let location = Text::new("Where do you want to install OSHome?").with_default(default_installation_path).prompt();
                install(location.unwrap().as_str()).await;
            }
        }
        Some(("uninstall", sub_matches)) => {
            let location = sub_matches.try_get_one::<String>("location").unwrap();
            if let Some(location) = location {
                // Perform installation logic here
                uninstall(location).await;
            } else {

                let location = Text::new("OS Home is not installed under the default location. Where should the uninstall script run?").with_default(default_installation_path).prompt();
                uninstall(location.unwrap().as_str()).await;
            }
        }
        Some(("run", _sub_matches)) => {
            run(config_file.clone()).await;
        }
        _ => {
            println!("No subcommand was used");
        }
    }
}

async fn run(config_file: Option<&String>){
    // let cli = Args::parse();

    #[cfg(target_os = "windows")]
    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();
    #[cfg(target_os = "windows")]
    let status_handle = service_control_handler::register(
        SERVICE_NAME,
        move |control_event| match control_event {
            ServiceControl::Stop => {
                shutdown_tx.send(()).unwrap();
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        },
    )?;
    #[cfg(target_os = "windows")]
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(10).as_secs() as u32,
    })?;
    // Read the configuration file or use the embedded default
    let config: Config =
        read_full_config(config_file).unwrap_or_else(|err| {
            warn!(
                "Failed to parse configuration: {}, using default configuration.",
                err
            );
            read_full_config(None).expect("Failed to load embedded default configuration")
        });

    let base_config: oshome_core::CoreConfig = read_base_config(config_file)
        .unwrap_or_else(|err| {
            warn!(
                "Failed to parse configuration: {}, using default configuration.",
                err
            );
            read_base_config(None).expect("Failed to load embedded default configuration")
        });



    let (tx, mut _rx) = broadcast::channel(16);

    let mqtt_base_config = base_config.clone();
    // Start MQTT client in a separate task
    if let Some(mqtt_config) = config.mqtt {
        let tx2 = tx.clone();
        tokio::spawn(async move {
            let rx2 = tx2.subscribe();
            start_mqtt_client(tx2, rx2, &mqtt_base_config, &mqtt_config).await;
        });
    }

    if let Some(shell_config) = config.shell {
        let tx2 = tx.clone();
        let shell_base_config = base_config.clone();

        tokio::spawn(async move {
            let rx2 = tx2.subscribe();
            start(tx2, rx2, &shell_base_config, &shell_config).await;

            // while let Ok(Some(cmd)) = rx.recv().await {
            //     use Message::*;

            //     match cmd {
            //         ButtonPress { key } => {
            //             if let Some(button) = &base_config.button.as_ref().and_then(|b| b.get(&key))
            //             {
            //                 debug!("Button pressed: {}", key);
            //                 debug!("Executing command: {}", button.command);
            //             } else {
            //                 debug!("Button pressed: {}", key);
            //                 warn!("No Action found?!");
            //             }
            //         },
            //         SensorValueChange { key, value } => {
            //             debug!("Sensor value changed: {} = {}", key, value);
            //             // Handle sensor value change
            //         }
            //     }
            // }
        });
    };
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    #[cfg(target_os = "windows")]
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: 0,
    })?;
    info!("Shutdown complete");
    std::process::exit(0);
}




// Define the service name

// Main service entry point
// fn service_main(_arguments: Vec<std::ffi::OsString>) {
//     if let Err(e) = run_service() {
//         error!("Service failed: {:?}", e);
//     }
// }
// fn run_service() -> Result<()> {
//     // Define service status
    
//     // Main service loop
//     info!("Service is running...");
//     loop {
//         if shutdown_rx.try_recv().is_ok() {
//             info!("Shutdown signal received");
//             break;
//         }
//         thread::sleep(Duration::from_secs(1));
//     }
//     info!("Service is stopping...");

//     Ok(())
// }