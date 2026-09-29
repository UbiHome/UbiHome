use duration_str::deserialize_duration;
use log::{debug, warn};
use serde::{Deserialize, Deserializer};
use std::{
    collections::HashMap, future::Future, io::ErrorKind, net::IpAddr, pin::Pin, time::Duration,
};
use tokio::{
    sync::broadcast::{Receiver, Sender},
    task, time,
};
#[allow(unused_imports)]
use ubihome_core::constants::{is_id_string_option, is_readable_string};
use ubihome_core::internal::sensors::{UbiBinarySensor, UbiComponent, UbiSensor};
#[allow(unused_imports)]
use ubihome_core::{
    config_template, template_binary_sensor, template_sensor, with_base_entity_properties,
    ChangedMessage, Module, NoConfig, PublishedMessage,
};

#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct IcmpPingConfig {
    #[serde(default = "default_timeout")]
    #[serde(deserialize_with = "deserialize_duration")]
    #[garde(skip)]
    pub timeout: Duration,
}

template_sensor! {
    #[derive(Clone, Deserialize, Debug, Validate)]
    #[garde(allow_unvalidated)]
    pub struct IcmpPingSensorConfig {
        #[garde(skip)]
        pub ip: IpAddr,

        #[serde(default = "default_update_interval")]
        #[serde(deserialize_with = "deserialize_duration")]
        #[garde(skip)]
        pub update_interval: Duration,

        #[serde(default = "default_timeout_none")]
        #[serde(deserialize_with = "duration_str::deserialize_option_duration")]
        #[garde(skip)]
        pub timeout: Option<Duration>,
    }
}

template_binary_sensor! {
    #[derive(Clone, Deserialize, Debug, Validate)]
    #[garde(allow_unvalidated)]
    pub struct IcmpPingBinarySensorConfig {
        #[garde(skip)]
        pub ip: IpAddr,

        #[serde(default = "default_update_interval")]
        #[serde(deserialize_with = "deserialize_duration")]
        #[garde(skip)]
        pub update_interval: Duration,

        #[serde(default = "default_timeout_none")]
        #[serde(deserialize_with = "duration_str::deserialize_option_duration")]
        #[garde(skip)]
        pub timeout: Option<Duration>,
    }
}

fn default_timeout() -> Duration {
    Duration::from_secs(5)
}

fn default_timeout_none() -> Option<Duration> {
    None
}

fn default_update_interval() -> Duration {
    Duration::from_secs(60)
}

config_template!(
    icmp_ping,
    IcmpPingConfig,
    NoConfig,
    IcmpPingBinarySensorConfig,
    IcmpPingSensorConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig
);

pub struct UbiHomePlatform {
    config: IcmpPingConfig,
    components: Vec<UbiComponent>,
    sensors: HashMap<String, IcmpPingSensorConfig>,
    binary_sensors: HashMap<String, IcmpPingBinarySensorConfig>,
}

