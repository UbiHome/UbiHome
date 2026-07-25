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

### State from a `lambda`

`lambda` is an inline JavaScript expression (see
[Triggers and Actions](/features/components/actions/#lambda-actions)) that
computes the reported state, e.g. `return id(relay_state)` to read a `bool`
[global](/features/components/globals/). The state tracks the lambda live —
whenever any global changes, it is re-evaluated. With a `lambda` the switch is
no longer optimistic; its state always reflects the lambda's result.

```yaml
globals:
  - id: relay_state
    type: bool
    initial_value: false

switch:
  - platform: template
    name: 'Living Room'
    id: living_room
    lambda: |-
      return id(relay_state)
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

A plain action in `set_action` (like `button.press` above) has no access to
the commanded value; a `lambda` action does, as the `x` variable (see
[Triggers and Actions](/features/components/actions/#lambda-actions)).

### Attributes

| Property         | Description                                                                    | Example   |
| ---------------- | ------------------------------------------------------------------------------- | --------- |
| `optimistic`     | Publish the commanded value right after a command, without state feedback.      | `true`    |
| `initial_value`  | Value to report on startup when not driven by a `lambda`. Defaults to `min_value`. | `0`    |
| `lambda`         | Source the reported value from a JavaScript expression (see below).             | see below |
| `set_action`     | List of [actions](/features/components/actions/) run when a value is set.       | see above |

### State from a `lambda`

The same `lambda` mechanism as the template switch above, e.g.
`return id(fan_speed_value)` to read a `float`
[global](/features/components/globals/). The number reports whatever the
lambda currently returns, live (re-evaluated on every global change). Unlike
`optimistic`, a `lambda`-driven number does not echo the commanded value on
its own — write the value back (e.g. with `set_global`) from a `lambda`
action in `set_action`:

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
    lambda: |-
      return id(fan_speed_value)
    set_action:
      then:
        - button.press: apply_fan_speed
        - lambda: |
            set_global('fan_speed_value', x)
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
