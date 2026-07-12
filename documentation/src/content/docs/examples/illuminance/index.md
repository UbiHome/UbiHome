---
title: "Report your laptop's ambient light sensor"
description: 'Expose ambient light sensor values from a laptop as a UbiHome sensor.'
---

Expose your laptop's built-in ambient light sensor as an illuminance value in UbiHome.

```yaml
# UbiHome Illuminance Sensor Example Configuration

ubihome:
  name: 'My Laptop'

illuminance:

# Individual illuminance sensor
sensor:
  - platform: illuminance
    name: 'Ambient Light Sensor'
    icon: mdi:brightness-6
    state_class: 'measurement'
    device_class: 'illuminance'
    update_interval: 15s
```

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/features/entities/sensor/">Sensor</a>
  <a href="/features/platforms/illuminance/">Illuminance</a>
</div>


