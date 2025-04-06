use log::debug;
use oshome_core::{binary_sensor::BinarySensorKind, sensor::SensorKind, CoreConfig, Message};
use serde::Deserialize;
use std::{thread, str, time::Duration};
use tokio::{
    sync::broadcast::{Receiver, Sender},
    time,
};

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GpioDevice {
    RaspberryPi,
}

#[derive(Clone, Deserialize, Debug)]
pub struct GpioConfig {
    pub device: GpioDevice,
}

pub async fn start(
    sender: Sender<Option<Message>>,
    mut receiver: Receiver<Option<Message>>,
    config: &CoreConfig,
    shell_config: &GpioConfig,
) {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        panic!("GPIO is not supported on macOS.");
    }
    #[cfg(target_os = "linux")]
    {
        use linux_embedded_hal::{Delay, I2cdev};
        use bme280::i2c::BME280;

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


        // Handle Button Presses
        let cloned_config = config.clone();
        let cloned_shell_config = shell_config.clone();
    }
}