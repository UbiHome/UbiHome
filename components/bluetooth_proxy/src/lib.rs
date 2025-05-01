use btleplug::api::BDAddr;
use btleplug::api::{
    bleuuid::BleUuid, Central, CentralEvent, Manager as _, Peripheral, ScanFilter,
};
use log::{debug, info, trace};
use oshome_core::{
    config_template, home_assistant::sensors::Component, ChangedMessage, Module, PublishedMessage,
};
use oshome_core::{BluetoothProxyMessage, NoConfig};
use serde::{ser, Deserialize, Deserializer};
use std::collections::HashMap;
use std::future;
use std::{future::Future, pin::Pin, str};
use tokio::sync::broadcast::{Receiver, Sender};
// use bluest::Adapter;
use btleplug::platform::{Adapter, Manager, PeripheralId};
use futures::stream::StreamExt;

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().nth(0).unwrap()
}

#[derive(Clone, Deserialize, Debug)]
pub struct BluetoothProxyConfig {
    pub disabled: Option<bool>,
}

config_template!(
    mdns,
    Option<BluetoothProxyConfig>,
    NoConfig,
    NoConfig,
    NoConfig
);

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
        sender: Sender<ChangedMessage>,
        _: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let config = self.config.clone();
        info!("Starting Bluetooth Proxy with config: {:?}", config.mdns);
        Box::pin(async move {
            // TODO: Check if bluetooth is enabled

            // let adapter = Adapter::default().await.ok_or("Bluetooth adapter not found")?;
            // adapter.wait_available().await?;

            // println!("starting scan");
            // let mut scan = adapter.scan(&[]).await?;
            // println!("scan started");
            // while let Some(discovered_device) = scan.next().await {
            //     discovered_device.adv_data.manufacturer_data;
            //    println!(
            //        "{}{}: {:?}",
            //        discovered_device.device.name().as_deref().unwrap_or("(unknown)"),
            //        discovered_device
            //            .rssi
            //            .map(|x| format!(" ({}dBm)", x))
            //            .unwrap_or_default(),
            //        discovered_device.adv_data.services
            //    );
            // }

            let manager = Manager::new()
                .await
                .expect("Failed to create bluetooth manager");

            // get the first bluetooth adapter
            // connect to the adapter
            let central = get_central(&manager).await;

            let central_state = central.adapter_state().await.expect("No adapter found");
            info!("CentralState: {:?}", central_state);

            // Each adapter has an event stream, we fetch via events(),
            // simplifying the type, this will return what is essentially a
            // Future<Result<Stream<Item=CentralEvent>>>.
            let mut events = central.events().await?;

            // start scanning for devices
            central.start_scan(ScanFilter::default()).await?;

            while let Some(event) = events
                // .filter(|e| {
                //     future::ready(match e {
                //         CentralEvent::DeviceDiscovered(_) => true,
                //         CentralEvent::ManufacturerDataAdvertisement {
                //             id,
                //             manufacturer_data,
                //         } => true,
                //         CentralEvent::ServiceDataAdvertisement { id, service_data } => true,
                //         _ => false,
                //     })
                // })
                .next()
                .await
            {
                match event {
                    CentralEvent::DeviceUpdated(id) => {
                        let peripheral = central.peripheral(&id).await?;
                        let properties = peripheral.properties().await?;

                        let rssi = properties
                            .as_ref()
                            .and_then(|p| p.rssi.clone())
                            .unwrap_or_default();
                        let name = properties
                            .as_ref()
                            .and_then(|p| p.local_name.clone())
                            .unwrap_or_default();
                        let services = properties
                            .as_ref()
                            .and_then(|p| Some(p.services.clone()))
                            .unwrap_or_default();
                        let service_data = properties
                            .as_ref()
                            .and_then(|p| Some(p.service_data.clone()))
                            .unwrap_or_default();
                        let manufacturer_data = properties
                            .as_ref()
                            .and_then(|p| Some(p.manufacturer_data.clone()))
                            .unwrap_or_default();
                        trace!("DeviceUpdated: {:?}, {:?}, {:?}", id, rssi, properties);

                        sender
                            .send(ChangedMessage::BluetoothProxyMessage(
                                BluetoothProxyMessage {
                                    reason: "DeviceUpdated".to_string(),
                                    name: name,
                                    mac: id.to_string(),
                                    rssi: rssi,
                                    service_data: service_data
                                        .iter()
                                        .map(|(k, v)| (k.to_string(), v.clone()))
                                        .collect(),
                                    service_uuids: services.iter().map(|s| s.to_string()).collect(),
                                    manufacturer_data: manufacturer_data
                                        .iter()
                                        .map(|(k, v)| (k.to_string(), v.clone()))
                                        .collect(),
                                },
                            ))
                            .unwrap();
                    }
                    CentralEvent::ManufacturerDataAdvertisement {
                        id,
                        manufacturer_data,
                    } => {
                        let peripheral = central.peripheral(&id).await?;
                        let properties = peripheral.properties().await?;

                        let rssi = properties
                            .as_ref()
                            .and_then(|p| p.rssi.clone())
                            .unwrap_or_default();
                        let name = properties
                            .as_ref()
                            .and_then(|p| p.local_name.clone())
                            .unwrap_or_default();
                        let services = properties
                            .as_ref()
                            .and_then(|p| Some(p.services.clone()))
                            .unwrap_or_default();
                        let service_data = properties
                            .as_ref()
                            .and_then(|p| Some(p.service_data.clone()))
                            .unwrap_or_default();
                        trace!(
                            "ManufacturerDataAdvertisement: {:?}, {:?}, {:?}",
                            id,
                            rssi,
                            properties
                        );

                        // sender
                        //     .send(ChangedMessage::BluetoothProxyMessage(
                        //         BluetoothProxyMessage {
                        //             reason: "ManufacturerDataAdvertisement".to_string(),
                        //             name: name,
                        //             mac: id.to_string(),
                        //             rssi: rssi,
                        //             service_data: service_data
                        //                 .iter()
                        //                 .map(|(k, v)| (k.to_string(), v.clone()))
                        //                 .collect(),
                        //             service_uuids: services.iter().map(|s| s.to_string()).collect(),
                        //             manufacturer_data: manufacturer_data
                        //                 .iter()
                        //                 .map(|(k, v)| (k.to_string(), v.clone()))
                        //                 .collect(),
                        //         },
                        //     ))
                        //     .unwrap();
                    }
                    CentralEvent::ServiceDataAdvertisement { id, service_data } => {
                        let peripheral = central.peripheral(&id).await?;
                        let properties = peripheral.properties().await?;

                        let rssi = properties
                            .as_ref()
                            .and_then(|p| p.rssi.clone())
                            .unwrap_or_default();
                        let name = properties
                            .as_ref()
                            .and_then(|p| p.local_name.clone())
                            .unwrap_or_default();

                        trace!("ServiceDataAdvertisement: {:?}, {:?}", id, service_data);
                        // sender
                        //     .send(ChangedMessage::BluetoothProxyMessage(
                        //         BluetoothProxyMessage {
                        //             reason: "ServiceDataAdvertisement".to_string(),
                        //             name: name,
                        //             mac: id.to_string(),
                        //             rssi: rssi,
                        //             service_data: service_data
                        //                 .iter()
                        //                 .map(|(k, v)| (k.to_string(), v.clone()))
                        //                 .collect(),
                        //             service_uuids: service_data
                        //                 .keys()
                        //                 .map(|k| k.to_string())
                        //                 .collect(),
                        //             manufacturer_data: HashMap::new(),
                        //         },
                        //     ))
                        //     .unwrap();
                    }
                    CentralEvent::DeviceDiscovered(id) => {
                        let peripheral = central.peripheral(&id).await?;
                        let properties = peripheral.properties().await?;
                        let rssi = properties
                            .as_ref()
                            .and_then(|p| p.rssi.clone())
                            .unwrap_or_default();
                        let name = properties
                            .as_ref()
                            .and_then(|p| p.local_name.clone())
                            .unwrap_or_default();
                        let services = properties
                            .as_ref()
                            .and_then(|p| Some(p.services.clone()))
                            .unwrap_or_default();
                        let service_data = properties
                            .as_ref()
                            .and_then(|p| Some(p.service_data.clone()))
                            .unwrap_or_default();
                        let manufacturer_data = properties
                            .as_ref()
                            .and_then(|p| Some(p.manufacturer_data.clone()))
                            .unwrap_or_default();
                        trace!("DeviceDiscovered: {:?}, {:?}, {:?}", id, rssi, properties);

                        // sender
                        //     .send(ChangedMessage::BluetoothProxyMessage(
                        //         BluetoothProxyMessage {
                        //             reason: "DeviceDiscovered".to_string(),
                        //             name: name,
                        //             mac: id.to_string(),
                        //             rssi: rssi,
                        //             service_data: service_data
                        //                 .iter()
                        //                 .map(|(k, v)| (k.to_string(), v.clone()))
                        //                 .collect(),
                        //             service_uuids: services.iter().map(|s| s.to_string()).collect(),
                        //             manufacturer_data: manufacturer_data
                        //                 .iter()
                        //                 .map(|(k, v)| (k.to_string(), v.clone()))
                        //                 .collect(),
                        //         },
                        //     ))
                        //     .unwrap();
                    }
                    CentralEvent::ServicesAdvertisement { id, services } => {
                        let peripheral = central.peripheral(&id).await?;
                        let properties = peripheral.properties().await?;
                        let rssi = properties
                            .as_ref()
                            .and_then(|p| p.rssi.clone())
                            .unwrap_or_default();

                        // let data: Vec<u8> =                            services.into_iter().map(|s| s.to_short_string()).collect();
                        debug!("ServicesAdvertisement: {:?}, {:?}", id, services);
                    }
                    CentralEvent::StateUpdate(state) => {
                        debug!("AdapterStatusUpdate {:?}", state);
                    }
                    CentralEvent::DeviceConnected(id) => {
                        debug!("DeviceConnected: {:?}", id);
                    }
                    CentralEvent::DeviceDisconnected(id) => {
                        debug!("DeviceDisconnected: {:?}", id);
                    }
                    _ => {}
                }
            }
            Ok(())
        })
    }
}
