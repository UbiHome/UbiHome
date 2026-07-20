---
title: 'GPIO'
description: 'Access and control GPIO pins'
tags:
  - linux
---

```yaml
gpio:
  device: raspberryPi

binary_sensor:
  - platform: gpio
    name: 'Motion'
    icon: 'mdi:motion-sensor'
    device_class: presence
    pin: 23

switch:
  - platform: gpio
    name: 'Relay'
    pin: 5
    inverted: false # optional, defaults to false
    restore_mode: ALWAYS_OFF # optional, see below for allowed values
```

### Switch

Uses the same `pin`, `inverted` and `restore_mode` options as [ESPHome's GPIO Switch](https://esphome.io/components/switch/gpio/) (interlocking between switches is not supported).

| Property       | Description                                                             | Default       |
| -------------- | ------------------------------------------------------------------------ | -------------- |
| `pin`          | GPIO pin number to drive.                                                 | _(required)_   |
| `inverted`     | Inverts the pin logic (`true` = active-low).                             | `false`         |
| `restore_mode` | One of `ALWAYS_OFF`, `ALWAYS_ON`, `DISABLED`.                             | `ALWAYS_OFF`    |

:::note
State is not persisted across restarts, so ESPHome's `RESTORE_*` restore modes aren't supported yet — only the modes that don't require persistence (`ALWAYS_OFF`, `ALWAYS_ON`, `DISABLED`).
:::

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/features/entities/binary_sensor/">Binary Sensor</a>
  <a href="/features/entities/switch/">Switch</a>
  <a href="/examples/motion_detection/">Motion Detection</a>
  <a href="/examples/automatic_screen_power_control/">Automatic Screen Power Control</a>
</div>
