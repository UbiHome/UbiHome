---
title: 'Automatic Screen Power Control'
description: 'Automatically switch a screen on or off based on device state.'
---

Control the screen power state based on a PIR sensor. The screen will turn on when motion is detected and off after a timeout.

```yaml
ubihome:
  name: UbiHome Example

gpio:
  device: raspberryPi

shell:

switch:
  - platform: shell
    name: 'Screen'
    id: screen
    command_on: 'vcgencmd display_power 1'
    command_off: 'vcgencmd display_power 0'
    command_state: 'vcgencmd display_power'

binary_sensor:
  - platform: gpio
    name: 'motion'
    icon: 'mdi:motion-sensor'
    device_class: presence
    pin: 23
    pull_up: true
    update_interval: '0s'
    filters:
      - delayed_off: 20s
    on_press:
      then:
        - switch.turn_on: 'screen'
    on_release:
      then:
        - switch.turn_off: 'screen'
```

> If the commands are not working you can try out others from the [screen on/off example](/examples/screen_on_off/).

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/features/components/actions/"></a>
  <a href="/features/components/filters/"></a>
  <a href="/features/entities/switch/">Switch</a>
  <a href="/features/entities/binary_sensor/">Binary Sensor</a>
  <a href="/features/platforms/shell/">Shell</a>
  <a href="/features/platforms/gpio/">GPIO</a>
</div>
