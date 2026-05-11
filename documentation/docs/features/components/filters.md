# Filters

Filters can be used on supported components to modify incoming state values.

```yaml
sensor:
  - platform: shell
    name: "RAM Usage"
    command: echo 1.123345
    filters:
      - round: 2

# Sensor value will be shown as 1.12
```

## Available Filters

### round

Rounds numeric sensor values to the configured number of decimal places.

```yaml
sensor:
  - platform: shell
    name: "CPU Usage"
    command: echo 12.3456
    filters:
      - round: 2

# Result: 12.35
```

### invert

Inverts a boolean binary sensor value (`true` becomes `false`, `false` becomes `true`).

```yaml
binary_sensor:
  - platform: shell
    name: "Door Closed"
    command: test -f /tmp/door_open && echo true || echo false
    filters:
      - invert:

# If command returns true, the reported state becomes false
```

### delayed_on

Delays propagation of `true` values by the configured duration. `false` values are still passed immediately.

```yaml
binary_sensor:
  - platform: gpio
    name: "Motion"
    pin: 17
    filters:
      - delayed_on: 1s

# Motion must stay true for 1 second before state changes to true
```

### delayed_off

Delays propagation of `false` values by the configured duration. `true` values are still passed immediately.

```yaml
binary_sensor:
  - platform: gpio
    name: "Motion"
    pin: 17
    filters:
      - delayed_off: 5s

# Motion must stay false for 5 seconds before state changes to false
```

## Filter Support by Component

| Component | Supported Filters |
| --- | --- |
| [Sensor](./sensor.md) | [`round`](#round) |
| [Binary Sensor](./binary_sensor.md) | [`invert`](#invert), [`delayed_on`](#delayed_on), [`delayed_off`](#delayed_off) |
| [Button](./button.md) | None |
| [Number](./number.md) | None |
| [Switch](./switch.md) | None |

See each component page for details and examples.
