---
title: 'Template'
description: 'A switch that runs automations when it is turned on or off'
---

The `template` platform creates a [switch](/features/entities/switch/) that is
driven entirely by automations: turning it on or off runs the configured
[actions](/features/components/actions/). It is built into UbiHome, so no
top-level section is required to enable it.

```yaml
switch:
  - platform: template
    name: 'Living Room'
    id: living_room
    # optimistic (default true): immediately report the new state after a command.
    optimistic: true
    turn_on_action:
      - switch.turn_on: relay
    turn_off_action:
      - switch.turn_off: relay
```

## Attributes

| Property           | Description                                                              | Example        |
| ------------------ | ------------------------------------------------------------------------ | -------------- |
| `optimistic`       | Publish the new state right after a command, without state feedback.     | `true`         |
| `assumed_state`    | Whether the state must be assumed. Defaults to `optimistic`.             | `true`         |
| `lambda`           | Source the reported state from a global (see below).                     | see below      |
| `turn_on_action`   | List of [actions](/features/components/actions/) run when turned on.     | see above      |
| `turn_off_action`  | List of [actions](/features/components/actions/) run when turned off.    | see above      |

### State from a global (`lambda`)

Instead of a C++ lambda, the `lambda` is written in YAML and currently supports
`globals.get`, which reports the switch state from a `bool`
[global](/features/components/globals/). The state tracks the global live —
whenever the global changes (e.g. from a `turn_on_action` that sets it, or from
elsewhere) the switch updates. With a `lambda` the switch is no longer optimistic;
its state always reflects the global.

```yaml
globals:
  - id: relay_state
    type: bool
    initial_value: 'false'

switch:
  - platform: template
    name: 'Living Room'
    id: living_room
    lambda:
      globals.get: relay_state
    turn_on_action:
      - globals.set:
          id: relay_state
          value: 'true'
    turn_off_action:
      - globals.set:
          id: relay_state
          value: 'false'
```

C++ lambdas are not supported; use `globals.get` for state and the
`turn_on_action` / `turn_off_action` automations to drive other entities.

Similar to ESPHome: [Template Switch](https://esphome.io/components/switch/template/)

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/features/entities/switch/">Switch</a>
  <a href="/features/components/actions/">Triggers and Actions</a>
  <a href="/features/components/globals/">Globals</a>
</div>
