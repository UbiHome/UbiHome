mod constants;
mod service;

use directories::BaseDirs;
use flexi_logger::{detailed_format, Age, Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming};
use inquire::Text;
use oshome_core::home_assistant::sensors::Component;
use oshome_core::{Message, Module};

use clap::{Arg, ArgAction, Command};
use log::{info, warn, error};
use service::{install, uninstall};
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use std::{env, fs};
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::{runtime::Runtime, signal};

#[cfg(target_os = "windows")]
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
};
#[cfg(target_os = "windows")]
define_windows_service!(ffi_service_main, windows_service_main);

#[cfg(target_os = "windows")]
fn windows_service_main(_arguments: Vec<std::ffi::OsString>) {
    use std::time::Duration;
    
    use constants::SERVICE_NAME;
    info!("Starting Windows service...");

    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();
    let status_handle =
        service_control_handler::register(SERVICE_NAME, move |control_event| match control_event {
            ServiceControl::Stop => {
                shutdown_tx.send(()).unwrap();
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        })
        .unwrap();

    status_handle
        .set_service_status(ServiceStatus {
            process_id: None,
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::from_secs(0),
        })
        .unwrap();

    let mut dir = env::current_exe().unwrap();
    dir.pop();
    dir.push("config.yaml");
    let config = dir.to_string_lossy().to_string();
    info!("Config file path: {}", config);
    if let Err(e) = run(Some(&config), Some(shutdown_rx)) {
        error!("{}", e)
    }

    info!("Service is stopping...");
    status_handle
        .set_service_status(ServiceStatus {
            process_id: None,
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::from_secs(0),
        })
        .unwrap();
}

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
                .short('c')
                .long("configuration")
                .help("Optional configuration file. If not provided, the default configuration will be used.")
                .default_value("config.yaml")
                // .group("tests")
        ])
        .subcommand(
            Command::new("install")
                    .about("Install OSHome")
                    .arg(
                        Arg::new("location")
                        .help( "The location to install OSHome.")
                    )
        )
        .subcommand(
            Command::new("uninstall")
                    .about("Uninstall OSHome")
                    .arg(
                        Arg::new("location")
                        .help( "The location where OSHome is installed.")
                    )
        )
        .subcommand(
            Command::new("run")
                .about("Run OSHome manually.")
                .args([
                    Arg::new("as-windows-service")
                        .long("as-windows-service")
                        .help("Flag to identify if run as windows service.")
                        .hide(true)
                        .action(ArgAction::SetTrue)
                        .num_args(0)
                        // .group("tests")
                ])
        )
}

// Embed the default configuration file at compile time
// const DEFAULT_CONFIG: &str = include_str!("../config.yaml");


fn read_base_config(path: Option<&String>) -> Result<String, String> {
    if let Some(path) = path {
        let config_file_path = fs::canonicalize(path).unwrap();
        if let Ok(content) = fs::read_to_string(config_file_path) {
            return Ok(content)
        } else {
            warn!(
                "Failed to read the configuration file at '{}'.", //, falling back to default.",
                path
            );
        }
    }

    // Fallback to the embedded default configuration
    // DEFAULT_CONFIG
    panic!("oh no!");
}

fn main() {
    println!("Starting OSHome - {}", VERSION);

    
    #[cfg(not(debug_assertions))]
    let base_dirs = BaseDirs::new().expect("Failed to get base directories");
    #[cfg(not(debug_assertions))]
    let log_directory = base_dirs.data_local_dir();

    #[cfg(debug_assertions)]
    let log_directory = Path::new("./");
    
    #[cfg(not(debug_assertions))]
    let log_level = "info";

    #[cfg(debug_assertions)]
    let log_level = "debug";

    let mut logger_builder = Logger::try_with_env_or_str(log_level).unwrap();

    logger_builder = logger_builder.format_for_files(detailed_format)
        .log_to_file(FileSpec::default().directory(&log_directory)) // write logs to file
        // .write_mode(WriteMode::BufferAndFlush)
        .append()
        .rotate(
            Criterion::AgeOrSize(Age::Day, 10 * 1024 * 1024),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(7),
        );
    
    if cfg!(debug_assertions) {
        logger_builder = logger_builder.duplicate_to_stdout(Duplicate::Debug);
    }

    //let mut logger = 
    logger_builder
        .start()
        .unwrap();
    
    // TODO: Implement logger entry
    // logger.parse_and_push_temp_spec("info, critical_mod = trace");
    // logger.pop_temp_spec();
    println!("LogDirectory: {}", log_directory.display());

    let matches = cli().get_matches();
    let config_file = matches.try_get_one::<String>("configuration_file").unwrap();
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let default_installation_path = "/usr/bin/oshome";
    #[cfg(target_os = "windows")]
    let default_installation_path = "C:\\Program Files\\oshome";

    match matches.subcommand() {
        Some(("install", sub_matches)) => {
            let location = sub_matches.try_get_one::<String>("location").unwrap();
            if let Some(location) = location {
                // Perform installation logic here
                install(location);
            } else {
                let location = Text::new("Where do you want to install OSHome?")
                    .with_default(default_installation_path)
                    .prompt();
                install(location.unwrap().as_str());
            }
        }
        Some(("uninstall", sub_matches)) => {
            let location = sub_matches.try_get_one::<String>("location").unwrap();
            if let Some(location) = location {
                // Perform installation logic here
                uninstall(location);
            } else {
                let location = Text::new("OS Home is not installed under the default location. Where should the uninstall script run?").with_default(default_installation_path).prompt();
                uninstall(location.unwrap().as_str());
            }
        }
        Some(("run", sub_matches)) => {
            let is_windows_service = sub_matches.get_one::<bool>("as-windows-service").unwrap();
            if *is_windows_service {
                #[cfg(target_os = "linux")]
                // Run as a Windows service
                info!("Running as Windows service");
                #[cfg(target_os = "windows")]
                use windows_service::service_dispatcher;
                #[cfg(target_os = "windows")]
                service_dispatcher::start(constants::SERVICE_NAME, ffi_service_main).unwrap();
                panic!("Running as a Windows service is not supported on Linux.");
            } else {
                // Run normally
                run(config_file.clone(), None).unwrap();
            }
        }
        _ => {
            println!("No subcommand was used");
        }
    }
    std::process::exit(0);
}


