---
title: 'Shell'
description: 'Execute any command to retrieve data or on trigger'
tags:
  - linux
  - macos
  - windows
---

Enable the platform:

```yaml
shell:
  type: powershell
```

## Usage

### Base Properties

| Property        | Description                                   |
| --------------- | --------------------------------------------- |
| update_interval | How often to run the command. Default is 60s. |

> In the future an update interval of `0` will allow you to stream the output of long running commands (e.g. a json log line by line).

### Sensors

```yaml
sensor:
  - platform: shell
    name: 'RAM Usage'
    update_interval: 30s
    command: |-
      free | grep Mem | awk '{print $3/$2 * 100.0}'
```

### Binary Sensor

```yaml
binary_sensor:
  - platform: shell
    name: 'Door Open'
    update_interval: 30s
    command: |-
      if [ -f /tmp/door_open ]; then echo true; else echo false; fi
```

The command output must be `true` or `false` (case-insensitive).

### Switch

```yaml
switch:
  - platform: shell
    name: 'Screen'
    id: screen
    command_on: 'wlr-randr --output HDMI-A-1 --on'
    command_off: 'wlr-randr --output HDMI-A-1 --off'
    command_state: |-
      wlr-randr --output HDMI-A-1 | grep -q "Enabled: yes"; then
          echo true
      else
          echo false
      fi
```

### Number

```yaml
number:
  - platform: shell
    name: 'Display Brightness'
    id: display_brightness
    unit_of_measurement: '%'
    min_value: 0.0
    max_value: 100.0
    step: 1.0
    update_interval: 5s
    command_state: 'cat /tmp/display_brightness'
    command_set: 'echo {{ value }} > /tmp/display_brightness'
```

### Text Sensor

```yaml
text_sensor:
  - platform: shell
    name: 'Host Name'
    id: host_name
    update_interval: 30s
    command: 'hostname'
```

### Button

```yaml
button:
  - platform: shell
    name: 'Open Chrome'
    command: 'xdg-open https://example.com'
```

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/features/entities/sensor/">Sensor</a>
  <a href="/features/entities/binary_sensor/">Binary Sensor</a>
  <a href="/features/entities/switch/">Switch</a>
  <a href="/features/entities/number/">Number</a>
  <a href="/features/entities/text_sensor/">Text Sensor</a>
  <a href="/features/entities/button/">Button</a>
  <a href="/examples/open_chrome_tab/">Open a new tab in chrome</a>
  <a href="/examples/display_notification/">Display a Notification</a>
  <a href="/examples/system_ressources/">Monitor system resources</a>
  <a href="/examples/screen_on_off/">Turn Raspberry screen on or off</a>
  <a href="/examples/automatic_screen_power_control/">Automatic Screen Power Control</a>
  <a href="/examples/bluetooth_monitor_control/">Monitor and control Bluetooth devices</a>
</div>
