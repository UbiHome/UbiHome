# Light Sensor Component

The light sensor component allows you to integrate laptop ambient light sensors into UbiHome. It supports both Linux and Windows systems with automatic sensor detection.

## Documentation

For complete documentation, configuration examples, and troubleshooting guides, please see the [light sensor platform documentation](../../documentation/docs/features/platforms/light_sensor.md).

## Quick Start

```yaml
light_sensor:
  update_interval: 30s

sensor:
  - platform: light_sensor
    name: "Ambient Light"
    device_class: illuminance
    unit_of_measurement: "lx"
```