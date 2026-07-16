use duration_str::{
    deserialize_duration, deserialize_option_duration as deserialize_optional_duration,
};
use log::{debug, warn};
use serde::{Deserialize, Deserializer};
use std::{collections::HashMap, future::Future, pin::Pin, time::Duration};
use tokio::{
    net::{TcpStream, UdpSocket},
    sync::broadcast::{Receiver, Sender},
    time,
};
use ubihome_core::constants::{is_id_string_option, is_readable_string};
use ubihome_core::internal::sensors::{UbiBinarySensor, UbiComponent};
use ubihome_core::template_binary_sensor;
use ubihome_core::with_base_entity_properties;
use ubihome_core::{config_template, ChangedMessage, Module, NoConfig, PublishedMessage};

#[derive(Clone, Default, Deserialize, Debug, PartialEq, Validate)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    #[default]
    Dns,
}

/// Standard DNS port; `dns` targets always use this.
const DNS_PORT: u16 = 53;

#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
#[serde(try_from = "TargetConfigInput")]
pub struct TargetConfig {
    pub host: String,
    #[garde(skip)]
    pub port: u16,
    pub protocol: Protocol,
    #[garde(skip)]
    pub timeout: Option<Duration>,
}

/// Raw form of [`TargetConfig`]. `port` is optional here and resolved during
/// deserialization: `tcp` requires it, `dns` defaults to [`DNS_PORT`].
#[derive(Deserialize)]
struct TargetConfigInput {
    host: String,
    #[serde(default)]
    port: Option<u16>,
    #[serde(default)]
    protocol: Protocol,
    #[serde(default, deserialize_with = "deserialize_optional_duration")]
    timeout: Option<Duration>,
}

impl TryFrom<TargetConfigInput> for TargetConfig {
    type Error = String;

    fn try_from(input: TargetConfigInput) -> Result<Self, Self::Error> {
        let port = match input.protocol {
            Protocol::Tcp => input
                .port
                .ok_or_else(|| "port is required for tcp targets".to_string())?,
            Protocol::Dns => input.port.unwrap_or(DNS_PORT),
        };
        Ok(TargetConfig {
            host: input.host,
            port,
            protocol: input.protocol,
            timeout: input.timeout,
        })
    }
}

fn default_targets() -> Vec<TargetConfig> {
    ["8.8.8.8", "8.8.4.4", "1.1.1.1", "1.0.0.1"]
        .into_iter()
        .map(|host| TargetConfig {
            host: host.to_string(),
            port: DNS_PORT,
            protocol: Protocol::Dns,
            timeout: None,
        })
        .collect()
}

#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct OnlineConfig {
    #[serde(default = "default_targets")]
    #[garde(dive)]
    pub targets: Vec<TargetConfig>,
    #[serde(default = "default_update_interval")]
    #[serde(deserialize_with = "deserialize_duration")]
    #[garde(skip)]
    pub update_interval: Duration,
    #[serde(default = "default_timeout")]
    #[serde(deserialize_with = "deserialize_duration")]
    #[garde(skip)]
    pub timeout: Duration,
}

fn default_update_interval() -> Duration {
    Duration::from_secs(30)
}

fn default_timeout() -> Duration {
    Duration::from_secs(3)
}

template_binary_sensor! {
#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct OnlineBinarySensorConfig {
}
}

config_template!(
    online,
    OnlineConfig,
    NoConfig,
    OnlineBinarySensorConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig
);

#[derive(Clone, Debug)]
struct RuntimeTarget {
    host: String,
    port: u16,
    protocol: Protocol,
    timeout: Duration,
}

#[derive(Clone, Debug)]
struct RuntimeBinarySensorConfig {
    targets: Vec<RuntimeTarget>,
    update_interval: Duration,
}

#[derive(Clone, Debug)]
pub struct UbiHomePlatform {
    components: Vec<UbiComponent>,
    binary_sensors: HashMap<String, RuntimeBinarySensorConfig>,
}

