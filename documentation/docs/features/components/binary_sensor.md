# Binary Sensor

```yaml title="Base Example"
binary_sensor:
  - platform: ... #(1)!
    name: "My Binary Sensor"
    device_class: presence
```

1. Here the [platform](../platforms/index.md) must be defined.

!!! info "Platform-specific properties"
    Each platform requires additional properties not shown above. Check the [Platforms documentation](../platforms/index.md) and the page for your chosen platform for the full list of required and optional options.

## Attributes

Common attributes are documented in [Common Component Properties](./base.md).

| Property | Description | Example |
| --- | --- | --- |
| `device_class` | Home Assistant binary sensor class (for example `presence`, `motion`, `door`). | `presence` |
| `on_press` | Trigger block that runs when state changes to `true`. | `then: - switch.turn_on: "screen"` |
| `on_release` | Trigger block that runs when state changes to `false`. | `then: - switch.turn_off: "screen"` |

## Supported Filters

| Filter | Description |
| --- | --- |
| [`invert`](./filters.md#invert) | Inverts the boolean value (`true` <-> `false`). |
| [`delayed_on`](./filters.md#delayed_on) | Delays propagation of `true` values by the configured duration. |
| [`delayed_off`](./filters.md#delayed_off) | Delays propagation of `false` values by the configured duration. |

## Supported Platforms

| Platform | Notes |
| --- | --- |
| [Shell](../platforms/shell.md) | Reads boolean state from command output. |
| [GPIO](../platforms/gpio.md) | Reads binary state from GPIO pin events. |

For platform-specific configuration options, use the linked platform pages.

## Used in Examples

- [Motion Detection](../../examples/motion_detection/index.md)
- [Automatic Screen Power Control](../../examples/automatic_screen_power_control/index.md)
- [Monitor and control Bluetooth devices](../../examples/bluetooth_monitor_control/index.md)

Similar to ESPHome: [Binary Sensor](https://esphome.io/components/binary_sensor/)
