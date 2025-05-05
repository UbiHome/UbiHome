use duration_str::deserialize_option_duration;
use log::{debug, warn};
use ubihome_core::{
    home_assistant::sensors::{Component, UbiSensor}, internal::sensors::{InternalComponent, InternalSensor}, sensor::{SensorBase, UnknownSensor}, ChangedMessage, Module, PublishedMessage, UbiHome
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
    pub ubihome: UbiHome,
    pub sensor: Vec<SensorConfig>,
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum Measurement {
    Temperature,
    Pressure,
    Humidity,
}

#[derive(Clone, Debug)]
pub struct BME280Sensor {
    pub entries: HashMap<Measurement, String>,
    pub address: Option<String>,
    pub update_interval: Option<Duration>,
}

#[derive(Clone, Debug)]
pub struct Default {
    // config: CoreConfig,
    components: Vec<InternalComponent>,
    sensors: Vec<BME280Sensor>,
}

impl Default {
    pub fn new(config_string: &String) -> Self {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();
        // info!("BME280 config: {:?}", config);
        let mut components: Vec<InternalComponent> = Vec::new();
        let mut sensors: Vec<BME280Sensor> = Vec::new();

        for n_sensor in config.sensor.clone() {
            match n_sensor.extra {
                SensorKind::Bme280(sensor) => {
                    let mut sensor_entries: HashMap<Measurement, String> = HashMap::new();
                    let temperature = sensor.temperature.clone().unwrap_or(SensorBase {
                        id: None,
                        name: "Temperature".to_string(),
                        icon: None,
                        state_class: None,
                        device_class: None,
                        unit_of_measurement: None,
                        filters: None,
                    });
                    let object_id = temperature.get_object_id();
                    let id = temperature.id.clone().unwrap_or(object_id.clone());
                    sensor_entries.insert(Measurement::Temperature, id.clone());
                    components.push(InternalComponent::Sensor(InternalSensor {
                        ha: UbiSensor {
                        platform: "sensor".to_string(),
                        icon: Some(
                            temperature
                                .icon
                                .clone()
                                .unwrap_or("mdi:thermometer".to_string())
                                .clone(),
                        ),
                        state_class: Some(
                            temperature
                                .state_class
                                .clone()
                                .unwrap_or("measurement".to_string()),
                        ),
                        device_class: Some(
                            temperature
                                .device_class
                                .clone()
                                .unwrap_or("temperature".to_string()),
                        ),
                        unit_of_measurement: Some(
                            temperature
                                .unit_of_measurement
                                .clone()
                                .unwrap_or("°C".to_string()),
                        ),
                        name: temperature.name.clone(),
                        id: object_id.clone(),
                    }, base: temperature }));
                    let pressure = sensor.pressure.clone().unwrap_or(SensorBase {
                        id: None,
                        name: "Pressure".to_string(),
                        icon: None,
                        state_class: None,
                        device_class: None,
                        unit_of_measurement: None,
                        filters: None,
                    });
                    let object_id = pressure.get_object_id();
                    let id = pressure.id.clone().unwrap_or(object_id.clone());
                    sensor_entries.insert(Measurement::Pressure, id.clone());
                    components.push(InternalComponent::Sensor(InternalSensor {
                        ha: UbiSensor {
                        platform: "sensor".to_string(),
                        icon: Some(pressure.icon.clone().unwrap_or("mdi:umbrella".to_string()).clone()),
                        state_class: Some(
                            pressure
                                .state_class
                                .clone()
                                .unwrap_or("measurement".to_string()),
                        ),
                        device_class: Some(
                            pressure
                                .device_class
                                .clone()
                                .unwrap_or("pressure".to_string()),
                        ),
                        unit_of_measurement: Some(
                            pressure
                                .unit_of_measurement
                                .clone()
                                .unwrap_or("Pa".to_string())
                        ),

                        name: pressure.name.clone(),
                        id: id.clone(),
                    }, base: pressure}));
                    let humidity = sensor.humidity.clone().unwrap_or(SensorBase {
                        id: None,
                        name: "Humidity".to_string(),
                        icon: None,
                        state_class: None,
                        device_class: None,
                        unit_of_measurement: None,
                        filters: None,
                    });
                    let object_id = humidity.get_object_id();
                    let id = humidity.id.clone().unwrap_or(object_id.clone());
                    sensor_entries.insert(Measurement::Humidity, id.clone());
                    components.push(InternalComponent::Sensor(InternalSensor {
                        ha: UbiSensor {
                        platform: "sensor".to_string(),
                        icon: Some(
                            humidity
                                .icon
                                .clone()
                                .unwrap_or("mdi:water-percent".to_string())
                        ),
                        state_class: Some(
                            humidity
                                .state_class
                                .clone()
                                .unwrap_or("measurement".to_string()),
                        ),
                        device_class: Some(
                            humidity
                                .device_class
                                .clone()
                                .unwrap_or("humidity".to_string()),
                        ),
                        unit_of_measurement: Some(
                            humidity
                                .unit_of_measurement
                                .clone()
                                .unwrap_or("%".to_string())
                        ),
                        name: humidity.name.clone(),
                        id: id.clone(),
                    }, base: humidity}));
                    let sensor_entry = BME280Sensor {
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
            // config,
            components,
            sensors,
        }
    }
}

impl Module for Default {
    fn validate(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn init(&mut self) -> Result<Vec<InternalComponent>, String> {
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
                            // measure temperature, pressure, and humidity
                            let measurements = bme280.measure(&mut delay).unwrap();

                            for (sensor_type, id) in cloned_sensor.entries.clone() {
                                match sensor_type {
                                    Measurement::Temperature => {
                                        debug!("Temperature: {}", measurements.temperature);
                                        let msg = ChangedMessage::SensorValueChange {
                                            key: id.clone(),
                                            value: measurements.temperature,
                                        };
                                        cloned_sender.send(msg).unwrap();
                                    }
                                    Measurement::Pressure => {
                                        debug!("Pressure: {}", measurements.pressure);
                                        let msg = ChangedMessage::SensorValueChange {
                                            key: id.clone(),
                                            value: measurements.pressure,
                                        };
                                        cloned_sender.send(msg).unwrap();
                                    }
                                    Measurement::Humidity => {
                                        debug!("Humidity: {}", measurements.humidity);
                                        let msg = ChangedMessage::SensorValueChange {
                                            key: id.clone(),
                                            value: measurements.humidity,
                                        };
                                        cloned_sender.send(msg).unwrap();
                                    }
                                }
                            }

                            // wait for the next interval
                            interval.tick().await;
                        }
                    });
                }
            }
            Ok(())
        })
    }
}
