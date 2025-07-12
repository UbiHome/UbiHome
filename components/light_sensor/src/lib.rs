use duration_str::deserialize_option_duration;
use log::{debug, error, info, warn};
use serde::Deserialize;
use std::{collections::HashMap, future::Future, pin::Pin, time::Duration};
use tokio::{
    sync::broadcast::{Receiver, Sender},
    time,
};
use ubihome_core::{
    home_assistant::sensors::{UbiSensor},
    internal::sensors::{InternalComponent, InternalSensor},
    sensor::{SensorBase, UnknownSensor},
    ChangedMessage, Module, PublishedMessage, UbiHome,
};

#[derive(Clone, Deserialize, Debug)]
pub struct LightSensorConfig {
    pub name: Option<String>,
    /// Update interval for reading light sensor values
    #[serde(default = "default_update_interval")]
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub update_interval: Option<Duration>,
    /// Device path (Linux only) - auto-detected if not specified
    pub device_path: Option<String>,
}

fn default_update_interval() -> Option<Duration> {
    Some(Duration::from_secs(30))
}

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "platform")]
#[serde(rename_all = "camelCase")]
pub enum SensorKind {
    #[serde(alias = "light_sensor")]
    LightSensor(LightSensorInternalConfig),
    #[serde(untagged)]
    Unknown(UnknownSensor),
}

#[derive(Clone, Deserialize, Debug)]
pub struct SensorConfig {
    #[serde(flatten)]
    pub default: SensorBase,
    #[serde(flatten)]
    pub extra: SensorKind,
}

#[derive(Clone, Deserialize, Debug)]
pub struct LightSensorInternalConfig {
    /// Update interval for reading light sensor values
    #[serde(default = "default_update_interval")]
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub update_interval: Option<Duration>,
    /// Device path (Linux only) - auto-detected if not specified
    pub device_path: Option<String>,
}

#[derive(Clone, Deserialize, Debug)]
struct CoreConfig {
    pub ubihome: UbiHome,
    #[serde(default)]
    pub light_sensor: Option<LightSensorConfig>,
    #[serde(default)]
    pub sensor: Option<Vec<SensorConfig>>,
}

pub struct Default {
    config: LightSensorConfig,
    components: Vec<InternalComponent>,
    sensors: HashMap<String, LightSensorInternalConfig>,
}

