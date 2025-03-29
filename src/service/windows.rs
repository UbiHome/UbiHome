use std::{env, fs, path::Path};

use log::{debug, info};

use crate::constants;

pub async fn install(location: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Installing OSHome to {}", location);
    info!(" - Creating Folder at {}", location);
    fs::create_dir_all(location).expect("Unable to create directory");

    let new_path = Path::new(location).join("oshome.exe");
    info!(" - Copying Binary to {}", new_path.display());
    fs::copy(env::current_exe().unwrap(), &new_path).expect("Unable to copy file");

    let current_dir = env::current_dir().unwrap().to_string_lossy().to_string();
    let config_file_path = Path::new(&current_dir).join("config.yaml");
    let new_config_file_path = Path::new(location).join("config.yaml");
    
    info!(" - Copying config.yml to {}", new_config_file_path.display());
    fs::copy(config_file_path, &new_config_file_path).expect("Unable to copy file");

    info!(" - Creating Windows Service");

    use std::ffi::OsString;
    use windows_service::{
        service::{ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceType},
        service_manager::{ServiceManager, ServiceManagerAccess},
    };

    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_info = ServiceInfo {
        name: OsString::from(constants::SERVICE_NAME),
        display_name: OsString::from(constants::SERVICE_NAME),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: new_path,
        launch_arguments: vec![OsString::from("-c"), OsString::from(new_config_file_path), OsString::from("run"), OsString::from("--as-windows-service")],
        dependencies: vec![],
        account_name: None, // run as System
        account_password: None,
    };
    let service = service_manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;
    service.set_description(constants::SERVICE_DESCRIPTION)?;

    info!(" - TODO: Create Readme...");

    Ok(())
}

pub async fn uninstall(location: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::{
        thread::sleep,
        time::{Duration, Instant},
    };

    use windows_service::{
        service::{ServiceAccess, ServiceState},
        service_manager::{ServiceManager, ServiceManagerAccess},
    };
    use windows_sys::Win32::Foundation::ERROR_SERVICE_DOES_NOT_EXIST;

    info!("Uninstalling OSHome to");

    
    info!(" - TODO: Cleanup Logs...");


    info!(" - Deleting Folder at {}", location);
    fs::remove_dir_all(location).expect("Unable to delete directory");

    info!(" - Removing Service");

    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
    let service = service_manager.open_service(constants::SERVICE_NAME, service_access)?;

    // Our handle to it is not closed yet. So we can still query it.
    if service.query_status()?.current_state != ServiceState::Stopped {
        // If the service cannot be stopped, it will be deleted when the system restarts.
        service.stop()?;
    }
    // The service will be marked for deletion as long as this function call succeeds.
    // However, it will not be deleted from the database until it is stopped and all open handles to it are closed.
    service.delete()?;
    // Explicitly close our open handle to the service. This is automatically called when `service` goes out of scope.
    // drop(service);

    // Win32 API does not give us a way to wait for service deletion.
    // To check if the service is deleted from the database, we have to poll it ourselves.
    let start = Instant::now();
    let timeout = Duration::from_secs(5);
    while start.elapsed() < timeout {
        if let Err(windows_service::Error::Winapi(e)) =
            service_manager.open_service(constants::SERVICE_NAME, ServiceAccess::QUERY_STATUS)
        {
            if e.raw_os_error() == Some(ERROR_SERVICE_DOES_NOT_EXIST as i32) {
                info!("service is deleted.");
                return Ok(());
            }
        }
        sleep(Duration::from_secs(1));
    }
    info!("service is marked for deletion.");
    Ok(())
}