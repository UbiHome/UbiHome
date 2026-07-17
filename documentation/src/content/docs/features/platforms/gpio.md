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
```

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/features/entities/binary_sensor/">Binary Sensor</a>
  <a href="/examples/motion_detection/">Motion Detection</a>
  <a href="/examples/automatic_screen_power_control/">Automatic Screen Power Control</a>
</div>
