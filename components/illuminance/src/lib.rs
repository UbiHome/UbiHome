use duration_str::deserialize_duration;
use log::{debug, error, info, warn};
use serde::Deserialize;
use serde::Deserializer;
use std::{collections::HashMap, future::Future, pin::Pin, time::Duration};
use tokio::{
    sync::broadcast::{Receiver, Sender},
    time,
};
use ubihome_core::constants::is_id_string_option;
use ubihome_core::constants::is_readable_string;
use ubihome_core::internal::sensors::UbiComponent;
use ubihome_core::template_sensor;
use ubihome_core::with_base_entity_properties;
use ubihome_core::{
    config_template, internal::sensors::UbiSensor, ChangedMessage, Module, NoConfig,
    PublishedMessage,
};

template_sensor! {
#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct AmbientLightSensorConfig {
    /// Update interval for reading light sensor values
    #[serde(default = "default_update_interval")]
    #[serde(deserialize_with = "deserialize_duration")]
    pub update_interval: Duration,
    /// Device path (Linux only) - auto-detected if not specified
    pub device_path: Option<String>,
}
}

fn default_update_interval() -> Duration {
    Duration::from_secs(30)
}

#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
struct AmbientLightConfig {}

config_template!(
    illuminance,
    AmbientLightConfig,
    NoConfig,
    NoConfig,
    AmbientLightSensorConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig
);

pub struct UbiHomePlatform {
    components: Vec<UbiComponent>,
    sensors: HashMap<String, AmbientLightSensorConfig>,
}

impl Module for UbiHomePlatform {
    fn new(config_string: &str, config_path: &str) -> Result<Self, String> {
        let config =
            ubihome_core::validation::validate_config::<CoreConfig>(config_string, config_path)?;

        debug!("AmbientLight sensor config: {:?}", config);

        let mut components: Vec<UbiComponent> = Vec::new();
        let mut sensors: HashMap<String, AmbientLightSensorConfig> = HashMap::new();

        if let Some(sensor_configs) = config.sensor {
            for (_, sensor) in sensor_configs {
                let id = sensor.get_object_id();
                components.push(UbiComponent::Sensor(UbiSensor {
                    platform: "sensor".to_string(),
                    icon: sensor
                        .icon
                        .clone()
                        .or_else(|| Some("mdi:brightness-6".to_string())),
                    device_class: sensor
                        .device_class
                        .clone()
                        .or_else(|| Some("illuminance".to_string())),
                    state_class: sensor
                        .state_class
                        .clone()
                        .or_else(|| Some("measurement".to_string())),
                    unit_of_measurement: sensor
                        .unit_of_measurement
                        .clone()
                        .or_else(|| Some("lx".to_string())),
                    accuracy_decimals: sensor.accuracy_decimals,
                    name: sensor.name.clone(),
                    id: id.clone(),
                    filters: sensor.filters.clone(),
                }));
                sensors.insert(id.clone(), sensor);
            }
        }

        Ok(UbiHomePlatform {
            // config: config.illuminance,
            components,
            sensors,
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
        let sensors = self.sensors.clone();
        Box::pin(async move {
            if sensors.is_empty() {
                debug!("No light sensors configured");
                return Ok(());
            }

            for (sensor_id, sensor_config) in sensors {
                let cloned_sender = sender.clone();

                tokio::spawn(async move {
                    let update_interval = sensor_config.update_interval;

                    let mut interval = time::interval(update_interval);

                    info!(
                        "Starting light sensor {} with update interval: {:?}",
                        sensor_id, update_interval
                    );

                    loop {
                        match read_illuminance(sensor_config.device_path.as_ref()).await {
                            Ok(illuminance) => {
                                debug!("Light sensor {} reading: {} lx", sensor_id, illuminance);

                                if let Err(e) =
                                    cloned_sender.send(ChangedMessage::SensorValueChange {
                                        key: sensor_id.clone(),
                                        value: illuminance as f32,
                                    })
                                {
                                    error!("Failed to send light sensor value: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                warn!("Failed to read light sensor {}: {}", sensor_id, e);
                            }
                        }

                        interval.tick().await;
                    }
                });
            }

            Ok(())
        })
    }
}

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux::read_illuminance;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows::read_illuminance;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos::read_illuminance;
