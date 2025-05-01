mod constants;
mod service;

use flexi_logger::writers::FileLogWriter;
use flexi_logger::{detailed_format, Age, Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming};
use inquire::Text;
use oshome::CoreConfig;
use oshome_core::binary_sensor::FilterType;
use oshome_core::home_assistant::sensors::Component;
use oshome_core::{ChangedMessage, Module, PublishedMessage};

use clap::{Arg, ArgAction, Command};
use log::{debug, info, warn};
use service::{install, uninstall};
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc;
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs};
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::{runtime::Runtime, signal};
use futures_signals::signal::{MapFuture, Mutable, MutableSignal, Signal, SignalExt, SignalFuture};

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

    if let Err(e) = run(Some(&config_file), Some(shutdown_rx)) {
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
            Command::new("update")
                    .about("Update the current OSHome executable (from GitHub).")
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
            Command::new("validate")
                    .about("Validates the Configuration File.")
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
                ])
        )
}

// Embed the default configuration file at compile time
// const DEFAULT_CONFIG: &str = include_str!("../config.yaml");

fn read_base_config(path: Option<&String>) -> Result<String, String> {
    if let Some(path) = path {
        println!("Config: {}", path);
        let config_file_path = fs::canonicalize(path).unwrap();
        if let Ok(content) = fs::read_to_string(config_file_path) {
            return Ok(content);
        } else {
            warn!(
                "Failed to read the configuration file at '{}'.", //, falling back to default.",
                path
            );
        }
    }

    // Fallback to the embedded default configuration
    // println!("Config file path: BUILTIN");
    // printlm!(DEFAULT_CONFIG);
    // DEFAULT_CONFIG
    panic!("oh no!");
}

