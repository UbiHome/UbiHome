---
title: "Automatic Screen Power Control"
---

# Automatic Screen Power Control

Control the screen power state based of a PIR sensor. The screen will turn on when motion is detected and off after a timeout.

```yaml
ubihome:
  name: UbiHome Example

gpio:
  device: raspberryPi

shell:

switch:
  - platform: shell
    name: "Screen"
    id: screen
    command_on: "vcgencmd display_power 1" # [!code highlight]
    command_off: "vcgencmd display_power 0"
    command_state: "vcgencmd display_power"

binary_sensor:
  - platform: gpio
    name: "motion"
    icon: "mdi:motion-sensor"
    device_class: presence
    pin: 23 
    pull_up: true
    update_interval: "0s"
    filters:
     - delayed_off: 20s
    on_press:
      then:
        - switch.turn_on: "screen"
    on_release:
      then:
        - switch.turn_off: "screen"
```

> If the commands are not working you can try out others from the [screen on/off example](../screen_on_off/index).

## Related documentation

- Components: [Switch](../../features/components/switch), [Binary Sensor](../../features/components/binary_sensor)
- Platforms: [Shell](../../features/platforms/shell), [GPIO](../../features/platforms/gpio)
