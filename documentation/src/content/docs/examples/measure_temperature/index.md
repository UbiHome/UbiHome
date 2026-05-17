# Measure Temperature, Humidity and Pressure with BME280

Get the BME280 readings and make them available via the native API:

```yaml
ubihome:
  name: UbiHome Example

bme280:

sensor:
  - platform: bme280
    update_interval: 30s
    name: 'Temperature'

# Native Homeassistant API
api:
  # encryption:
  #   key:  # Generate here: https://ubihome.github.io/features/connectivity/native_api.html
```

With the following pin usage: 

BME280: 
 - Vin	-- PIN 1 / 3.3V
 - GND	-- PIN 9 / GND
 - SCL	-- PIN 5 / GPIO 3
 - SDA  -- PIN 3 / GPIO 2

> The BME Component automatically detects devices on i2c1

## Related documentation

- Component: [Sensor](../../features/components/sensor)
- Platform: [BME280](../../features/platforms/bme280)
