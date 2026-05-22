---
title: 'Sensor'
sidebar:
  order: 1
---

```yaml title="Base Example"
sensor:
  - platform: ... # [!code highlight]
    name: 'My Sensor'
    id: ram_usage
    icon: mdi:memory
    device_class: data_size
    state_class: measurement
    unit_of_measurement: '%'
    accuracy_decimals: 2
```

:::note[Platform-specific properties]
Each platform requires additional properties not shown above. Check the [Platforms documentation](/features/) and the page for your chosen platform for the full list of required and optional options.
:::

## Attributes

Common attributes are documented in [Common Component Properties](/features/components/base/).

| Property              | Description                                                  | Example       |
| --------------------- | ------------------------------------------------------------ | ------------- |
| `device_class`        | Home Assistant device classification for the sensor value.   | `data_size`   |
| `state_class`         | Home Assistant state class (for example `measurement`).      | `measurement` |
| `unit_of_measurement` | Unit shown in user interfaces (for example `%`, `°C`, `Pa`). | `%`           |
| `accuracy_decimals`   | Number of decimals reported for the sensor value.            | `2`           |

## Supported Filters

| Filter                                         | Description                                                       |
| ---------------------------------------------- | ----------------------------------------------------------------- |
| [`round`](/features/components/filters/#round) | Rounds numeric sensor values to the configured decimal precision. |

## Supported Platforms

| Platform                                        | Notes                                                             |
| ----------------------------------------------- | ----------------------------------------------------------------- |
| [Shell](/features/platforms/shell/)             | Reads sensor values from command output.                          |
| [BME280](/features/platforms/bme280/)           | Exposes temperature, pressure, and humidity as sensor components. |
| [Illuminance](/features/platforms/illuminance/) | Exposes ambient light as a sensor component.                      |

For platform-specific configuration options, use the linked platform pages.

## Used in Examples

- [Monitor system resources](/examples/system_ressources/)
- [Measure Temperature, Humidity and Pressure with BME280](/examples/measure_temperature/)
- [Report your laptop's ambient light sensor](/examples/illuminance/)

Similar to ESPHome: [Sensor](https://esphome.io/components/sensor/)
