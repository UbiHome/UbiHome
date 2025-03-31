use oshome_core::{CoreConfig, Message};
use serde::Deserialize;
use std::str;
use tokio::sync::broadcast::{Receiver, Sender};


#[derive(Clone, Deserialize, Debug)]
pub struct MdnsConfig {
    pub disabled: Option<bool>,
}

pub async fn start(
    shell_config: &MdnsConfig,
) {
    // // Only disable if explicitly set to true
    // if !shell_config.disabled.unwrap_or(false) {
    //     return;
    // }

    // let responder = libmdns::Responder::new().unwrap();

    // // Native API
    // let _svc = responder.register(
    //     "_esphomelib._tcp.local.".to_owned(),
    //     "Test Device".to_owned(),
    //     6053,
    //     &["friendly_name=Hello", "version=1.0", "mac=00:00:00:00:00:00"],
    // );

    // // HTTP API
    // let _svc = responder.register(
    //     "_http._tcp.local.".to_owned(),
    //     "Test Device".to_owned(),
    //     80, // TODO: Get Port?
    //     &["version=1.0"],
    // );


    use mdns_sd::{ServiceDaemon, ServiceInfo};
    use std::collections::HashMap;

    // Create a daemon
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");

    // Create a service info.
    let service_type = "_esphomelib._tcp.local.";
    let instance_name = "Test Device";
    let ip = "192.168.1.12";
    let host_name = "test.local.";
    let port = 6053;
    let properties = [("friendly_name", "Hello"), ("version", "1.0")];

    let my_service = ServiceInfo::new(
        service_type,
        instance_name,
        host_name,
        ip,
        port,
        &properties[..],
    ).unwrap();

    // Register with the daemon, which publishes the service.
    mdns.register(my_service).expect("Failed to register our service");

    // // Gracefully shutdown the daemon
    // std::thread::sleep(std::time::Duration::from_secs(1));
    // mdns.shutdown().unwrap();
}
