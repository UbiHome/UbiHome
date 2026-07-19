---
title: 'Run commands from a template switch'
description: 'Toggle a template switch from a binary sensor and run shell commands on switching.'
---

A [template](/features/platforms/template/) switch turns a binary sensor input
into on/off automations. Here a [shell](/features/platforms/shell/) binary
sensor toggles the template switch, whose `turn_on_action` / `turn_off_action`
run shell commands (via [button](/features/entities/button/) presses) and track
state in a [global](/features/components/globals/). The switch's `lambda` reads
that global back with `globals.get`, so its reported state always reflects the
global.

```yaml
ubihome:
  name: UbiHome Example

shell:

globals:
  - id: light_on
    type: bool
    initial_value: 'false'

switch:
  - platform: template
    name: 'Desk Light'
    id: desk_light
    lambda:
      globals.get: light_on
    turn_on_action:
      - button.press: turn_light_on
      - globals.set:
          id: light_on
          value: 'true'
    turn_off_action:
      - button.press: turn_light_off
      - globals.set:
          id: light_on
          value: 'false'

button:
  - platform: shell
    name: 'Turn light on'
    id: turn_light_on
    command: 'echo on > /tmp/desk_light'
  - platform: shell
    name: 'Turn light off'
    id: turn_light_off
    command: 'echo off > /tmp/desk_light'

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

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/features/entities/switch/">Switch</a>
  <a href="/features/entities/binary_sensor/">Binary Sensor</a>
  <a href="/features/components/actions/">Triggers and Actions</a>
  <a href="/features/components/globals/">Globals</a>
  <a href="/features/platforms/template/">Template</a>
  <a href="/features/platforms/shell/">Shell</a>
</div>
