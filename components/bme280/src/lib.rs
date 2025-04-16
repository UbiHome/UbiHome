use log::{debug, warn};
use oshome_core::{
    button::UnknownButton,
    home_assistant::sensors::{Component, HASensor},
    sensor::SensorBase,
    ChangedMessage, Module, OSHome, PublishedMessage,
};
use serde::Deserialize;
use std::{collections::HashMap, future::Future, pin::Pin, str};
use tokio::sync::broadcast::{Receiver, Sender};

#[derive(Clone, Deserialize, Debug)]
pub struct BME280InternalConfig {
    pub name: Option<String>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "platform")]
#[serde(rename_all = "camelCase")]
pub enum SensorKind {
    Bme280(BME280SensorConfig),
    #[serde(untagged)]
    Unknown(UnknownButton),
}

#[derive(Clone, Deserialize, Debug)]
pub struct ButtonConfig {
    #[serde(flatten)]
    pub extra: SensorKind,
}

#[derive(Clone, Deserialize, Debug)]
pub struct BME280SensorConfig {
    // pub platform: String,
    pub temperature: Option<SensorBase>,
    pub pressure: Option<SensorBase>,
    pub humidity: Option<SensorBase>,
}

// template_sensor!(bme280, BME280SensorConfig);

#[derive(Clone, Deserialize, Debug)]
struct CoreConfig {
    pub oshome: OSHome,
    // pub bme280: Option<BME280InternalConfig>,
    pub sensor: Vec<ButtonConfig>,
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum Measurement {
    Temperature,
    Pressure,
    Humidity,
}

// config_template!(bme280, Option<NoConfig>, NoConfig, NoConfig, BME280SensorConfig);

#[derive(Clone, Debug)]
pub struct Default {
    config: CoreConfig,
    components: Vec<Component>,
    sensors: Vec<HashMap<Measurement, String>>
}

impl Default {
    pub fn new(config_string: &String) -> Self {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();
        let mut components: Vec<Component> = Vec::new();
        let mut sensors: Vec<HashMap<Measurement, String>> = Vec::new();

        for n_sensor in config.sensor.clone() {
            let mut sensor_entries: HashMap<Measurement, String> = HashMap::new();
            match n_sensor.extra {
                SensorKind::Bme280(sensor) => {
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
                }
                _ => {}
            }
            sensors.push(sensor_entries);
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
        Box::pin(async move {
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            {
                panic!("GPIO is not supported.");
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

                // measure temperature, pressure, and humidity
                let measurements = bme280.measure(&mut delay).unwrap();

                for sensor in sensors {
                    for (sensor_type, id) in sensor {
                        match sensor_type {
                            Measurement::Temperature => {
                                debug!("Temperature: {}", measurements.temperature);
                                let msg = ChangedMessage::SensorValueChange {
                                    key: id.clone(),
                                    value: measurements.temperature.to_string(),
                                };
                                sender.send(msg).unwrap();
                            }
                            Measurement::Pressure => {
                                debug!("Pressure: {}", measurements.pressure);
                                let msg = ChangedMessage::SensorValueChange {
                                    key: id.clone(),
                                    value: measurements.pressure.to_string(),
                                };
                                sender.send(msg).unwrap();
                            }
                            Measurement::Humidity => {
                                debug!("Humidity: {}", measurements.humidity);
                                let msg = ChangedMessage::SensorValueChange {
                                    key: id.clone(),
                                    value: measurements.humidity.to_string(),
                                };
                                sender.send(msg).unwrap();
                            }
                        }                        
                       
                    }
                }
            }
            Ok(())
        })
    }
}
