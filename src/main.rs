mod constants;

use commands::run;
use commands::un_install::{install, uninstall};
use commands::update::update;
use flexi_logger::{Duplicate, Logger};
use inquire::Text;

use clap::{Arg, ArgAction, Command};
use log::{debug, error, info, trace, warn};
use std::{env, fs};

mod commands;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const CARGO_PKG_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

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
    use log::error;
    use std::time::Duration;

    use constants::SERVICE_NAME;
    println!("Starting Windows service...");
    println!("Args: {:?}", _arguments);

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
    let config_file = dir.to_string_lossy().to_string();

    if let Err(e) = run::run(Some(&config_file), Some(shutdown_rx)) {
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

fn cli() -> Command {
    Command::new("UbiHome")
        .about(format!("UbiHome {}\n\n{}\nDocumentation: https://ubihome.github.io/\nHomepage: {}" ,VERSION, DESCRIPTION, CARGO_PKG_HOMEPAGE))
        .version(VERSION)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .args([
            Arg::new("configuration_file")
                .short('c')
                .long("configuration")
                .help("Optional configuration file. If not provided, the default configuration will be used.")
                .default_value("config.yaml"),
            Arg::new("log_level")
                .long("log-level")
                .help("The log level (overwrites the setting from config.yaml).")

        ])
        .subcommand(
            Command::new("run")
                .about("Run UbiHome manually.")
                .args([
                    // TODO:
                    // #[cfg(target_os = "windows")]
                    Arg::new("as-windows-service")
                        .long("as-windows-service")
                        .help("Flag to identify if run as windows service.")
                        .hide(true)
                        .action(ArgAction::SetTrue)
                        .num_args(0)
                ])
        )
        .subcommand(
            Command::new("validate")
                    .about("Validates the configuration file.")
        )
        .subcommand(
            Command::new("install")
                    .about("Install UbiHome")
                    .arg(
                        Arg::new("location")
                        .help( "The location to install UbiHome.")
                    )
        )
        .subcommand(
            Command::new("update")
                    .about("Update the current UbiHome executable (from GitHub).")
        )
        .subcommand(
            Command::new("uninstall")
                    .about("Uninstall UbiHome")
                    .arg(
                        Arg::new("location")
                        .help( "The location where UbiHome is installed.")
                    )
        )
}

fn main() {
    let matches = cli().get_matches();
    let config_file = matches.try_get_one::<String>("configuration_file").unwrap();
    let log_level = matches.try_get_one::<String>("log_level").unwrap();

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let default_installation_path = "/usr/bin/ubihome";
    #[cfg(target_os = "windows")]
    let default_installation_path = "C:\\Program Files\\ubihome";

    match matches.subcommand() {
        Some(("install", sub_matches)) => {
            let location = sub_matches.try_get_one::<String>("location").unwrap();
            if let Some(location) = location {
                // Perform installation logic here
                install(location);
            } else {
                let location = Text::new("Where do you want to install UbiHome?")
                    .with_default(default_installation_path)
                    .prompt();
                install(location.unwrap().as_str());
            }
        }
        Some(("update", sub_matches)) => {
            if let Some(log_level) = log_level {
                let mut logger_builder = Logger::try_with_env_or_str(log_level).unwrap();
                logger_builder = logger_builder.duplicate_to_stdout(Duplicate::Debug);
                logger_builder.start().unwrap();

            }
            update().unwrap();
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
        Some(("validate", sub_matches)) => {
            todo!("Validate UbiHome");
        }
        Some(("run", sub_matches)) => {
            println!("UbiHome - {}", VERSION);
            let is_windows_service = sub_matches.get_one::<bool>("as-windows-service").unwrap();
            if *is_windows_service {
                // Run as a Windows service
                info!("Running as Windows service");
                #[cfg(target_os = "windows")]
                use windows_service::service_dispatcher;
                #[cfg(target_os = "windows")]
                service_dispatcher::start(constants::SERVICE_NAME, ffi_service_main).unwrap();
                panic!("Running as a Windows service is not supported on Linux.");
            } else {
                // Run normally
                run::run(config_file, None).unwrap();
            }
        }
        _ => {
            println!("No subcommand was used");
        }
    }
    std::process::exit(0);
}