impl Module for UbiHomePlatform {
    fn new(config_string: &str, config_path: &str) -> Result<Self, String> {
        let config =
            ubihome_core::validation::validate_config::<CoreConfig>(config_string, config_path)?;

        let mut components = Vec::new();
        let mut sensors = HashMap::new();
        let mut binary_sensors = HashMap::new();

        for (_, sensor) in config.sensor.clone().unwrap_or_default() {
            let id = sensor.get_object_id();
            components.push(UbiComponent::Sensor(UbiSensor {
                platform: "sensor".to_string(),
                icon: sensor
                    .icon
                    .clone()
                    .or_else(|| Some("mdi:timer-outline".to_string())),
                device_class: sensor.device_class.clone(),
                state_class: sensor
                    .state_class
                    .clone()
                    .or_else(|| Some("measurement".to_string())),
                unit_of_measurement: sensor
                    .unit_of_measurement
                    .clone()
                    .or_else(|| Some("ms".to_string())),
                accuracy_decimals: sensor.accuracy_decimals.or(Some(2)),
                name: sensor.name.clone(),
                id: id.clone(),
                filters: sensor.filters.clone(),
            }));
            sensors.insert(id, sensor);
        }

        for (_, binary_sensor) in config.binary_sensor.clone().unwrap_or_default() {
            let id = binary_sensor.get_object_id();
            components.push(UbiComponent::BinarySensor(UbiBinarySensor {
                platform: "binary_sensor".to_string(),
                icon: binary_sensor
                    .icon
                    .clone()
                    .or_else(|| Some("mdi:lan-connect".to_string())),
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
            binary_sensors.insert(id, binary_sensor);
        }

        Ok(Self {
            config: config.icmp_ping,
            components,
            sensors,
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
        let config = self.config.clone();
        let sensors = self.sensors.clone();
        let binary_sensors = self.binary_sensors.clone();

        Box::pin(async move {
            for (key, sensor) in sensors {
                let sender = sender.clone();
                let default_timeout = config.timeout;

                tokio::spawn(async move {
                    let mut interval = time::interval(sensor.update_interval);
                    loop {
                        interval.tick().await;
                        let timeout = sensor.timeout.unwrap_or(default_timeout);
                        match ping(&sensor.ip, timeout).await {
                            Ok(latency_ms) => {
                                debug!(
                                    "ICMP ping sensor {} ({}) latency: {}ms",
                                    key, sensor.ip, latency_ms
                                );
                                _ = sender.send(ChangedMessage::SensorValueChange {
                                    key: key.clone(),
                                    value: latency_ms,
                                });
                            }
                            Err(error) => {
                                warn!("ICMP ping sensor {} ({}) failed: {}", key, sensor.ip, error);
                            }
                        }
                    }
                });
            }

            for (key, binary_sensor) in binary_sensors {
                let sender = sender.clone();
                let default_timeout = config.timeout;

                tokio::spawn(async move {
                    let mut interval = time::interval(binary_sensor.update_interval);
                    loop {
                        interval.tick().await;
                        let timeout = binary_sensor.timeout.unwrap_or(default_timeout);
                        let online = match ping(&binary_sensor.ip, timeout).await {
                            Ok(latency_ms) => {
                                debug!(
                                    "ICMP online sensor {} ({}) reachable in {}ms",
                                    key, binary_sensor.ip, latency_ms
                                );
                                true
                            }
                            Err(error) => {
                                warn!(
                                    "ICMP online sensor {} ({}) failed: {}",
                                    key, binary_sensor.ip, error
                                );
                                false
                            }
                        };

                        _ = sender.send(ChangedMessage::BinarySensorValueChange {
                            key: key.clone(),
                            value: online,
                        });
                    }
                });
            }

            Ok(())
        })
    }
}

async fn ping(ip: &IpAddr, timeout: Duration) -> Result<f32, String> {
    let ip = *ip;

    task::spawn_blocking(move || {
        let mut request = ::ping::new(ip);
        request.timeout(timeout);
        request
            .send()
            .map(|result| latency_ms(result.rtt))
            .map_err(|error| format_ping_error(error, timeout))
    })
    .await
    .map_err(|error| format!("Ping task failed: {error}"))?
}

fn latency_ms(duration: Duration) -> f32 {
    duration.as_secs_f32() * 1000.0
}

fn format_ping_error(error: ::ping::Error, timeout: Duration) -> String {
    match error {
        ::ping::Error::IoError { error } if error.kind() == ErrorKind::TimedOut => {
            format!("Ping timed out after {}ms", timeout.as_millis())
        }
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_duration_to_latency_ms() {
        assert_eq!(latency_ms(Duration::from_millis(1234)), 1234.0);
    }

    #[test]
    fn formats_timeout_errors() {
        let timeout = Duration::from_secs(2);
        let error = ::ping::Error::IoError {
            error: std::io::Error::new(ErrorKind::TimedOut, "Timeout occurred"),
        };

        assert_eq!(
            format_ping_error(error, timeout),
            "Ping timed out after 2000ms"
        );
    }

    #[test]
    fn preserves_non_timeout_ping_errors() {
        assert_eq!(
            format_ping_error(::ping::Error::InternalError, Duration::from_secs(2)),
            "internal error"
        );
    }

    #[test]
    fn parses_icmp_ping_config() {
        let config = r#"
ubihome:
  name: "Test ICMP Ping"

icmp_ping:
  timeout: 3s

sensor:
  - platform: icmp_ping
    name: "Router Latency"
    id: router_latency
    ip: 192.168.0.1
    update_interval: 15s
    timeout: 1s

binary_sensor:
  - platform: icmp_ping
    name: "Router Online"
    id: router_online
    ip: 192.168.0.1
    update_interval: 5s
"#;

        let module = UbiHomePlatform::new(config, "config.yaml").unwrap();

        assert_eq!(module.config.timeout, Duration::from_secs(3));
        assert_eq!(module.sensors.len(), 1);
        assert_eq!(module.binary_sensors.len(), 1);

        let sensor = module.sensors.get("router_latency").unwrap();
        assert_eq!(sensor.ip, "192.168.0.1".parse::<IpAddr>().unwrap());
        assert_eq!(sensor.update_interval, Duration::from_secs(15));
        assert_eq!(sensor.timeout, Some(Duration::from_secs(1)));

        let binary_sensor = module.binary_sensors.get("router_online").unwrap();
        assert_eq!(binary_sensor.ip, "192.168.0.1".parse::<IpAddr>().unwrap());
        assert_eq!(binary_sensor.update_interval, Duration::from_secs(5));
        assert_eq!(binary_sensor.timeout, None);
    }
}
