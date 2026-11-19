---
title: 'Run commands from a template switch'
description: 'Toggle a template switch from a binary sensor and drive a GPIO relay plus a shell command on switching.'
---

A [template](/features/platforms/template/#switch) switch turns a binary
sensor input into on/off automations. Here a [shell](/features/platforms/shell/)
binary sensor toggles the template switch, whose `turn_on_action` /
`turn_off_action` drive entities on two different platforms - a
[gpio](/features/platforms/gpio/) relay switch and a shell button - and track
state in a [global](/features/components/globals/). The switch's `lambda`
reads that global back with `globals.get`, so its reported state always
reflects the global.

```yaml
ubihome:
  name: UbiHome Example

gpio:
  device: raspberryPi

shell:

globals:
  - id: light_on
    type: bool
    initial_value: false

switch:
  - platform: template
    name: 'Desk Light'
    id: desk_light
    lambda:
      globals.get: light_on
    turn_on_action:
      then:
        - switch.turn_on: relay
        - button.press: log_light_on
        - globals.set:
            id: light_on
            value: true
    turn_off_action:
      then:
        - switch.turn_off: relay
        - button.press: log_light_off
        - globals.set:
            id: light_on
            value: false
  - platform: gpio
    id: relay
    pin: 17

button:
  - platform: shell
    id: log_light_on
    command: 'echo on >> /tmp/desk_light.log'
  - platform: shell
    id: log_light_off
    command: 'echo off >> /tmp/desk_light.log'

binary_sensor:
  - platform: shell
    name: 'Desk Occupancy'
    id: desk_occupancy
    update_interval: 2s
    command: 'cat /tmp/desk_presence 2>/dev/null || echo false'
    on_press:
      then:
        - switch.turn_on: desk_light
    on_release:
      then:
        - switch.turn_off: desk_light
```

The `relay` switch and the two `log_light_*` buttons are configured with only
an `id` (no `name`), so they stay internal - wired up for the template switch
to drive, but not exposed as separate entities.

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/features/entities/switch/">Switch</a>
  <a href="/features/entities/binary_sensor/">Binary Sensor</a>
  <a href="/features/components/actions/">Triggers and Actions</a>
  <a href="/features/components/globals/">Globals</a>
  <a href="/features/platforms/template/">Template</a>
  <a href="/features/platforms/shell/">Shell</a>
  <a href="/features/platforms/gpio/">GPIO</a>
</div>
