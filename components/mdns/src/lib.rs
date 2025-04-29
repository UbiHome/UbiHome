use log::info;
use mac_address::get_mac_address;
use oshome_core::NoConfig;
use oshome_core::{
    config_template, home_assistant::sensors::Component, ChangedMessage, Module, PublishedMessage,
};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::future;
use std::{future::Future, pin::Pin, str};
use tokio::sync::broadcast::{Receiver, Sender};
use local_ip_address::list_afinet_netifas;

#[derive(Clone, Deserialize, Debug)]
pub struct MdnsConfig {
    pub disabled: Option<bool>,
    pub ip_addresses: Option<Vec<String>>,
    pub hostname: Option<String>,
}

config_template!(mdns, Option<MdnsConfig>, NoConfig, NoConfig, NoConfig);

#[derive(Clone, Debug)]
pub struct Default {
    config: CoreConfig,
}

impl Default {
    pub fn new(config_string: &String) -> Self {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();

        Default { config: config }
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

    fn run(
        &self,
        _sender: Sender<ChangedMessage>,
        _: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let config = self.config.clone();
        info!("Starting MDNS with config: {:?}", config.mdns);
        Box::pin(async move {
            match get_mac_address() {
                Ok(Some(ma)) => {
                    let network_interfaces = list_afinet_netifas().unwrap();
                    println!("NETWORK:");

                    // TODO: Find out which one to use

                    for (name, ip) in network_interfaces.iter() {
                        println!("{}:\t{:?}", name, ip);
                    }

                    let mac_hex = ma.to_string().replace(":", "").to_uppercase();
                    let vec: Vec<std::net::IpAddr> = vec![
                        "192.168.178.33".parse::<std::net::Ipv4Addr>().unwrap().into(),
                        // std::net::Ipv6Addr::new(0, 0, 0, 0xfe80, 0x1ff, 0xfe23, 0x4567, 0x890a).into(),
                    ];
                    // let (responder, _) = libmdns::Responder::with_default_handle_and_ip_list_and_hostname(vec, "".to_string()).unwrap();
                    let responder = libmdns::Responder::new_with_ip_list(vec).unwrap();

                    let svc_name = config.oshome.name;
                    let friendly_name = config.oshome.friendly_name.unwrap_or(svc_name.clone());
                    // Native API
                    let _svc = responder.register(
                        "_esphomelib._tcp".to_owned(),
                        svc_name.clone(),
                        6053,
                        &[
                            &format!("friendly_name={}", friendly_name).to_string(),
                            "version=2024.4.2",
                            "network=wifi",
                            &format!("mac={}", mac_hex),
                            "platform=ESP32",
                            "board=esp32dev",
                        ],
                    );

                    // HTTP API
                    let _svc = responder.register(
                        "_http._tcp".to_owned(),
                        svc_name.clone(),
                        80, // TODO: Get Port?
                        &[
                            &format!("friendly_name={}", friendly_name).to_string(),
                            "version=1.0",
                            "path=/",
                        ],
                    );

                    // Gracefully shutdown the daemon
                    // mdns.shutdown().unwrap();

                    // Wait indefinitely for the interrupts
                    let future = future::pending();
                    let () = future.await;
                }
                Ok(None) => println!("No MAC address found."),
                Err(e) => println!("{:?}", e),
            }
            Ok(())
        })
    }
}