fn get_all_modules(yaml: &String) -> Vec<Box<dyn Module>> {
    let mut modules: Vec<Box<dyn Module>> = Vec::new();
    #[cfg(target_os = "windows")]
    {
        // TODO: Windows module
    }
    #[cfg(target_os = "linux")]
    {
        modules.push(Box::new(oshome_bme280::Default::new(&yaml)));
        modules.push(Box::new(oshome_gpio::Default::new(&yaml)));
    }
    modules.push(Box::new(oshome_shell::Default::new(&yaml)));
    modules.push(Box::new(oshome_mqtt::Default::new(&yaml)));
    modules
}

async fn initialize_modules(modules: &mut Vec<Box<dyn Module>>) -> Result<Vec<Component>, String> {
    let mut all_components: Vec<Component> = Vec::new();
    for module in modules.iter_mut() {
        let mut components = module.init().unwrap();
        all_components.append(&mut components);
    }
    Ok(all_components)
}

// async fn run_modules(
//     modules: Vec<Box<dyn Module>>,
//     sender: Sender<Option<Message>>,
//     receiver: Receiver<Option<Message>>,
// ) {
//     for module in modules {
//         let tx2 = sender.clone();
//         tokio::spawn({
//             let tx2 = tx2.clone();
//             async move {
//                 let rx2 = tx2.subscribe();
//                 let _ = module.run(tx2, rx2).await.unwrap();
//             }
//         });
//     }
// }

async fn run_modules(modules: Vec<Box<dyn Module>>,
    sender: Sender<Option<Message>>,
    _receiver: Receiver<Option<Message>>) {
    for module in modules {
        // Start MQTT client in a separate task
        // if let Some(mqtt_config) = config.mqtt {
        let tx2 = sender.clone();
        // let mut module = module.clone();
        tokio::spawn({
            let tx2 = tx2.clone();
            async move {
                let rx2 = tx2.subscribe();
                module.run(tx2, rx2).await.unwrap();
            }
        });
        // }
    }
    // Ok(all_components)
}

fn run(
    config_file: Option<&String>,
    shutdown_signal: Option<mpsc::Receiver<()>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let rt = Runtime::new().unwrap();

    // Spawn the root task
    rt.block_on(async {
        let config: String = read_base_config(config_file).expect("Failed to load base configuration");

        let mut modules = get_all_modules(&config);

        let _ = initialize_modules(&mut modules).await.unwrap();

        let (tx, rx) = broadcast::channel(16);

        run_modules(modules, tx, rx).await;




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

        if let Some(some_shutdown_signal) = shutdown_signal {
            let shutdown_event = async {
                loop {
                    // Poll shutdown event.
                    match some_shutdown_signal.recv_timeout(Duration::from_secs(1)) {
                        // Break the loop either upon stop or channel disconnect
                        Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => break,

                        // Continue work if no events were received within the timeout
                        Err(mpsc::RecvTimeoutError::Timeout) => ()
                    };
                }
            };
            tokio::select! {
                _ = ctrl_c => {},
                _ = terminate => {},
                _ = shutdown_event => {},
            }
        } else {
            tokio::select! {
                _ = ctrl_c => {},
                _ = terminate => {},
            }
        }
    });
    info!("Shutdown complete");
    Ok(())
}
