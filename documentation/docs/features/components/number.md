# Number

```yaml title="Base Example"
number:
  - platform: ... #(1)!
    name: "Display Brightness"
    id: display_brightness
    icon: mdi:brightness-6
    unit_of_measurement: "%"
    min_value: 0.0
    max_value: 100.0
    step: 1.0
```

1.  Here the [platform](./../platforms/index.md) must be defined.

## Attributes

| Property            | Description |
| ------------------- | ----------- |
| min_value           | Lower boundary for allowed values. |
| max_value           | Upper boundary for allowed values. |
| step                | Increment/decrement step size. |
| unit_of_measurement | Optional value unit shown in user interfaces. |

Similar to ESPHome:

- https://esphome.io/components/number/