impl Module for UbiHomePlatform {
    fn new(config_string: &str, config_path: &str) -> Result<Self, String> {
        let config =
            ubihome_core::validation::validate_config::<CoreConfig>(config_string, config_path)?;

        let global_timeout = config.online.timeout;
        let global_targets = &config.online.targets;

        let mut components: Vec<UbiComponent> = Vec::new();
        let mut binary_sensors: HashMap<String, RuntimeBinarySensorConfig> = HashMap::new();

        for (_, binary_sensor) in config.binary_sensor.clone().unwrap_or_default() {
            let id = binary_sensor.get_object_id();
            components.push(UbiComponent::BinarySensor(UbiBinarySensor {
                platform: "sensor".to_string(),
                icon: binary_sensor
                    .icon
                    .clone()
                    .or_else(|| Some("mdi:web".to_string())),
                device_class: binary_sensor
                    .device_class
                    .clone()
                    .or_else(|| Some("connectivity".to_string())),
                name: binary_sensor.name.clone(),
                id: id.clone(),
                on_press: binary_sensor.on_press.clone(),
                on_release: binary_sensor.on_release.clone(),
                filters: binary_sensor.filters.clone(),
            }));

            let targets = global_targets
                .iter()
                .map(|target| RuntimeTarget {
                    host: target.host.clone(),
                    port: target.port,
                    protocol: target.protocol.clone(),
                    timeout: target.timeout.unwrap_or(global_timeout),
                })
                .collect();

            binary_sensors.insert(
                id,
                RuntimeBinarySensorConfig {
                    targets,
                    update_interval: config.online.update_interval,
                },
            );
        }

        Ok(UbiHomePlatform {
            components,
            binary_sensors,
        })
    }

    fn components(&mut self) -> Vec<UbiComponent> {
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
                debug!("No online binary sensors configured");
            }

            for (key, binary_sensor) in binary_sensors {
                let mut interval = time::interval(binary_sensor.update_interval);
                let cloned_sender = sender.clone();

                tokio::spawn(async move {
                    loop {
                        let connected = check_any_target(&binary_sensor.targets).await;

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

async fn check_any_target(targets: &[RuntimeTarget]) -> bool {
    for target in targets {
        let reachable = match target.protocol {
            Protocol::Tcp => check_tcp(&target.host, target.port, target.timeout).await,
            Protocol::Dns => check_dns(&target.host, target.port, target.timeout).await,
        };
        if reachable {
            return true;
        }
    }
    false
}

async fn check_tcp(host: &str, port: u16, timeout: Duration) -> bool {
    let address = format!("{}:{}", host, port);

    match time::timeout(timeout, TcpStream::connect(&address)).await {
        Ok(Ok(_)) => true,
        Ok(Err(e)) => {
            debug!("Online TCP check failed for {}: {}", address, e);
            false
        }
        Err(_) => {
            warn!("Online TCP check timed out for {}", address);
            false
        }
    }
}

async fn check_dns(host: &str, port: u16, timeout: Duration) -> bool {
    let address = format!("{}:{}", host, port);

    let query: [u8; 17] = [
        0x00, 0x01, // ID
        0x01, 0x00, // Flags: standard query, recursion desired
        0x00, 0x01, // QDCOUNT: 1 question
        0x00, 0x00, // ANCOUNT: 0
        0x00, 0x00, // NSCOUNT: 0
        0x00, 0x00, // ARCOUNT: 0
        0x00, // QNAME: root label (empty)
        0x00, 0x01, // QTYPE: A
        0x00, 0x01, // QCLASS: IN
    ];

    let inner = async {
        let socket = UdpSocket::bind("0.0.0.0:0").await.ok()?;
        socket.connect(&address).await.ok()?;
        socket.send(&query).await.ok()?;
        let mut buf = [0u8; 512];
        socket.recv(&mut buf).await.ok()?;
        Some(())
    };

    match time::timeout(timeout, inner).await {
        Ok(Some(())) => true,
        Ok(None) => {
            debug!("Online DNS check failed for {}", address);
            false
        }
        Err(_) => {
            warn!("Online DNS check timed out for {}", address);
            false
        }
    }
}
