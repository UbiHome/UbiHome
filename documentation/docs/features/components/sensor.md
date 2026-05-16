# Sensor

```yaml title="Base Example"
sensor:
  - platform: ... #(1)!
    name: "My Sensor"
    id: ram_usage
    icon: mdi:memory
    device_class: data_size
    state_class: measurement
    unit_of_measurement: "%"
    accuracy_decimals: 2
```

1. Here the [platform](../platforms/index.md) must be defined.

!!! info "Platform-specific properties"
    Each platform requires additional properties not shown above. Check the [Platforms documentation](../platforms/index.md) and the page for your chosen platform for the full list of required and optional options.

## Attributes

Common attributes are documented in [Common Component Properties](./base.md).

| Property | Description | Example |
| --- | --- | --- |
| `device_class` | Home Assistant device classification for the sensor value. | `data_size` |
| `state_class` | Home Assistant state class (for example `measurement`). | `measurement` |
| `unit_of_measurement` | Unit shown in user interfaces (for example `%`, `°C`, `Pa`). | `%` |
| `accuracy_decimals` | Number of decimals reported for the sensor value. | `2` |

## Supported Filters

| Filter | Description |
| --- | --- |
| [`round`](./filters.md#round) | Rounds numeric sensor values to the configured decimal precision. |

## Supported Platforms

| Platform | Notes |
| --- | --- |
| [Shell](../platforms/shell.md) | Reads sensor values from command output. |
| [BME280](../platforms/bme280.md) | Exposes temperature, pressure, and humidity as sensor components. |
| [Illuminance](../platforms/illuminance.md) | Exposes ambient light as a sensor component. |

For platform-specific configuration options, use the linked platform pages.

## Used in Examples

- [Monitor system resources](../../examples/system_ressources/index.md)
- [Measure Temperature, Humidity and Pressure with BME280](../../examples/measure_temperature/index.md)
- [Report your laptop's ambient light sensor](../../examples/ambient_light_sensor/index.md)

Similar to ESPHome: [Sensor](https://esphome.io/components/sensor/)
