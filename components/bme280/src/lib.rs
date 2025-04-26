use duration_str::deserialize_option_duration;
use log::{debug, warn};
use oshome_core::{
    home_assistant::sensors::{Component, HASensor},
    sensor::{SensorBase, UnknownSensor},
    ChangedMessage, Module, OSHome, PublishedMessage,
};
use serde::Deserialize;
use std::{collections::HashMap, future::Future, pin::Pin, str, time::Duration};
use tokio::{
    sync::broadcast::{Receiver, Sender},
    time,
};

#[derive(Clone, Deserialize, Debug)]
pub struct BME280InternalConfig {
    pub name: Option<String>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "platform")]
#[serde(rename_all = "camelCase")]
pub enum SensorKind {
    #[serde(alias = "bme280")]
    Bme280(BME280SensorConfig),
    #[serde(untagged)]
    Unknown(UnknownSensor),
}

#[derive(Clone, Deserialize, Debug)]
pub struct SensorConfig {
    #[serde(flatten)]
    pub extra: SensorKind,
}

#[derive(Clone, Deserialize, Debug)]
pub struct BME280SensorConfig {
    pub temperature: Option<SensorBase>,
    pub pressure: Option<SensorBase>,
    pub humidity: Option<SensorBase>,
    pub address: Option<String>,
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub update_interval: Option<Duration>,
}

