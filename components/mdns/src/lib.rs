use log::{debug, info};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::future;
use std::net::IpAddr;
use std::{future::Future, pin::Pin, str};
use tokio::sync::broadcast::{Receiver, Sender};
use ubihome_core::features::ip::{get_ip_address, get_network_mac_address};
use ubihome_core::internal::sensors::InternalComponent;
use ubihome_core::NoConfig;
use ubihome_core::{
    config_template, home_assistant::sensors::Component, ChangedMessage, Module, PublishedMessage,
};

#[derive(Clone, Deserialize, Debug)]
pub struct MdnsConfig {
    pub disabled: Option<bool>,
    pub ip: Option<IpAddr>,
    pub mac: Option<String>,
    pub ip_addresses: Option<Vec<String>>,
    pub hostname: Option<String>,
}

config_template!(
    mdns,
    Option<MdnsConfig>,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig
);

#[derive(Clone, Debug)]
pub struct Default {
    config: CoreConfig,
}

impl Module for Default {
    fn new(config_string: &String) -> Result<Self, String> {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();

        Ok(Default { config: config })
    }
    fn components(&mut self) -> Vec<InternalComponent> {
        let components: Vec<InternalComponent> = Vec::new();

        components
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
            let default_ip = get_ip_address().unwrap();

            let ip = config.mdns.clone().and_then(|c| c.ip).unwrap_or(default_ip);
            debug!("Advertising IP: {:?}", ip);

            let default_mac = get_network_mac_address(ip).unwrap();

            let mac = config
                .mdns
                .clone()
                .and_then(|c| c.mac)
                .unwrap_or(default_mac);
            debug!("Advertising Mac: {:?}", mac);

            let responder: libmdns::Responder;
            // let (responder, _) = libmdns::Responder::with_default_handle_and_ip_list_and_hostname(vec, "".to_string()).unwrap();
            responder = libmdns::Responder::new_with_ip_list(vec![ip]).unwrap();

            let svc_name = config.ubihome.name;
            let friendly_name = config.ubihome.friendly_name.unwrap_or(svc_name.clone());
            // Native API
            let _svc = responder.register(
                "_esphomelib._tcp".to_owned(),
                svc_name.clone(),
                6053,
                &[
                    &format!("friendly_name={}", friendly_name).to_string(),
                    "version=2024.4.2",
                    "network=wifi",
                    &format!("mac={}", str::replace(&mac, ":", "").to_ascii_lowercase()),
                    "platform=ESP32",
                    "board=ubihome",
                    // api_encryption=Noise_NNpsk0_25519_ChaChaPoly_SHA256
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
            Ok(())
        })
    }
}
