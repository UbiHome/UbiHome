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

## Attributes

Common attributes are documented in [Common Component Properties](./base.md).

| Property | Description |
| --- | --- |
| `device_class` | Home Assistant device classification for the sensor value. |
| `state_class` | Home Assistant state class (for example `measurement`). |
| `unit_of_measurement` | Unit shown in user interfaces (for example `%`, `°C`, `Pa`). |
| `accuracy_decimals` | Number of decimals reported for the sensor value. |

## Supported Filters

| Filter | Description |
| --- | --- |
| `round` | Rounds numeric sensor values to the configured decimal precision. |

## Supported Platforms

| Platform | Notes |
| --- | --- |
| [Shell](../platforms/shell.md) | Reads sensor values from command output. |
| [BME280](../platforms/bme280.md) | Exposes temperature, pressure, and humidity as sensor components. |
| [Illuminance](../platforms/illuminance.md) | Exposes ambient light as a sensor component. |

For platform-specific configuration options, use the linked platform pages.

Similar to ESPHome: [Sensor](https://esphome.io/components/sensor/)