#[derive(Clone, Deserialize, Debug)]
struct CoreConfig {
    pub oshome: OSHome,
    pub sensor: Vec<SensorConfig>,
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum Measurement {
    Temperature,
    Pressure,
    Humidity,
}

#[derive(Clone, Debug)]
pub struct InternalSensor {
    pub entries: HashMap<Measurement, String>,
    pub address: Option<String>,
    pub update_interval: Option<Duration>,
}


#[derive(Clone, Debug)]
pub struct Default {
    config: CoreConfig,
    components: Vec<Component>,
    sensors: Vec<InternalSensor>,
}

impl Default {
    pub fn new(config_string: &String) -> Self {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();
        // info!("BME280 config: {:?}", config);
        let mut components: Vec<Component> = Vec::new();
        let mut sensors: Vec<InternalSensor> = Vec::new();

        for n_sensor in config.sensor.clone() {
            match n_sensor.extra {
                SensorKind::Bme280(sensor) => {
                    let mut sensor_entries: HashMap<Measurement, String> = HashMap::new();
                    let temperature = sensor.temperature.unwrap_or(SensorBase {
                        id: None,
                        name: format!("{} Temperature", config.oshome.name),
                        icon: None,
                        state_class: None,
                        device_class: None,
                        unit_of_measurement: None,
                    });
                    let object_id = format!("{}_{}", config.oshome.name, "temperature");
                    let id = temperature.id.unwrap_or(object_id.clone());
                    sensor_entries.insert(Measurement::Temperature, id.clone());
                    components.push(Component::Sensor(HASensor {
                        platform: "sensor".to_string(),
                        icon: Some(
                            temperature
                                .icon
                                .unwrap_or("mdi:thermometer".to_string())
                                .clone(),
                        ),
                        unique_id: Some(id.clone()),
                        device_class: temperature.device_class.clone(),
                        unit_of_measurement: Some(
                            temperature
                                .unit_of_measurement
                                .unwrap_or("°C".to_string())
                                .clone(),
                        ),
                        name: temperature.name.clone(),
                        object_id: object_id.clone(),
                    }));
                    let pressure = sensor.pressure.unwrap_or(SensorBase {
                        id: None,
                        name: format!("{} Pressure", config.oshome.name),
                        icon: None,
                        state_class: None,
                        device_class: None,
                        unit_of_measurement: None,
                    });
                    let object_id = format!("{}_{}", config.oshome.name, "pressure");
                    let id = pressure.id.unwrap_or(object_id.clone());
                    sensor_entries.insert(Measurement::Pressure, id.clone());
                    components.push(Component::Sensor(HASensor {
                        platform: "sensor".to_string(),
                        icon: Some(pressure.icon.unwrap_or("mdi:umbrella".to_string()).clone()),
                        unique_id: Some(id.clone()),
                        device_class: pressure.device_class.clone(),
                        unit_of_measurement: Some(
                            pressure
                                .unit_of_measurement
                                .unwrap_or("Pa".to_string())
                                .clone(),
                        ),
                        name: pressure.name.clone(),
                        object_id: id.clone(),
                    }));
                    let humidity = sensor.humidity.unwrap_or(SensorBase {
                        id: None,
                        name: format!("{} Humidity", config.oshome.name),
                        icon: None,
                        state_class: None,
                        device_class: None,
                        unit_of_measurement: None,
                    });
                    let object_id = format!("{}_{}", config.oshome.name, "humidity");
                    let id = humidity.id.unwrap_or(object_id.clone());
                    sensor_entries.insert(Measurement::Humidity, id.clone());
                    components.push(Component::Sensor(HASensor {
                        platform: "sensor".to_string(),
                        icon: Some(
                            humidity
                                .icon
                                .unwrap_or("mdi:water-percent".to_string())
                                .clone(),
                        ),
                        unique_id: Some(id.clone()),
                        device_class: humidity.device_class.clone(),
                        unit_of_measurement: Some(
                            humidity
                                .unit_of_measurement
                                .unwrap_or("%".to_string())
                                .clone(),
                        ),
                        name: humidity.name.clone(),
                        object_id: id.clone(),
                    }));
                    let sensor_entry = InternalSensor {
                        address: sensor.address.clone(),
                        update_interval: sensor.update_interval.clone(),
                        entries: sensor_entries,
                    };
                    sensors.push(sensor_entry);
                }
                _ => {}
            }
        }
        Default {
            config,
            components,
            sensors,
        }
    }
}

impl Module for Default {
    fn validate(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn init(&mut self) -> Result<Vec<Component>, String> {
        Ok(self.components.clone())
    }

    fn run(
        &self,
        sender: Sender<ChangedMessage>,
        _: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        // let mqtt_config = self.mqtt_config.clone();
        // let config = self.config.clone();
        let sensors = self.sensors.clone();
        let c_sender = sender.clone();
        Box::pin(async move {
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            {
                warn!("BME280 is not supported on this platform.");
                return Ok(());
            }
            #[cfg(target_os = "linux")]
            {
                use bme280::i2c::BME280;
                use linux_embedded_hal::{Delay, I2cdev};

                let result = I2cdev::new("/dev/i2c-1");
                match result {
                    Err(e) => {
                        warn!("Error initializing I2C: {}", e);
                        return Ok(());
                    }
                    _ => {}
                }

                for sensor in sensors {
                    // using Linux I2C Bus #1 in this example
                    let i2c_bus = I2cdev::new("/dev/i2c-1").unwrap();

                    // initialize the BME280 using the primary I2C address 0x76
                    let mut bme280 = BME280::new_primary(i2c_bus);

                    // or, initialize the BME280 using the secondary I2C address 0x77
                    // let mut bme280 = BME280::new_secondary(i2c_bus, Delay);

                    // or, initialize the BME280 using a custom I2C address
                    // let bme280_i2c_addr = 0x88;
                    // let mut bme280 = BME280::new(i2c_bus, bme280_i2c_addr, Delay);

                    // initialize the sensor
                    let mut delay = Delay;
                    bme280.init(&mut delay).unwrap();

                    let cloned_sensor = sensor.clone();
                    let cloned_sender = c_sender.clone();
                    tokio::spawn(async move {
                        let duration = cloned_sensor
                            .update_interval
                            .unwrap_or(Duration::from_secs(30));
                        let mut interval = time::interval(duration);
                        debug!(
                            "Address {:?} has update interval: {:?}",
                            cloned_sensor.address, interval
                        );
                        loop {
                            interval.tick().await;
                            // measure temperature, pressure, and humidity
                            let measurements = bme280.measure(&mut delay).unwrap();

                            for (sensor_type, id) in cloned_sensor.entries.clone() {
                                match sensor_type {
                                    Measurement::Temperature => {
                                        debug!("Temperature: {}", measurements.temperature);
                                        let msg = ChangedMessage::SensorValueChange {
                                            key: id.clone(),
                                            value: measurements.temperature.to_string(),
                                        };
                                        cloned_sender.send(msg).unwrap();
                                    }
                                    Measurement::Pressure => {
                                        debug!("Pressure: {}", measurements.pressure);
                                        let msg = ChangedMessage::SensorValueChange {
                                            key: id.clone(),
                                            value: measurements.pressure.to_string(),
                                        };
                                        cloned_sender.send(msg).unwrap();
                                    }
                                    Measurement::Humidity => {
                                        debug!("Humidity: {}", measurements.humidity);
                                        let msg = ChangedMessage::SensorValueChange {
                                            key: id.clone(),
                                            value: measurements.humidity.to_string(),
                                        };
                                        cloned_sender.send(msg).unwrap();
                                    }
                                }
                            }
                        }
                    });
                }
            }
            Ok(())
        })
    }
}
