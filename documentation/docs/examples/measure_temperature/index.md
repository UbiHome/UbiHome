# Measure Temperature, Humidity and Pressure with BME280

Get the BME280 readings and make them available via the native API:

```yaml
--8<-- "examples/measure_temperature/raspberry.yml"
```

With the following pin usage: 

BME280: 
 - Vin	-- PIN 1 / 3.3V
 - GND	-- PIN 9 / GND
 - SCL	-- PIN 5 / GPIO 3
 - SDA  -- PIN 3 / GPIO 2

> The BME Component automatically detects devices on i2c1
