---
tags:
  - Illuminance
  - Linux
  - Windows
---

# Illuminance Sensor

The illuminance sensor platform allows you to integrate a laptop ambient light sensors into UbiHome. It supports both Linux and Windows systems.

## Supported Platforms

### Linux

- Uses Industrial I/O (IIO) framework (`/sys/bus/iio/devices/`)
- Supports auto-detection of light sensor devices
- Fallback support for alternative hardware monitor paths
- Common device locations:
  - `/sys/bus/iio/devices/iio:deviceN/in_illuminance_raw`
  - `/sys/bus/iio/devices/iio:deviceN/in_illuminance_input`
  - `/sys/class/hwmon/hwmonN/device/als`
  - `/sys/devices/platform/applesmc.768/light` (Apple devices)

### Windows

- Native Windows Sensor API support using COM interfaces
- Automatic detection of ambient light sensors through Windows Sensor Manager
- Direct sensor data access without external dependencies

## Configuration

Enable the platform:

```yaml
illuminance:
  update_interval: 30s # Default update interval for all sensors
```

### Basic Usage

```yaml
sensor:
  - platform: illuminance
    name: 'Ambient Light'
    icon: mdi:brightness-6
    unit_of_measurement: 'lx'
    device_class: illuminance
    state_class: measurement
```

### Advanced Configuration with Manual Device Path

```yaml
sensor:
  - platform: illuminance
    name: 'Ambient Light Sensor'
    device_path: '/sys/bus/iio/devices/iio:device0/in_illuminance_raw'
    update_interval: 15s
    icon: mdi:brightness-6
    unit_of_measurement: 'lx'
    device_class: illuminance
    state_class: measurement
```

## Configuration Options

### Light Sensor Platform

| Property              | Type     | Default            | Description                     |
| --------------------- | -------- | ------------------ | ------------------------------- |
| `platform`            | string   | **Required**       | Must be `illuminance`           |
| `name`                | string   | **Required**       | Display name for the sensor     |
| `device_path`         | string   | Optional           | Manual device path (Linux only) |
| `update_interval`     | duration | 30s                | How often to read the sensor    |
| `icon`                | string   | `mdi:brightness-6` | Home Assistant icon             |
| `device_class`        | string   | `illuminance`      | Home Assistant device class     |
| `state_class`         | string   | `measurement`      | Home Assistant state class      |
| `unit_of_measurement` | string   | `lx`               | Unit of measurement             |

### Global Configuration

```yaml
illuminance:
  update_interval: 30s # Default update interval for all sensors
  device_path: '/custom/path' # Default device path (Linux only)
```

## Troubleshooting

### Linux

1. **No sensor detected**: Check if your system has a light sensor:

   ```bash
   ls /sys/bus/iio/devices/
   find /sys -name "*illuminance*" 2>/dev/null
   ```

   Specify the exact path:

   ```yaml
   sensor:
     - platform: illuminance
       name: 'Light Sensor'
       device_path: '/sys/bus/iio/devices/iio:device0/in_illuminance_raw'
   ```
