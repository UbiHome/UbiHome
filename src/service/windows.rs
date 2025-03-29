use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher, Result,
};


pub fn install(location: &str){
    info!("Installing OSHome to {}", location);
    const SERVICE_NAME: &str = "RustWindowsService";
        let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)
            .unwrap();
        let service_info = ServiceInfo {
            name: SERVICE_NAME.into(),
            display_name: "Rust Windows Service Example".into(),
            service_type: ServiceType::OWN_PROCESS,
            start_type: ServiceStartType::AutoStart,
            error_control: ServiceErrorControl::Normal,
            executable_path: std::env::current_exe().unwrap(),
            launch_arguments: vec![],
            dependencies: vec![],
            account_name: None, // Local system account
            account_password: None,
        };
        manager.create_service(service_info, ServiceAccess::START_STOP).unwrap();
        println!("Service installed successfully");
        
}

pub fn uninstall() {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::DELETE)
        .unwrap();
    let service = manager.open_service(SERVICE_NAME, ServiceAccess::DELETE).unwrap();
    service.delete().unwrap();
    println!("Service uninstalled successfully");
}