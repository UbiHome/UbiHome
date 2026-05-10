use duration_str::deserialize_duration;
use log::{debug, warn};
use serde::Deserialize;
use serde::Deserializer;
use std::{collections::HashMap, future::Future, pin::Pin, time::Duration};
use tokio::{
    net::TcpStream,
    sync::broadcast::{Receiver, Sender},
    time,
};
use ubihome_core::{
    config_template,
    home_assistant::sensors::UbiBinarySensor,
    internal::sensors::{InternalBinarySensor, InternalComponent},
    ChangedMessage, Module, NoConfig, PublishedMessage,
};

#[derive(Clone, Deserialize, Debug)]
pub struct InternetConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_update_interval")]
    #[serde(deserialize_with = "deserialize_duration")]
    pub update_interval: Duration,
    #[serde(default = "default_timeout")]
    #[serde(deserialize_with = "deserialize_duration")]
    pub timeout: Duration,
}

fn default_host() -> String {
    "8.8.8.8".to_string()
}

fn default_port() -> u16 {
    53
}

fn default_update_interval() -> Duration {
    Duration::from_secs(30)
}

fn default_timeout() -> Duration {
    Duration::from_secs(3)
}

#[derive(Clone, Deserialize, Debug)]
pub struct InternetBinarySensorConfig {
    pub host: Option<String>,
    pub port: Option<u16>,

    #[serde(default = "default_timeout_none")]
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub update_interval: Option<Duration>,

    #[serde(default = "default_timeout_none")]
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub timeout: Option<Duration>,
}

fn default_timeout_none() -> Option<Duration> {
    None
}

config_template!(
    internet,
    InternetConfig,
    NoConfig,
    InternetBinarySensorConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig
);

#[derive(Clone, Debug)]
struct RuntimeBinarySensorConfig {
    host: String,
    port: u16,
    update_interval: Duration,
    timeout: Duration,
}

pub struct Default {
    components: Vec<InternalComponent>,
    binary_sensors: HashMap<String, RuntimeBinarySensorConfig>,
}

impl Module for Default {
    fn new(config_string: &String) -> Result<Self, String> {
        let config = serde_yaml::from_str::<CoreConfig>(config_string)
            .map_err(|e| format!("Failed to parse internet config: {}", e))?;

        let mut components: Vec<InternalComponent> = Vec::new();
        let mut binary_sensors: HashMap<String, RuntimeBinarySensorConfig> = HashMap::new();

        for (_, any_sensor) in config.binary_sensor.clone().unwrap_or_default() {
            match any_sensor.extra {
                BinarySensorKind::internet(binary_sensor) => {
                    let id = any_sensor.default.get_object_id();

                    components.push(InternalComponent::BinarySensor(InternalBinarySensor {
                        ha: UbiBinarySensor {
                            platform: "sensor".to_string(),
                            icon: any_sensor
                                .default
                                .icon
                                .clone()
                                .or_else(|| Some("mdi:web".to_string())),
                            device_class: any_sensor
                                .default
                                .device_class
                                .clone()
                                .or_else(|| Some("connectivity".to_string())),
                            name: any_sensor.default.name.clone(),
                            id: id.clone(),
                        },
                        base: any_sensor.default.clone(),
                    }));

                    binary_sensors.insert(
                        id,
                        RuntimeBinarySensorConfig {
                            host: binary_sensor
                                .host
                                .unwrap_or_else(|| config.internet.host.clone()),
                            port: binary_sensor.port.unwrap_or(config.internet.port),
                            update_interval: binary_sensor
                                .update_interval
                                .unwrap_or(config.internet.update_interval),
                            timeout: binary_sensor.timeout.unwrap_or(config.internet.timeout),
                        },
                    );
                }
                _ => {}
            }
        }

        Ok(Default {
            components,
            binary_sensors,
        })
    }

    fn components(&mut self) -> Vec<InternalComponent> {
        self.components.clone()
    }

    fn run(
        &self,
        sender: Sender<ChangedMessage>,
        _receiver: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let binary_sensors = self.binary_sensors.clone();

        Box::pin(async move {
            if binary_sensors.is_empty() {
                debug!("No internet binary sensors configured");
            }

            for (key, binary_sensor) in binary_sensors {
                let mut interval = time::interval(binary_sensor.update_interval);
                let cloned_sender = sender.clone();

                tokio::spawn(async move {
                    loop {
                        let connected = check_connection(
                            &binary_sensor.host,
                            binary_sensor.port,
                            binary_sensor.timeout,
                        )
                        .await;

                        _ = cloned_sender.send(ChangedMessage::BinarySensorValueChange {
                            key: key.clone(),
                            value: connected,
                        });

                        interval.tick().await;
                    }
                });
            }

            Ok(())
        })
    }
}

async fn check_connection(host: &str, port: u16, timeout: Duration) -> bool {
    let address = format!("{}:{}", host, port);

    match time::timeout(timeout, TcpStream::connect(&address)).await {
        Ok(Ok(_)) => true,
        Ok(Err(e)) => {
            debug!("Internet check failed for {}: {}", address, e);
            false
        }
        Err(_) => {
            warn!("Internet check timed out for {}", address);
            false
        }
    }
}
