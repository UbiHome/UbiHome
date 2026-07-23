---
title: 'Triggers and Actions'
---

Some components expose **triggers** that run a list of **actions** when something happens (for example a state change).

Triggers are available on the [Binary Sensor](/features/entities/binary_sensor/):

- `on_press` — runs when the state changes to `true`.
- `on_release` — runs when the state changes to `false`.

and on the [Template](/features/platforms/template/) switch, button and number:

- `turn_on_action` — runs when the template switch is turned on.
- `turn_off_action` — runs when the template switch is turned off.
- `on_press` — runs when the template button is pressed.
- `set_action` — runs when the template number is set to a new value.

A trigger takes a `then` block listing the actions to run in order:

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
| `delay`           | duration    | Pauses the action list for the given duration (e.g. `2s`, `500ms`) before running the next action. |

For entity actions the argument is the `id` of the target entity, so make sure the switch or button you reference has an `id` set.

```yaml
binary_sensor:
  - platform: gpio
    name: 'Button'
    pin: 17
    on_press:
      then:
        - switch.turn_on: screen
        - delay: 30s
        - switch.turn_off: screen
```

`globals.set` takes `id`/`value` arguments instead of a single id; see
[Globals](/features/components/globals/) for the `value` syntax.
