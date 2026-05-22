---
title: 'Binary Sensor'
sidebar:
  order: 1
---

```yaml title="Base Example"
binary_sensor:
  - platform: ... # [!code highlight]
    name: 'My Binary Sensor'
    device_class: presence
```

:::note[Platform-specific properties]
Each platform requires additional properties not shown above. Check the [Platforms documentation](/features/) and the page for your chosen platform for the full list of required and optional options.
:::

## Attributes

Common attributes are documented in [Common Component Properties](/features/components/base/).

| Property       | Description                                                                    | Example                             |
| -------------- | ------------------------------------------------------------------------------ | ----------------------------------- |
| `device_class` | Home Assistant binary sensor class (for example `presence`, `motion`, `door`). | `presence`                          |
| `on_press`     | Trigger block that runs when state changes to `true`.                          | `then: - switch.turn_on: "screen"`  |
| `on_release`   | Trigger block that runs when state changes to `false`.                         | `then: - switch.turn_off: "screen"` |

## Supported Filters

| Filter                                                     | Description                                                      |
| ---------------------------------------------------------- | ---------------------------------------------------------------- |
| [`invert`](/features/components/filters/#invert)           | Inverts the boolean value (`true` <-> `false`).                  |
| [`delayed_on`](/features/components/filters/#delayed_on)   | Delays propagation of `true` values by the configured duration.  |
| [`delayed_off`](/features/components/filters/#delayed_off) | Delays propagation of `false` values by the configured duration. |

## Supported Platforms

| Platform                            | Notes                                    |
| ----------------------------------- | ---------------------------------------- |
| [Shell](/features/platforms/shell/) | Reads boolean state from command output. |
| [GPIO](/features/platforms/gpio/)   | Reads binary state from GPIO pin events. |

For platform-specific configuration options, use the linked platform pages.

## Used in Examples

- [Motion Detection](/examples/motion_detection/)
- [Automatic Screen Power Control](/examples/automatic_screen_power_control/)
- [Monitor and control Bluetooth devices](/examples/bluetooth_monitor_control/)

Similar to ESPHome: [Binary Sensor](https://esphome.io/components/binary_sensor/)
