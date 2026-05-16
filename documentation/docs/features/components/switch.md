# Switch

```yaml title="Base Example"
switch:
  - platform: ... #(1)!
    name: "My Switch"
```

1. Here the [platform](../platforms/index.md) must be defined.

!!! info "Platform-specific properties"
    Each platform requires additional properties not shown above. Check the [Platforms documentation](../platforms/index.md) and the page for your chosen platform for the full list of required and optional options.

## Attributes

Common attributes are documented in [Common Component Properties](./base.md).

| Property | Description | Example |
| --- | --- | --- |
| `device_class` | Home Assistant switch classification. | `outlet` |

## Supported Filters

| Filter | Description |
| --- | --- |
| _(none)_ | Switch does not currently support filters. |

## Supported Platforms

| Platform | Notes |
| --- | --- |
| [Shell](../platforms/shell.md) | Uses `command_on` and `command_off`; optional `command_state` for state polling. |

For platform-specific configuration options, use the linked platform pages.

## Used in Examples

- [Turn Raspberry screen on or off](../../examples/screen_on_off/index.md)
- [Automatic Screen Power Control](../../examples/automatic_screen_power_control/index.md)

## Shell Platform Notes

For full shell switch setup, see [Shell platform documentation](../platforms/shell.md#switch).

When using `command_state`, the command output must resolve to `true` or `false`.

Similar to ESPHome: [Switch](https://esphome.io/components/switch/index.html)
