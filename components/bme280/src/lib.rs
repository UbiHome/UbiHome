use log::warn;
use oshome_core::{button::UnknownButton, config_template, home_assistant::sensors::{Component, HASensor}, sensor::SensorBase, template_mapper, template_sensor, ChangedMessage, Module, NoConfig, OSHome, PublishedMessage};
use std::{collections::HashMap, future::Future, pin::Pin, str, time::Duration};
use tokio::sync::broadcast::{Receiver, Sender};
use serde::{Deserialize, Deserializer};
use duration_str::deserialize_option_duration;


#[derive(Clone, Deserialize, Debug)]
pub struct BME280InternalConfig {
    pub name: Option<String>
}

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "platform")]
#[serde(rename_all = "camelCase")]
pub enum ButtonKind {
    bme280(BME280SensorConfig),
    #[serde(untagged)]
    Unknown(UnknownButton),
}

#[derive(Clone, Deserialize, Debug)]
pub struct ButtonConfig {
    #[serde(flatten)]
    pub extra: ButtonKind,
}

#[derive(Clone, Deserialize, Debug)]
pub struct BME280SensorConfig {
    // pub platform: String,
    pub temperature: SensorBase,
    pub pressure: SensorBase,
    pub humidity: SensorBase,
}

// template_sensor!(bme280, BME280SensorConfig);

#[derive(Clone, Deserialize, Debug)]
struct CoreConfig {
    pub oshome: OSHome,
    pub bme280: Option<BME280InternalConfig>,

    pub sensor: Vec<ButtonConfig>,
}

// config_template!(bme280, Option<NoConfig>, NoConfig, NoConfig, BME280SensorConfig);


#[derive(Clone, Debug)]
pub struct Default{
    config: CoreConfig,

} 

impl Default {
    pub fn new(config_string: &String) -> Self {
        let core_config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();

        Default {
            config: core_config,
        }
    }
}

impl Module for Default {
    fn validate(&mut self) -> Result<(), String> {
        Ok(())
    }



    fn init(&mut self) -> Result<Vec<Component>, String> {
        let mut components: Vec<Component> = Vec::new();

        // if let Some(sensors) = self.config.sensor.clone() {
            for (n_sensor) in self.config.sensor.clone() {
                match n_sensor.extra {
                    ButtonKind::bme280(sensor) => {
                        let id = format!("{}_{}_{}", self.config.oshome.name, sensor.temperature.name, "temperature");
                        components.push(
                            Component::Sensor(
                                HASensor {
                                    platform: "sensor".to_string(),
                                    icon: sensor.temperature.icon.clone(),
                                    unique_id: id.clone(),
                                    device_class: sensor.temperature.device_class.clone(),
                                    unit_of_measurement: Some("°C".to_string()), //sensor.temperature.unit_of_measurement.clone(),
                                    name: sensor.temperature.name.clone(),
                                    object_id: id.clone(),
                                }
                            )
                        );
                        let id = format!("{}_{}_{}", self.config.oshome.name, sensor.pressure.name, "pressure");
                        components.push(
                            Component::Sensor(
                                HASensor {
                                    platform: "sensor".to_string(),
                                    icon: sensor.pressure.icon.clone(),
                                    unique_id: id.clone(),
                                    device_class: sensor.pressure.device_class.clone(),
                                    unit_of_measurement: Some("Pa".to_string()), //sensor.pressure.unit_of_measurement.clone(),
                                    name: sensor.pressure.name.clone(),
                                    object_id: id.clone(),
                                }
                            )
                        );
                        let id = format!("{}_{}_{}", self.config.oshome.name, sensor.humidity.name, "humidity");
                        components.push(
                            Component::Sensor(
                                HASensor {
                                    platform: "sensor".to_string(),
                                    icon: sensor.humidity.icon.clone(),
                                    unique_id: id.clone(),
                                    device_class: sensor.humidity.device_class.clone(),
                                    unit_of_measurement: Some("%".to_string()), //sensor.humidity.unit_of_measurement.clone(),
                                    name: sensor.humidity.name.clone(),
                                    object_id: id.clone(),
                                }
                            )
                        );
                    },
                    _ => {}
                }
            // }
        }
        Ok(components)
    }

    fn run(
        &self,
        sender: Sender<ChangedMessage>,
        _: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        // let mqtt_config = self.mqtt_config.clone();
        // let config = config.clone();
        Box::pin(async move {
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            {
                panic!("GPIO is not supported.");
            }
            #[cfg(target_os = "linux")]
            {
                use linux_embedded_hal::{Delay, I2cdev};
                use bme280::i2c::BME280;

                
                let result = I2cdev::new("/dev/i2c-1");
                match result {
                    Err(e) => {
                        warn!("Error initializing I2C: {}", e);
                        return Ok(())
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

                println!("Relative Humidity = {}%", measurements.humidity);
                println!("Temperature = {} deg C", measurements.temperature);
                println!("Pressure = {} pascals", measurements.pressure);

                let msg = ChangedMessage::SensorValueChange {
                    key: "bme280.temperature".to_string(),
                    value: measurements.temperature.to_string(),
                };

                sender.send(msg).unwrap();


                // Handle Button Presses
                // let cloned_config = config.clone();
                // let cloned_shell_config = shell_config.clone();
            }



            Ok(())
        })
    }

} 
