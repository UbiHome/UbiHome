---
title: "Switch"
sidebar:
  order: 1
---

# Switch

```yaml title="Base Example"
switch:
  - platform: ... # [!code highlight]
    name: "My Switch"
```


:::note[Platform-specific properties]
Each platform requires additional properties not shown above. Check the [Platforms documentation](/features/platforms/) and the page for your chosen platform for the full list of required and optional options.
:::

## Attributes

Common attributes are documented in [Common Component Properties](/features/components/base/).

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
| [Shell](/features/platforms/shell/) | Uses `command_on` and `command_off`; optional `command_state` for state polling. |

For platform-specific configuration options, use the linked platform pages.

## Used in Examples

- [Turn Raspberry screen on or off](/examples/screen_on_off/)
- [Automatic Screen Power Control](/examples/automatic_screen_power_control/)

## Shell Platform Notes

For full shell switch setup, see [Shell platform documentation](/features/platforms/shell/#switch).

When using `command_state`, the command output must resolve to `true` or `false`.

Similar to ESPHome: [Switch](https://esphome.io/components/switch/index.html)






