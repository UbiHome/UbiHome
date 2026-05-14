use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use futures::stream::StreamExt;
use log::{debug, info, trace};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::{future::Future, pin::Pin, str};
use tokio::sync::broadcast::{Receiver, Sender};
use ubihome_core::{
    config_template, internal::sensors::UbiComponent, ChangedMessage, Module, PublishedMessage,
};
use ubihome_core::{BluetoothProxyMessage, NoConfig};

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().next().unwrap()
}

#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct BluetoothProxyConfig {
    pub disabled: Option<bool>,
}

config_template!(
    bluetooth_proxy,
    Option<BluetoothProxyConfig>,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig
);

#[derive(Clone, Debug)]
pub struct UbiHomePlatform {
    config: CoreConfig,
}

impl Module for UbiHomePlatform {
    fn new(config_string: &str) -> Result<Self, String> {
        let config =
            serde_saphyr::from_str::<CoreConfig>(config_string).map_err(|e| e.to_string())?;

        Ok(UbiHomePlatform { config })
    }

    fn components(&mut self) -> Vec<UbiComponent> {
        let components: Vec<UbiComponent> = Vec::new();

        components
    }

    fn run(
        &self,
        sender: Sender<ChangedMessage>,
        _: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let config = self.config.clone();
        info!(
            "Starting Bluetooth Proxy with config: {:?}",
            config.bluetooth_proxy
        );
        Box::pin(async move {
            // TODO: Check if bluetooth is enabled

            let manager = Manager::new()
                .await
                .expect("Failed to create bluetooth manager");

            // get the first bluetooth adapter
            // connect to the adapter
            let central = get_central(&manager).await;
            info!("CentralNac: {:?}", central.adapter_mac().await);

            let central_state = central.adapter_state().await.expect("No adapter found");
            info!("CentralState: {:?}", central_state);

            // Each adapter has an event stream, we fetch via events(),
            // simplifying the type, this will return what is essentially a
            // Future<Result<Stream<Item=CentralEvent>>>.
            let mut events = central.events().await?;

            // start scanning for devices
            central.start_scan(ScanFilter::default()).await?;

            while let Some(event) = events.next().await {
                if let CentralEvent::DeviceUpdated(id) = event {
                    let peripheral = central.peripheral(&id).await?;
                    let properties = peripheral.properties().await?;

                    let rssi = properties.as_ref().and_then(|p| p.rssi).unwrap_or_default();
                    let name = properties
                        .as_ref()
                        .and_then(|p| p.local_name.clone())
                        .unwrap_or_default();
                    let services = properties
                        .as_ref()
                        .map(|p| p.services.clone())
                        .unwrap_or_default();
                    let service_data = properties
                        .as_ref()
                        .map(|p| p.service_data.clone())
                        .unwrap_or_default();
                    let manufacturer_data = properties
                        .as_ref()
                        .map(|p| p.manufacturer_data.clone())
                        .unwrap_or_default();
                    let mac_address = properties.as_ref().map(|p| p.address).unwrap_or_default();

                    trace!(
                        "DeviceUpdated: {:?}, {:?}, {:?}",
                        mac_address,
                        rssi,
                        properties
                    );

                    sender
                        .send(ChangedMessage::BluetoothProxyMessage(
                            BluetoothProxyMessage {
                                reason: "DeviceUpdated".to_string(),
                                name,
                                mac: mac_address.to_string(),
                                rssi,
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
            }
            Ok(())
        })
    }
}
