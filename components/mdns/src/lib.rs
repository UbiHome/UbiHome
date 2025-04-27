use log::info;
use oshome_core::NoConfig;
use oshome_core::{config_template, home_assistant::sensors::Component, ChangedMessage, Module, PublishedMessage};
use std::future;
use std::{future::Future, pin::Pin, str};
use tokio::sync::broadcast::{Receiver, Sender};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;


#[derive(Clone, Deserialize, Debug)]
pub struct MdnsConfig {
    pub disabled: Option<bool>,
}

config_template!(mdns, Option<MdnsConfig>, NoConfig, NoConfig, NoConfig);

#[derive(Clone, Debug)]
pub struct Default {
    config: CoreConfig
} 


impl Default {
    pub fn new(config_string: &String) -> Self {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();

        Default {
            config: config,
        }
    }
}

impl Module for Default {
    fn validate(&mut self) -> Result<(), String> {
        Ok(())
    }


    fn init(&mut self) -> Result<Vec<Component>, String> {
        let components: Vec<Component> = Vec::new();

        Ok(components)
    }

    fn run(&self,
        _sender: Sender<ChangedMessage>,
        _: Receiver<PublishedMessage>,
) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>{
        let config = self.config.clone();
        info!("Starting MDNS with config: {:?}", config.mdns);
        Box::pin(async move {
        let responder = libmdns::Responder::new().unwrap();

            // Native API
            let _svc = responder.register(
                "_esphomelib._tcp.local.".to_owned(),
                "Test Device".to_owned(),
                6053,
                &["friendly_name=Hello", "version=1.0", "mac=00:00:00:00:00:00"],
            );

            // HTTP API
            let _svc = responder.register(
                "_http._tcp.local.".to_owned(),
                "Test Device".to_owned(),
                80, // TODO: Get Port?
                &["version=1.0"],
            );

            // Wait indefinitely for the interrupts
            let future = future::pending();
            let () = future.await;


            // use mdns_sd::{ServiceDaemon, ServiceInfo};

            // // Create a daemon
            // let mdns = ServiceDaemon::new().expect("Failed to create daemon");

            // // Create a service info.
            // let service_type = "_esphomelib._tcp.local.";
            // let instance_name = "Test Device";
            // let ip = "192.168.1.12";
            // let host_name = "test.local.";
            // let port = 6053;
            // let properties = [("friendly_name", "Hello"), ("version", "1.0")];

            // let my_service = ServiceInfo::new(
            //     service_type,
            //     instance_name,
            //     host_name,
            //     ip,
            //     port,
            //     &properties[..],
            // ).unwrap();

            // // Register with the daemon, which publishes the service.
            // mdns.register(my_service).expect("Failed to register our service");

            // // Gracefully shutdown the daemon
            // std::thread::sleep(std::time::Duration::from_secs(1));
            // mdns.shutdown().unwrap();
            Ok(()) 
        })
     }

} 


