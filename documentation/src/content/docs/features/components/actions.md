---
title: 'Triggers and Actions'
---

Some components expose **triggers** that run a list of **actions** when something happens (for example a state change).

Triggers are available on the [Binary Sensor](/features/entities/binary_sensor/):

- `on_press` — runs when the state changes to `true`.
- `on_release` — runs when the state changes to `false`.

and on the [Template](/features/platforms/template/) switch:

- `turn_on_action` — runs when the switch is turned on.
- `turn_off_action` — runs when the switch is turned off.

A binary sensor trigger takes a `then` block listing the actions to run in order
(template switch actions are a plain list, without `then`):

```yaml
binary_sensor:
  - platform: ...
    name: 'Motion'
    filters:
      # Debounce with a delayed_off/delayed_on filter so a trigger only
      # fires once the state has held for the configured duration.
      - delayed_off: 5s
    on_press:
      then:
        - switch.turn_on: screen
    on_release:
      then:
        - switch.turn_off: screen
```

See [Filters](/features/components/filters/) for the available debounce filters.

## Supported Actions

| Action            | Argument    | Description                                             |
| ----------------- | ----------- | ------------------------------------------------------ |
| `switch.turn_on`  | switch `id` | Turns the referenced [switch](/features/entities/switch/) on.  |
| `switch.turn_off` | switch `id` | Turns the referenced [switch](/features/entities/switch/) off. |
| `button.press`    | button `id` | Presses the referenced [button](/features/entities/button/), running its platform action. |
| `globals.set`     | `id`, `value` | Sets a [global](/features/components/globals/) variable to `value`. |

The argument is the `id` of the target entity, so make sure the switch or button you reference has an `id` set.

```yaml
# Setting a global uses id/value arguments instead of a single id:
- globals.set:
    id: door_open
    value: 'true'
```