impl Module for Default {
    fn new(config_string: &String) -> Result<Self, String> {
        let config = serde_yaml::from_str::<CoreConfig>(config_string)
            .map_err(|e| format!("Failed to parse light sensor config: {}", e))?;
        
        debug!("Light sensor config: {:?}", config);
        
        let mut components: Vec<InternalComponent> = Vec::new();
        let mut sensors: HashMap<String, LightSensorInternalConfig> = HashMap::new();

        if let Some(sensor_configs) = config.sensor {
            for sensor_config in sensor_configs {
                match sensor_config.extra {
                    SensorKind::LightSensor(sensor) => {
                        let id = sensor_config.default.get_object_id();
                        components.push(InternalComponent::Sensor(InternalSensor {
                            ha: UbiSensor {
                                platform: "sensor".to_string(),
                                icon: sensor_config.default.icon.clone()
                                    .or_else(|| Some("mdi:brightness-6".to_string())),
                                device_class: sensor_config.default.device_class.clone()
                                    .or_else(|| Some("illuminance".to_string())),
                                state_class: sensor_config.default.state_class.clone()
                                    .or_else(|| Some("measurement".to_string())),
                                unit_of_measurement: sensor_config.default.unit_of_measurement.clone()
                                    .or_else(|| Some("lx".to_string())),
                                name: sensor_config.default.name.clone(),
                                id: id.clone(),
                            },
                            base: sensor_config.default.clone(),
                        }));
                        sensors.insert(id.clone(), sensor);
                    }
                    _ => {}
                }
            }
        }

        Ok(Default {
            config: config.light_sensor.unwrap_or_default(),
            components,
            sensors,
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
        let sensors = self.sensors.clone();
        let global_config = self.config.clone();
        
        Box::pin(async move {
            if sensors.is_empty() {
                debug!("No light sensors configured");
                return Ok(());
            }

            for (sensor_id, sensor_config) in sensors {
                let cloned_sender = sender.clone();
                let cloned_global_config = global_config.clone();
                
                tokio::spawn(async move {
                    let update_interval = sensor_config.update_interval
                        .or(cloned_global_config.update_interval)
                        .unwrap_or(Duration::from_secs(30));
                    
                    let mut interval = time::interval(update_interval);
                    
                    info!("Starting light sensor {} with update interval: {:?}", 
                          sensor_id, update_interval);

                    loop {
                        match read_light_sensor(sensor_config.device_path.as_ref()).await {
                            Ok(illuminance) => {
                                debug!("Light sensor {} reading: {} lx", sensor_id, illuminance);
                                
                                if let Err(e) = cloned_sender.send(ChangedMessage::SensorValueChange {
                                    key: sensor_id.clone(),
                                    value: illuminance as f32,
                                }) {
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

impl std::default::Default for LightSensorConfig {
    fn default() -> Self {
        Self {
            name: None,
            update_interval: default_update_interval(),
            device_path: None,
        }
    }
}

#[cfg(target_os = "linux")]
async fn read_light_sensor(device_path: Option<&String>) -> Result<f64, String> {
    use std::{fs, path::Path};
    
    // If a specific device path is provided, use it
    if let Some(path) = device_path {
        return read_linux_light_sensor_from_path(path).await;
    }
    
    // Auto-detect light sensor devices
    let iio_base = "/sys/bus/iio/devices";
    if Path::new(iio_base).exists() {
        let entries = fs::read_dir(iio_base)
            .map_err(|e| format!("Failed to read IIO devices directory: {}", e))?;
            
        for entry in entries {
            if let Ok(entry) = entry {
                let device_path = entry.path();
                if let Some(device_name) = device_path.file_name() {
                    if device_name.to_string_lossy().starts_with("iio:device") {
                        // Check if this device has illuminance capabilities
                        let illuminance_raw_path = device_path.join("in_illuminance_raw");
                        let illuminance_input_path = device_path.join("in_illuminance_input");
                        
                        if illuminance_raw_path.exists() {
                            if let Ok(value) = read_linux_light_sensor_from_path(
                                &illuminance_raw_path.to_string_lossy().to_string()
                            ).await {
                                return Ok(value);
                            }
                        } else if illuminance_input_path.exists() {
                            if let Ok(value) = read_linux_light_sensor_from_path(
                                &illuminance_input_path.to_string_lossy().to_string()
                            ).await {
                                return Ok(value);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Try alternative paths for some hardware
    let hwmon_paths = [
        "/sys/class/hwmon/hwmon0/device/als",
        "/sys/class/hwmon/hwmon1/device/als",
        "/sys/devices/platform/applesmc.768/light",
    ];
    
    for path in &hwmon_paths {
        if Path::new(path).exists() {
            if let Ok(value) = read_linux_light_sensor_from_path(&path.to_string()).await {
                return Ok(value);
            }
        }
    }
    
    Err("No light sensor found on this system".to_string())
}

#[cfg(target_os = "linux")]
async fn read_linux_light_sensor_from_path(path: &str) -> Result<f64, String> {
    use std::fs;
    
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read light sensor from {}: {}", path, e))?;
    
    let value: f64 = content.trim().parse()
        .map_err(|e| format!("Failed to parse light sensor value '{}': {}", content.trim(), e))?;
    
    Ok(value)
}

#[cfg(target_os = "windows")]
async fn read_light_sensor(_device_path: Option<&String>) -> Result<f64, String> {
    // For now, return a simulated value to demonstrate functionality
    // In a production environment, you would implement actual Windows sensor API calls
    use std::process::Command;
    
    // Try to use PowerShell to get sensor information
    // This is a fallback approach that users can customize
    let output = Command::new("powershell")
        .args(&[
            "-Command",
            "try { (Get-WmiObject -Class Win32_LightSensor -ErrorAction Stop | Select-Object -First 1).CurrentReading } catch { 'No sensor found' }"
        ])
        .output();
        
    match output {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Ok(value) = output_str.parse::<f64>() {
                Ok(value)
            } else {
                // Return a default message suggesting shell platform usage
                Err(format!(
                    "Windows light sensor not available or accessible. Consider using the shell platform with a custom PowerShell command. Output: {}",
                    output_str
                ))
            }
        }
        Err(e) => Err(format!(
            "Failed to execute PowerShell command to read light sensor: {}. Consider using the shell platform instead.",
            e
        ))
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
async fn read_light_sensor(_device_path: Option<&String>) -> Result<f64, String> {
    Err("Light sensor is only supported on Linux and Windows".to_string())
}