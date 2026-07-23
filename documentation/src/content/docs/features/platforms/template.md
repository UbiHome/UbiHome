---
title: 'Template'
description: 'A switch, button or number that runs automations instead of talking to hardware'
---

The `template` platform creates entities driven entirely by automations
instead of hardware: their [actions](/features/components/actions/) run in
response to a command or press. It is built into UbiHome, so no top-level
section is required to enable it.

## Switch

Turning the switch on or off runs the configured actions.

```yaml
switch:
  - platform: template
    name: 'Living Room'
    id: living_room
    # optimistic (default true): immediately report the new state after a command.
    optimistic: true
    turn_on_action:
      then:
        - switch.turn_on: relay
    turn_off_action:
      then:
        - switch.turn_off: relay
```

### Attributes

| Property          | Description                                                            | Example   |
| ----------------- | ----------------------------------------------------------------------- | --------- |
| `optimistic`      | Publish the new state right after a command, without state feedback.    | `true`    |
| `assumed_state`   | Whether the state must be assumed. Defaults to `optimistic`.            | `true`    |
| `lambda`          | Source the reported state from a global (see below).                    | see below |
| `turn_on_action`  | List of [actions](/features/components/actions/) run when turned on.    | see above |
| `turn_off_action` | List of [actions](/features/components/actions/) run when turned off.   | see above |

### State from a global (`lambda`)

The `lambda` is written in YAML and currently supports `globals.get`, which
reports the switch state from a `bool` [global](/features/components/globals/).
The state tracks the global live — whenever the global changes (e.g. from a
`turn_on_action` that sets it, or from elsewhere) the switch updates. With a
`lambda` the switch is no longer optimistic; its state always reflects the
global.

```yaml
globals:
  - id: relay_state
    type: bool
    initial_value: false

switch:
  - platform: template
    name: 'Living Room'
    id: living_room
    lambda:
      globals.get: relay_state
    turn_on_action:
      then:
        - globals.set:
            id: relay_state
            value: true
    turn_off_action:
      then:
        - globals.set:
            id: relay_state
            value: false
```

Similar to ESPHome: [Template Switch](https://esphome.io/components/switch/template/)

## Button

Pressing the button (from the API, MQTT, or a `button.press` action) runs
`on_press`.

```yaml
button:
  - platform: template
    name: 'Restart Service'
    id: restart_service
    on_press:
      then:
        - button.press: real_restart_button
```

### Attributes

| Property   | Description                                                         | Example   |
| ---------- | --------------------------------------------------------------------- | --------- |
| `on_press` | List of [actions](/features/components/actions/) run when pressed.    | see above |

Similar to ESPHome: [Template Button](https://esphome.io/components/button/template/)

## Number

Setting the number (from the API, MQTT, or Home Assistant) runs `set_action`.
`min_value`, `max_value`, `step`, `unit_of_measurement` and `device_class` are
the shared [Number](/features/entities/number/) attributes.

```yaml
number:
  - platform: template
    name: 'Fan Speed'
    id: fan_speed
    min_value: 0
    max_value: 100
    step: 1
    optimistic: true
    set_action:
      then:
        - button.press: apply_fan_speed
```

`set_action` has no access to the commanded value (there is no `x` variable,
unlike ESPHome's C++ lambdas); use `optimistic` or a `lambda` so the entity's
own reported state reflects it instead.

### Attributes

| Property         | Description                                                                    | Example   |
| ---------------- | ------------------------------------------------------------------------------- | --------- |
| `optimistic`     | Publish the commanded value right after a command, without state feedback.      | `true`    |
| `initial_value`  | Value to report on startup when not driven by a `lambda`. Defaults to `min_value`. | `0`    |
| `lambda`         | Source the reported value from a `float` global (see below).                    | see below |
| `set_action`     | List of [actions](/features/components/actions/) run when a value is set.       | see above |

### State from a global (`lambda`)

The same `globals.get` mechanism as the template switch above, but reading a
`float` [global](/features/components/globals/) instead of a `bool` one. The
number reports whatever the global currently holds, live, and updates the
global automatically whenever it is set to a new value:

```yaml
globals:
  - id: fan_speed_value
    type: float
    initial_value: 0

number:
  - platform: template
    name: 'Fan Speed'
    id: fan_speed
    min_value: 0
    max_value: 100
    step: 1
    lambda:
      globals.get: fan_speed_value
    # A `lambda`-driven number automatically writes the commanded value back
    # to its backing global, so `set_action` is only needed for extra side
    # effects (e.g. applying the value elsewhere) and can be omitted.
    set_action:
      then:
        - button.press: apply_fan_speed
```

Similar to ESPHome: [Template Number](https://esphome.io/components/number/template/)

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/features/entities/switch/">Switch</a>
  <a href="/features/entities/button/">Button</a>
  <a href="/features/entities/number/">Number</a>
  <a href="/features/components/actions/">Triggers and Actions</a>
  <a href="/features/components/globals/">Globals</a>
</div>
