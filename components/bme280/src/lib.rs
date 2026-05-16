use log::debug;
use serde::{Deserialize, Deserializer};
use std::{collections::HashMap, future::Future, pin::Pin, str, time::Duration};
use tokio::sync::broadcast::{Receiver, Sender};
use ubihome_core::{
    config_template,
    constants::{is_id_string_option, is_readable_string},
    internal::sensors::{UbiComponent, UbiSensor},
    template_sensor, with_base_entity_properties, ChangedMessage, Module, NoConfig,
    PublishedMessage,
};

#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct BME280InternalConfig {
    pub name: Option<String>,
}

template_sensor! {
    #[derive(Clone, Deserialize, Debug, Validate)]
    pub struct SpecificSensorConfig {
    }
}

#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct BME280SensorConfig {
    platform: String,
    #[garde(dive)]
    pub temperature: Option<SpecificSensorConfig>,
    #[garde(dive)]
    pub pressure: Option<SpecificSensorConfig>,
    #[garde(dive)]
    pub humidity: Option<SpecificSensorConfig>,
    pub address: Option<String>,
    #[serde(deserialize_with = "deserialize_option_duration")]
    #[garde(skip)]
    pub update_interval: Option<Duration>,
}

impl BME280SensorConfig {
    pub fn get_object_id(&self) -> String {
        let name = self.address.clone().unwrap_or("".to_string());
        format!("bme280_{}", name.to_lowercase().replace(" ", "_"))
    }
    pub fn is_configured(&self) -> bool {
        true
    }
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
pub struct UbiHomePlatform {
    // config: CoreConfig,
    components: Vec<UbiComponent>,
    sensors: Vec<BME280Sensor>,
}

config_template!(
    bme280,
    BME280InternalConfig,
    NoConfig,
    NoConfig,
    BME280SensorConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig
);

impl Module for UbiHomePlatform {
    fn new(config_string: &str) -> Result<Self, String> {
        let config =
            serde_saphyr::from_str::<CoreConfig>(config_string).map_err(|e| e.to_string())?;
        // info!("BME280 config: {:?}", config);
        let mut components: Vec<UbiComponent> = Vec::new();
        let mut sensors: Vec<BME280Sensor> = Vec::new();

        for (_, n_sensor) in config.sensor.clone().unwrap_or_default() {
            let mut sensor_entries: HashMap<Measurement, String> = HashMap::new();
            let temperature = n_sensor
                .temperature
                .clone()
                .unwrap_or(SpecificSensorConfig {
                    id: None,
                    name: "Temperature".to_string(),
                    icon: None,
                    state_class: None,
                    accuracy_decimals: None,
                    device_class: None,
                    unit_of_measurement: None,
                    filters: None,
                    platform: "bme280".to_string(),
                });
            let object_id = temperature.get_object_id();
            let id = temperature.id.clone().unwrap_or(object_id.clone());
            sensor_entries.insert(Measurement::Temperature, id.clone());
            components.push(UbiComponent::Sensor(UbiSensor {
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
                accuracy_decimals: None,
                name: temperature.name.clone(),
                id: object_id.clone(),
                filters: temperature.filters.clone(),
            }));
            let pressure = n_sensor.pressure.clone().unwrap_or(SpecificSensorConfig {
                id: None,
                name: "Pressure".to_string(),
                icon: None,
                state_class: None,
                accuracy_decimals: None,
                device_class: None,
                unit_of_measurement: None,
                filters: None,
                platform: "bme280".to_string(),
            });
            let object_id = pressure.get_object_id();
            let id = pressure.id.clone().unwrap_or(object_id.clone());
            sensor_entries.insert(Measurement::Pressure, id.clone());
            components.push(UbiComponent::Sensor(UbiSensor {
                platform: "sensor".to_string(),
                icon: Some(
                    pressure
                        .icon
                        .clone()
                        .unwrap_or("mdi:umbrella".to_string())
                        .clone(),
                ),
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
                        .unwrap_or("Pa".to_string()),
                ),
                accuracy_decimals: None,
                name: pressure.name.clone(),
                id: id.clone(),
                filters: pressure.filters.clone(),
            }));
            let humidity = n_sensor.humidity.clone().unwrap_or(SpecificSensorConfig {
                id: None,
                name: "Humidity".to_string(),
                icon: None,
                state_class: None,
                accuracy_decimals: None,
                device_class: None,
                unit_of_measurement: None,
                filters: None,
                platform: "bme280".to_string(),
            });
            let object_id = humidity.get_object_id();
            let id = humidity.id.clone().unwrap_or(object_id.clone());
            sensor_entries.insert(Measurement::Humidity, id.clone());
            components.push(UbiComponent::Sensor(UbiSensor {
                platform: "sensor".to_string(),
                icon: Some(
                    humidity
                        .icon
                        .clone()
                        .unwrap_or("mdi:water-percent".to_string()),
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
                        .unwrap_or("%".to_string()),
                ),
                accuracy_decimals: None,
                name: humidity.name.clone(),
                id: id.clone(),
                filters: humidity.filters.clone(),
            }));
            let sensor_entry = BME280Sensor {
                address: n_sensor.address.clone(),
                update_interval: n_sensor.update_interval,
                entries: sensor_entries,
            };
            sensors.push(sensor_entry);
        }
        Ok(UbiHomePlatform {
            // config,
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
        _: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        // let mqtt_config = self.mqtt_config.clone();
        // let config = self.config.clone();

        #[allow(unused_variables)]
        let sensors = self.sensors.clone();
        #[allow(unused_variables)]
        let c_sender = sender.clone();
        Box::pin(async move {
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            {
                log::warn!("BME280 is not supported on this platform.");
            }
            #[cfg(target_os = "linux")]
            {
                use bme280::i2c::BME280;
                use linux_embedded_hal::{Delay, I2cdev};
                use tokio::time;

                let result = I2cdev::new("/dev/i2c-1");
                if let Err(e) = result {
                    log::warn!("Error initializing I2C: {}", e);
                    return Ok(());
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
                        log::debug!(
                            "Address {:?} has update interval: {:?}",
                            cloned_sensor.address,
                            interval
                        );
                        loop {
                            // measure temperature, pressure, and humidity
                            let measurements = bme280.measure(&mut delay).unwrap();

                            for (sensor_type, id) in cloned_sensor.entries.clone() {
                                match sensor_type {
                                    Measurement::Temperature => {
                                        log::debug!("Temperature: {}", measurements.temperature);
                                        let msg = ChangedMessage::SensorValueChange {
                                            key: id.clone(),
                                            value: measurements.temperature,
                                        };
                                        cloned_sender.send(msg).unwrap();
                                    }
                                    Measurement::Pressure => {
                                        log::debug!("Pressure: {}", measurements.pressure);
                                        let msg = ChangedMessage::SensorValueChange {
                                            key: id.clone(),
                                            value: measurements.pressure,
                                        };
                                        cloned_sender.send(msg).unwrap();
                                    }
                                    Measurement::Humidity => {
                                        log::debug!("Humidity: {}", measurements.humidity);
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
