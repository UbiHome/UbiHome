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

and on the [Media Player](/features/entities/media_player/):

- `on_play` — runs when playback starts.
- `on_pause` — runs when playback stops/pauses.
- `on_volume_change` — runs when the volume changes.
- `on_mute_change` — runs when the muted state changes.

There is also a global `on_startup` trigger, configured under `ubihome:`, that runs once when UbiHome starts:

```yaml
ubihome:
  name: 'Raspberry Pi behind the TV'
  on_startup:
    then:
      - switch.turn_on: status_led
```

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
| `lambda`          | JavaScript source | Runs an inline JavaScript lambda. See below. |
| `logger.log`      | value       | Logs the given value to the console at info level. |

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

`logger.log` takes a plain YAML scalar (string, boolean, or number), the same
way `globals.set`'s `value` does:

```yaml
- logger.log: 'Motion detected'
```

## `lambda` Actions

A `lambda` action runs inline JavaScript, similar to an ESPHome C++ lambda.
It has access to:

- `id(name)` — the current value of a [global](/features/components/globals/),
  or (for anything else) a handle exposing only the commands valid for that
  id's actual kind: `turn_on()`/`turn_off()` for a switch, `press()` for a
  button. Calling an unsupported command (or referencing an unknown id)
  throws.
- `set_global(name, value)` — sets a global (there is no assignment syntax
  like ESPHome's `id(x) = value;`).
- `x` — for triggers that carry a commanded value (currently only a
  [template](/features/platforms/template/) number's `set_action`); otherwise
  `undefined`.
- `log(message)` — writes to the application log.
- `delay(ms)` — pauses the script for `ms` milliseconds, e.g. to pace
  repeated commands in a loop instead of firing them all at once.

```yaml
number:
  - platform: template
    name: 'Volume'
    min_value: 0
    max_value: 100
    step: 1
    lambda: |-
      return id(global_volume)
    set_action:
      then:
        - lambda: |
            // x is the value given by set_action; press volume_up_button /
            // volume_down_button one step at a time towards it.
            let difference = id(global_volume) - x
            while (difference != 0) {
              if (difference < 0) {
                id(volume_up_button).press();
                difference = difference + 1
              } else {
                id(volume_down_button).press();
                difference = difference - 1
              }
            }
```

A template switch/number's own `lambda` (the one that computes its reported
state, not a `lambda` action) runs the same JavaScript engine and supports
`id()`/`log()` too, but not `x` — see
[Template](/features/platforms/template/).
