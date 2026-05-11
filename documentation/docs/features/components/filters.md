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

## Filter Support by Component

| Component | Supported Filters |
| --- | --- |
| [Sensor](./sensor.md) | `round` |
| [Binary Sensor](./binary_sensor.md) | `invert`, `delayed_on`, `delayed_off` |
| [Button](./button.md) | None |
| [Number](./number.md) | None |
| [Switch](./switch.md) | None |

See each component page for details and examples.
