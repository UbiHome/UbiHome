# Binary Sensor

```yaml title="Base Example"
binary_sensor:
  - platform: ... #(1)!
    name: "My Binary Sensor"
    device_class: presence
```

1. Here the [platform](../platforms/index.md) must be defined.

## Attributes

Common attributes are documented in [Common Component Properties](./base.md).

| Property | Description |
| --- | --- |
| `device_class` | Home Assistant binary sensor class (for example `presence`, `motion`, `door`). |
| `on_press` | Trigger block that runs when state changes to `true`. |
| `on_release` | Trigger block that runs when state changes to `false`. |

## Supported Filters

| Filter | Description |
| --- | --- |
| `invert` | Inverts the boolean value (`true` <-> `false`). |
| `delayed_on` | Delays propagation of `true` values by the configured duration. |
| `delayed_off` | Delays propagation of `false` values by the configured duration. |

## Supported Platforms

| Platform | Notes |
| --- | --- |
| [Shell](../platforms/shell.md) | Reads boolean state from command output. |
| [GPIO](../platforms/gpio.md) | Reads binary state from GPIO pin events. |

For platform-specific configuration options, use the linked platform pages.

Similar to ESPHome: [Binary Sensor](https://esphome.io/components/binary_sensor/)