fn main() {
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
        Some(("update", sub_matches)) => {
            todo!("Update OSHome");
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
            todo!("Validate OSHome");
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
                run( config_file,None).unwrap();
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

    modules.push(Box::new(oshome_bme280::Default::new(&yaml)));
    modules.push(Box::new(oshome_gpio::Default::new(&yaml)));
    modules.push(Box::new(oshome_shell::Default::new(&yaml)));
    modules.push(Box::new(oshome_mqtt::Default::new(&yaml)));
    modules.push(Box::new(oshome_mdns::Default::new(&yaml)));
    modules.push(Box::new(oshome_api::OSHomeDefault::new(&yaml)));
    modules.push(Box::new(oshome_power_utils::Default::new(&yaml)));
    modules.push(Box::new(oshome_bluetooth_proxy::Default::new(&yaml)));
    modules.push(Box::new(oshome_web_server::Default::new(&yaml)));
    return modules
}

async fn initialize_modules(modules: &mut Vec<Box<dyn Module>>) -> Result<Vec<Component>, String> {
    let mut all_components: Vec<Component> = Vec::new();
    for module in modules.iter_mut() {
        let components = module.init();
        match components {
            Ok(mut components) => {
                println!("Module: {:?}", components);
                all_components.append(&mut components);
            }
            Err(e) => {
                debug!("Error loading component: {}", e);
            }
        }
    }
    Ok(all_components)
}

async fn run_modules(
    modules: Vec<Box<dyn Module>>,
    sender: Sender<ChangedMessage>,
    receiver: Receiver<PublishedMessage>,
) {
    for module in modules {
        // Start MQTT client in a separate task
        let tx = sender.clone();
        let rx = receiver.resubscribe();
        tokio::spawn({
            async move {
                module.run(tx, rx).await.unwrap();
            }
        });
    }
}

fn run(
    config_path: Option<&String>,
    shutdown_signal: Option<mpsc::Receiver<()>>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("OSHome - {}", VERSION);



    #[cfg(not(debug_assertions))]
    use directories::BaseDirs;
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

    logger_builder = logger_builder
        .format_for_files(detailed_format)
        .log_to_file(FileSpec::default().directory(&log_directory)) // write logs to file
        // .write_mode(WriteMode::BufferAndFlush)
        .append()
        .rotate(
            Criterion::AgeOrSize(Age::Day, 10 * 1024 * 1024),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(7),
        );

    // if cfg!(debug_assertions) {
        logger_builder = logger_builder.duplicate_to_stdout(Duplicate::Debug);
    // }

    let mut logger = logger_builder.start().unwrap();

    println!("LogDirectory: {}", log_directory.display());
    
    
    let config_string: String = read_base_config(config_path).expect("Failed to load base configuration");

    let config = serde_yaml::from_str::<CoreConfig>(&config_string).unwrap();
    if let Some(logger_config) = config.logger.clone() {
        logger.reset_flw(&FileLogWriter::builder(
            FileSpec::default().directory(
                logger_config
                .clone()
                    .directory
                    .unwrap_or(log_directory.to_string_lossy().to_string()),
            ),
        )).unwrap();

        logger
            .parse_and_push_temp_spec(logger_config.get_flexi_logger_spec())
            .unwrap();
    };

    warn!("Config: {:?}", &config);

    // Spawn the root task
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut modules = get_all_modules(&config_string);
        log::info!("Loaded {} modules", modules.len());
        let components = initialize_modules(&mut modules).await.unwrap();

        let (internal_tx, modules_rx) = broadcast::channel::<PublishedMessage>(16);
        let (modules_tx, mut internal_rx) = broadcast::channel::<ChangedMessage>(16);
        let internal_tx_clone = internal_tx.clone();

        // Workaround for https://github.com/Pauan/rust-signals/issues/75
        let mut signal_map_binary_sensor: HashMap<String, Mutable<Option<Option<bool>>>> = HashMap::new();
        for (key, any_sensor) in config.binary_sensor.clone().unwrap_or_default() {
            let mutable: Mutable<Option<Option<bool>>> = Mutable::new(Option::None);
            signal_map_binary_sensor.insert(
                key,
                mutable.clone());

                let mutable_clone = mutable.clone();
                tokio::spawn(async move {
                    // let future: SignalFuture<MutableSignal<bool>> = mutable_clone.signal()
                    println!("Filters: {:?}", any_sensor.default.filters);

                    let mut signal = mutable_clone.signal().boxed();
                    for filter in any_sensor.default.filters.unwrap_or_default() {
                        println!("Filter: {:?}", filter);
                        match filter.filter {
                            FilterType::delayed_on(time) => {
                                    // println!("delayed_on");
                                    let time_clone = time.clone();
                                signal = signal.map_future(move |value| {
                                    let time_clone = time_clone.clone();
                                    Box::pin(async move {
                                        let value = value.and_then(|v| v);
                                        if let Some(v) = value {
                                            if v {
                                                // println!("Before delay");
    
                                                sleep(time_clone);
                                                // println!("After delay");
                                                return value;
                                            }
                                        }
                                        return value;
                                    })
                                }).boxed();
                            }
                            FilterType::delayed_off(time) => {
                                let time_clone = time.clone();
                                signal = signal.map_future(move |value| {
                                    println!("delayed_off");
                                    let time_clone = time_clone.clone();

                                    Box::pin(async move {
                                        let value = value.and_then(|v| v);
                                        // println!("delayed_off value {:?}", value);

                                        if let Some(v) = value {
                                            if v {
                                                // println!("Before delay");
    
                                                sleep(time_clone);
                                                // println!("After delay");
                                                return value;
                                            }
                                        }
                                        return value;
                                    })
                                }).boxed();
                            }
                            FilterType::invert(_) => {
                                signal = signal.map(|value| {
                                    println!("invert");
                                    if value.is_some() {
                                        return Some(Some(!value.and_then(|v| v).unwrap()));
                                    }
                                    return value;
                                }).boxed();
                            }
                        }
                    
                    }
                    // React to signal changes
                    signal
                        .for_each(|value| {
                            println!("Value: {:?}", value);
                            async {}
                        })
                        .await;



                    // // Debounce future for "off" values
                    // future
                    //     .for_each(|value| {
                    //         // This code is run for the current value of my_state,
                    //         // and also every time my_state changes
                    //         println!("From signal: {}", value);
                    //         async {}
                    //     })
                    //     .await;
                });
            
        }

        tokio::spawn({
            async move {
                while let Ok(cmd) = internal_rx.recv().await {
                    debug!("Received command: {:?}", cmd);
                    let publish_cmd: Option<PublishedMessage>;
                    match cmd {
                        ChangedMessage::ButtonPress { key } => {
                            publish_cmd = Some(PublishedMessage::ButtonPressed { key });
                        }
                        ChangedMessage::SensorValueChange { key, value } => {
                            println!("SensorValueChange: {}", value);

                            // signal_map_binary_sensor.get(&key).map(|signal| {
                            //     signal.set(value.to_string());
                            // });
                            publish_cmd = Some(PublishedMessage::SensorValueChanged { key, value });
                        }
                        ChangedMessage::BinarySensorValueChange { key, value } => {
                            println!("BinarySensorValueChange: {}", value);
                            signal_map_binary_sensor.get(&key).map(|signal| {
                                signal.set(Some(Some(value)));
                            });
                            publish_cmd =
                                Some(PublishedMessage::BinarySensorValueChanged { key, value });
                        }
                        ChangedMessage::BluetoothProxyMessage(msg)=> {
                            publish_cmd = Some(PublishedMessage::BluetoothProxyMessage(msg));
                        
                        }
                    }
                    if let Some(pcmd) = publish_cmd {
                        debug!("Publishing command: {:?}", pcmd);
                        internal_tx_clone.send(pcmd).unwrap();
                    }
                }
            }
        });

        run_modules(modules, modules_tx.clone(), modules_rx).await;

        println!("Components: {:?}", components);
        internal_tx
            .send(PublishedMessage::Components {
                components: components,
            })
            .unwrap();

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
                        Err(mpsc::RecvTimeoutError::Timeout) => (),
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
