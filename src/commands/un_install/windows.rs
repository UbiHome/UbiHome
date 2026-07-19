use std::{env, ffi::OsStr, fs, path::Path};

use log::debug;
use std::ffi::OsString;
use std::{
    thread::sleep,
    time::{Duration, Instant},
};
use windows_service::{
    service::{
        Service, ServiceAccess, ServiceAction, ServiceActionType, ServiceErrorControl,
        ServiceFailureActions, ServiceFailureResetPeriod, ServiceInfo, ServiceStartType,
        ServiceState, ServiceType,
    },
    service_manager::{ServiceManager, ServiceManagerAccess},
};

use windows_sys::Win32::Foundation::ERROR_SERVICE_DOES_NOT_EXIST;

use crate::constants;

/// Connects to the SCM and opens the UbiHome service, requesting the access rights needed
/// by both install and uninstall, and returning both handles so the manager can still be
/// reused (e.g. to create the service if it does not exist yet).
fn get_service() -> windows_service::Result<(ServiceManager, Service)> {
    let manager_access = ServiceManagerAccess::CONNECT
        | ServiceManagerAccess::CREATE_SERVICE
        | ServiceManagerAccess::ENUMERATE_SERVICE;
    let service_access = ServiceAccess::START
        | ServiceAccess::STOP
        | ServiceAccess::CHANGE_CONFIG
        | ServiceAccess::QUERY_STATUS
        | ServiceAccess::DELETE;

    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;
    let service = service_manager.open_service(constants::SERVICE_NAME, service_access)?;
    Ok((service_manager, service))
}

pub async fn install(location: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Installing UbiHome to {}", location);
    println!(" - Creating Folder at {}", location);
    fs::create_dir_all(location).expect("Unable to create directory");

    // Opened up front (before copying the binary) so a running service can be stopped first;
    // overwriting a running executable fails with a sharing violation on Windows.
    let existing_service = get_service();

    if let Ok((_, service)) = &existing_service {
        if service.query_status()?.current_state != ServiceState::Stopped {
            // If the service cannot be stopped, it will be deleted when the system restarts.
            service.stop()?;
        }
    }

    let current_exe = env::current_exe().unwrap();
    let new_path = Path::new(location).join("ubihome.exe");

    // Compare against the canonicalized target directory (not `new_path` itself) so this
    // also works when no binary exists at the target location yet.
    let already_in_place = new_path
        .parent()
        .and_then(|dir| dir.canonicalize().ok())
        .map(|dir| dir.join("ubihome.exe"))
        .and_then(|target| current_exe.canonicalize().ok().map(|exe| exe == target))
        .unwrap_or(false);

    if already_in_place {
        println!(
            " - Binary is already at {}, skipping copy",
            new_path.display()
        );
    } else {
        println!(" - Copying Binary to {}", new_path.display());
        fs::copy(&current_exe, &new_path).expect("Unable to copy file");
    }

    let service_info = ServiceInfo {
        name: OsString::from(constants::SERVICE_NAME),
        display_name: OsString::from(constants::SERVICE_NAME),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: new_path,
        launch_arguments: vec![
            OsString::from("run"),
            OsString::from("--as-windows-service"),
        ],
        dependencies: vec![],
        account_name: None, // run as System
        account_password: None,
    };

    let service = if let Ok((_service_manager, service)) = existing_service {
        debug!("Service already exists, updating...");
        service.set_description(constants::SERVICE_DESCRIPTION)?;
        service.change_config(&service_info)?;
        println!(" - Service updated.");
        service
    } else {
        let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
        let service_access =
            ServiceAccess::START | ServiceAccess::CHANGE_CONFIG | ServiceAccess::QUERY_STATUS;
        let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;
        let service = service_manager.create_service(&service_info, service_access)?;
        service.set_description(constants::SERVICE_DESCRIPTION)?;
        println!(" - Created Windows Service");
        service
    };

    // Ensure the SCM restarts the service if the process exits unexpectedly or expected
    service.update_failure_actions(ServiceFailureActions {
        reset_period: ServiceFailureResetPeriod::After(Duration::from_secs(86400)),
        reboot_msg: None,
        command: None,
        actions: Some(vec![
            ServiceAction {
                action_type: ServiceActionType::Restart,
                delay: Duration::from_secs(5),
            },
            ServiceAction {
                action_type: ServiceActionType::Restart,
                delay: Duration::from_secs(5),
            },
            ServiceAction {
                action_type: ServiceActionType::Restart,
                delay: Duration::from_secs(5),
            },
        ]),
    })?;
    // The SCM only applies failure actions to genuine crashes by default; this flag makes
    // it also apply them to a clean exit (code 0)
    service.set_failure_actions_on_non_crash_failures(true)?;

    if service.query_status()?.current_state == ServiceState::Running {
        println!(" - Service is already running.");
    } else {
        service.start::<&OsStr>(&[])?;
        println!(" - Service started.");
    }

    Ok(())
}

pub async fn uninstall(location: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Uninstalling UbiHome to");

    if let Ok((service_manager, service)) = get_service() {
        print!(" - Removing Service... ");

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
                    println!("service is deleted.");
                    break;
                }
            }
            sleep(Duration::from_secs(1));
        }
        println!("service is marked for deletion.");
    } else {
        println!(" - Service already removed");
    }

    println!(" - Deleting Folder and contents at {}", location);
    fs::remove_dir_all(location).expect("Unable to delete directory");

    Ok(())
}
