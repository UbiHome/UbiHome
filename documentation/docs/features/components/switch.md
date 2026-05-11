# Switch

```yaml title="Base Example"
switch:
  - platform: ... #(1)!
    name: "My Switch"
```

1. Here the [platform](../platforms/index.md) must be defined.

## Attributes

Common attributes are documented in [Common Component Properties](./base.md).

| Property | Description |
| --- | --- |
| `device_class` | Home Assistant switch classification. |

## Supported Filters

| Filter | Description |
| --- | --- |
| _(none)_ | Switch does not currently support filters. |

## Supported Platforms

| Platform | Notes |
| --- | --- |
| [Shell](../platforms/shell.md) | Uses `command_on` and `command_off`; optional `command_state` for state polling. |

For platform-specific configuration options, use the linked platform pages.

## Shell Platform Notes

For full shell switch setup, see [Shell platform documentation](../platforms/shell.md#switch).

When using `command_state`, the command output must resolve to `true` or `false`.

Similar to ESPHome: [Switch](https://esphome.io/components/switch/index.html)
