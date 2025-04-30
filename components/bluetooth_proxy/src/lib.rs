use log::info;
use oshome_core::NoConfig;
use oshome_core::{
    config_template, home_assistant::sensors::Component, ChangedMessage, Module, PublishedMessage,
};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::future;
use std::{future::Future, pin::Pin, str};
use tokio::sync::broadcast::{Receiver, Sender};
use btleplug::api::{
    bleuuid::BleUuid, Central, CentralEvent, Manager as _, Peripheral, ScanFilter,
};
use btleplug::platform::{Adapter, Manager};
use futures::stream::StreamExt;

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().nth(0).unwrap()
}

#[derive(Clone, Deserialize, Debug)]
pub struct BluetoothProxyConfig {
    pub disabled: Option<bool>,
}

config_template!(mdns, Option<BluetoothProxyConfig>, NoConfig, NoConfig, NoConfig);

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
        info!("Starting Bluetooth Proxy with config: {:?}", config.mdns);
        Box::pin(async move {

            let manager = Manager::new().await.expect("Failed to create bluetooth manager");

            // get the first bluetooth adapter
            // connect to the adapter
            let central = get_central(&manager).await;
        
            let central_state = central.adapter_state().await.expect("No adapter found");
            println!("CentralState: {:?}", central_state);
        
            // Each adapter has an event stream, we fetch via events(),
            // simplifying the type, this will return what is essentially a
            // Future<Result<Stream<Item=CentralEvent>>>.
            let mut events = central.events().await?;
        
            // start scanning for devices
            central.start_scan(ScanFilter::default()).await?;
        
            // Print based on whatever the event receiver outputs. Note that the event
            // receiver blocks, so in a real program, this should be run in its own
            // thread (not task, as this library does not yet use async channels).
            while let Some(event) = events.next().await {
                match event {
                    CentralEvent::DeviceDiscovered(id) => {
                        let peripheral = central.peripheral(&id).await?;
                        let properties = peripheral.properties().await?;
                        let name = properties
                            .and_then(|p| p.local_name)
                            .map(|local_name| format!("Name: {local_name}"))
                            .unwrap_or_default();
                        println!("DeviceDiscovered: {:?} {}", id, name);
                    }
                    CentralEvent::StateUpdate(state) => {
                        println!("AdapterStatusUpdate {:?}", state);
                    }
                    CentralEvent::DeviceConnected(id) => {
                        println!("DeviceConnected: {:?}", id);
                    }
                    CentralEvent::DeviceDisconnected(id) => {
                        println!("DeviceDisconnected: {:?}", id);
                    }
                    CentralEvent::ManufacturerDataAdvertisement {
                        id,
                        manufacturer_data,
                    } => {
                        println!(
                            "ManufacturerDataAdvertisement: {:?}, {:?}",
                            id, manufacturer_data
                        );
                    }
                    CentralEvent::ServiceDataAdvertisement { id, service_data } => {
                        println!("ServiceDataAdvertisement: {:?}, {:?}", id, service_data);
                    }
                    CentralEvent::ServicesAdvertisement { id, services } => {
                        let services: Vec<String> =
                            services.into_iter().map(|s| s.to_short_string()).collect();
                        println!("ServicesAdvertisement: {:?}, {:?}", id, services);
                    }
                    _ => {}
                }
            }
            Ok(())
        })
    }
}