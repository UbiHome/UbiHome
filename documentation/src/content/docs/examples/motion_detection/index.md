---
title: 'Motion Detection'
---

Detect motion using a simple PIR Sensor.

```yaml
ubihome:
  name: UbiHome Example

gpio:
  device: raspberryPi

binary_sensor:
  - platform: gpio
    name: 'motion'
    icon: 'mdi:motion-sensor'
    device_class: presence
    pin: 23
    pull_up: true
    update_interval: '0s'
    filters:
      - delayed_off: 10s
```

You can combine this with the [screen on/off example](/examples/screen_on_off/) to turn the screen on when motion is detected and off after a timeout. Look at the [automatic screen power control](/examples/automatic_screen_power_control/) on how to set it up.

## Related documentation

- Component: [Binary Sensor](/features/components/entities/binary_sensor/)
- Platform: [GPIO](/features/platforms/gpio/)
