use duration_str::deserialize_duration;
use log::{debug, warn};
use serde::Deserialize;
use serde::Deserializer;
use std::{collections::HashMap, future::Future, pin::Pin, time::Duration};
use tokio::{
    net::{TcpStream, UdpSocket},
    sync::broadcast::{Receiver, Sender},
    time,
};
use ubihome_core::{
    config_template,
    home_assistant::sensors::UbiBinarySensor,
    internal::sensors::{InternalBinarySensor, InternalComponent},
    ChangedMessage, Module, NoConfig, PublishedMessage,
};

/// Transport protocol for a connectivity target.
#[derive(Clone, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Udp,
}

impl std::default::Default for Protocol {
    fn default() -> Self {
        Protocol::Udp
    }
}

/// A single connectivity target.
#[derive(Clone, Deserialize, Debug)]
pub struct TargetConfig {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub protocol: Protocol,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub timeout: Option<Duration>,
}

fn default_targets() -> Vec<TargetConfig> {
    vec![
        TargetConfig {
            host: "8.8.8.8".to_string(),
            port: 53,
            protocol: Protocol::Udp,
            timeout: None,
        },
        TargetConfig {
            host: "8.8.4.4".to_string(),
            port: 53,
            protocol: Protocol::Udp,
            timeout: None,
        },
        TargetConfig {
            host: "1.1.1.1".to_string(),
            port: 53,
            protocol: Protocol::Udp,
            timeout: None,
        },
        TargetConfig {
            host: "1.0.0.1".to_string(),
            port: 53,
            protocol: Protocol::Udp,
            timeout: None,
        },
    ]
}

#[derive(Clone, Deserialize, Debug)]
pub struct OnlineConfig {
    #[serde(default = "default_targets")]
    pub targets: Vec<TargetConfig>,
    #[serde(default = "default_update_interval")]
    #[serde(deserialize_with = "deserialize_duration")]
    pub update_interval: Duration,
    #[serde(default = "default_timeout")]
    #[serde(deserialize_with = "deserialize_duration")]
    pub timeout: Duration,
}

fn default_update_interval() -> Duration {
    Duration::from_secs(30)
}

fn default_timeout() -> Duration {
    Duration::from_secs(3)
}

#[derive(Clone, Deserialize, Debug)]
pub struct OnlineBinarySensorConfig {
    pub targets: Option<Vec<TargetConfig>>,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub update_interval: Option<Duration>,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub timeout: Option<Duration>,
}

config_template!(
    online,
    OnlineConfig,
    NoConfig,
    OnlineBinarySensorConfig,
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

pub struct Default {
    components: Vec<InternalComponent>,
    binary_sensors: HashMap<String, RuntimeBinarySensorConfig>,
}

impl Module for Default {
    fn new(config_string: &String) -> Result<Self, String> {
        let config = serde_yaml::from_str::<CoreConfig>(config_string)
            .map_err(|e| format!("Failed to parse online config: {}", e))?;

        let global_timeout = config.online.timeout;
        let global_targets = &config.online.targets;

        let mut components: Vec<InternalComponent> = Vec::new();
        let mut binary_sensors: HashMap<String, RuntimeBinarySensorConfig> = HashMap::new();

        for (_, any_sensor) in config.binary_sensor.clone().unwrap_or_default() {
            match any_sensor.extra {
                BinarySensorKind::online(binary_sensor) => {
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

                    let effective_timeout = binary_sensor.timeout.unwrap_or(global_timeout);
                    let source_targets = binary_sensor.targets.as_ref().unwrap_or(global_targets);

                    let targets = source_targets
                        .iter()
                        .map(|t| RuntimeTarget {
                            host: t.host.clone(),
                            port: t.port,
                            protocol: t.protocol.clone(),
                            timeout: t.timeout.unwrap_or(effective_timeout),
                        })
                        .collect();

                    binary_sensors.insert(
                        id,
                        RuntimeBinarySensorConfig {
                            targets,
                            update_interval: binary_sensor
                                .update_interval
                                .unwrap_or(config.online.update_interval),
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

/// Returns `true` if at least one target is reachable.
async fn check_any_target(targets: &[RuntimeTarget]) -> bool {
    for target in targets {
        let reachable = match target.protocol {
            Protocol::Tcp => check_tcp(&target.host, target.port, target.timeout).await,
            Protocol::Udp => check_udp_dns(&target.host, target.port, target.timeout).await,
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

/// Sends a minimal DNS query over UDP and waits for any response.
///
/// The query asks for the root zone (".") with type A. Any valid DNS response
/// (including SERVFAIL or NXDOMAIN) proves the host is reachable.
async fn check_udp_dns(host: &str, port: u16, timeout: Duration) -> bool {
    let address = format!("{}:{}", host, port);

    // Minimal DNS query packet (RFC 1035):
    //   ID=0x0001, Flags=0x0100 (standard query, RD=1),
    //   QDCOUNT=1, ANCOUNT=0, NSCOUNT=0, ARCOUNT=0,
    //   QNAME=<root> (single zero byte), QTYPE=A (1), QCLASS=IN (1)
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
            debug!("Online UDP check failed for {}", address);
            false
        }
        Err(_) => {
            warn!("Online UDP check timed out for {}", address);
            false
        }
    }
}
