# Number

```yaml title="Base Example"
number:
  - platform: ... #(1)!
    name: "Display Brightness"
    id: display_brightness
    icon: mdi:brightness-6
    device_class: ""
    state_class: measurement
    unit_of_measurement: "%"
    min_value: 0.0
    max_value: 100.0
    step: 1.0
```

1. Here the [platform](../platforms/index.md) must be defined.

## Attributes

Common attributes are documented in [Common Component Properties](./base.md).

| Property | Description |
| --- | --- |
| `device_class` | Home Assistant number classification. |
| `state_class` | Home Assistant state class for number values. |
| `unit_of_measurement` | Optional value unit shown in user interfaces. |
| `min_value` | Lower boundary for allowed values. |
| `max_value` | Upper boundary for allowed values. |
| `step` | Increment/decrement step size. |

## Supported Filters

| Filter | Description |
| --- | --- |
| _(none)_ | Number does not currently support filters. |

## Supported Platforms

| Platform | Notes |
| --- | --- |
| [Shell](../platforms/shell.md) | Uses `command_state` for reads and `command_set` for writes. |

For platform-specific configuration options, use the linked platform pages.

Similar to ESPHome: [Number](https://esphome.io/components/number/)
