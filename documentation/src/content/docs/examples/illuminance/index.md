---
title: "Report your laptop's ambient light sensor"
---

# Report your laptop's ambient light sensor

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

## Related documentation

- Component: [Sensor](/features/components/entities/sensor/)
- Platform: [Illuminance](/features/platforms/illuminance/)






