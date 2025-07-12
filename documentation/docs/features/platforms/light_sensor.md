---
tags:
  - Light
  - Illuminance
  - Sensor
  - Linux
  - Windows
---

# Light Sensor

The light sensor platform allows you to integrate laptop ambient light sensors into UbiHome. It supports both Linux and Windows systems with automatic sensor detection.

## Features

- Auto-detection of light sensors on Linux systems using Industrial I/O (IIO) framework
- Support for manual device path configuration
- Configurable update intervals (global and per-sensor)
- Cross-platform support (Linux with Windows fallback)
- Proper Home Assistant integration with illuminance device class

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
- Basic Windows sensor framework support (work in progress)
- For immediate use, consider using the [shell platform](shell.md) with PowerShell commands

## Configuration

Enable the platform:

```yaml
light_sensor:
  update_interval: 30s  # Default update interval for all sensors
```

### Basic Usage

```yaml
sensor:
  - platform: light_sensor
    name: "Ambient Light"
    icon: mdi:brightness-6
    unit_of_measurement: "lx"
    device_class: illuminance
    state_class: measurement
```

### Advanced Configuration with Manual Device Path

```yaml
sensor:
  - platform: light_sensor
    name: "Ambient Light Sensor"
    device_path: "/sys/bus/iio/devices/iio:device0/in_illuminance_raw"
    update_interval: 15s
    icon: mdi:brightness-6
    unit_of_measurement: "lx"
    device_class: illuminance
    state_class: measurement
```

## Configuration Options

### Light Sensor Platform

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `platform` | string | **Required** | Must be `light_sensor` |
| `name` | string | **Required** | Display name for the sensor |
| `device_path` | string | Optional | Manual device path (Linux only) |
| `update_interval` | duration | 30s | How often to read the sensor |
| `icon` | string | `mdi:brightness-6` | Home Assistant icon |
| `device_class` | string | `illuminance` | Home Assistant device class |
| `state_class` | string | `measurement` | Home Assistant state class |
| `unit_of_measurement` | string | `lx` | Unit of measurement |

### Global Configuration

```yaml
light_sensor:
  update_interval: 30s  # Default update interval for all sensors
  device_path: "/custom/path"  # Default device path (Linux only)
```

## Troubleshooting

### Linux

1. **No sensor detected**: Check if your system has a light sensor:
   ```bash
   ls /sys/bus/iio/devices/
   find /sys -name "*illuminance*" 2>/dev/null
   ```

2. **Permission issues**: Ensure the user running UbiHome has read access to the sensor files:
   ```bash
   sudo chmod a+r /sys/bus/iio/devices/iio:device*/in_illuminance*
   ```

3. **Manual device path**: If auto-detection fails, specify the exact path:
   ```yaml
   sensor:
     - platform: light_sensor
       name: "Light Sensor"
       device_path: "/sys/bus/iio/devices/iio:device0/in_illuminance_raw"
   ```

### Windows

For Windows systems, consider using the [shell platform](shell.md) as an alternative:

```yaml
sensor:
  - platform: shell
    name: "Ambient Light"
    command: |
      # PowerShell command to read light sensor
      # This is a placeholder - specific implementation depends on your hardware
      Get-WmiObject -Class Win32_LightSensor | Select-Object -ExpandProperty CurrentReading
    update_interval: 30s
    unit_of_measurement: "lx"
    device_class: illuminance
```

## Example Complete Configuration

```yaml
ubihome:
  name: "My Laptop"

mqtt:
  broker: 192.168.1.100
  username: ubihome
  password: your_password

light_sensor:
  update_interval: 30s

sensor:
  - platform: light_sensor
    name: "Ambient Light"
    icon: mdi:brightness-6
    unit_of_measurement: "lx"
    device_class: illuminance
    state_class: measurement
```