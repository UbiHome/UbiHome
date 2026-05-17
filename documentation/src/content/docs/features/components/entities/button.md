---
title: "Button"
---

# Button

```yaml title="Base Example"
button:
  - platform: ... # [!code highlight]
    name: "Write Hello World"
```


:::note[Platform-specific properties]
Each platform requires additional properties not shown above. Check the [Platforms documentation](../platforms/index) and the page for your chosen platform for the full list of required and optional options.
:::

## Attributes

Common attributes are documented in [Common Component Properties](./base).

| Property | Description | Example |
| --- | --- | --- |
| _(none)_ | Button currently has no additional component-specific attributes. | _(n/a)_ |

## Supported Filters

| Filter | Description |
| --- | --- |
| _(none)_ | Button does not currently support filters. |

## Supported Platforms

| Platform | Notes |
| --- | --- |
| [Shell](../platforms/shell) | Triggers a command when the button is pressed. |
| [Power Utilities](../platforms/power_utils) | Triggers device power actions (shutdown/reboot/etc.). |

For platform-specific configuration options, use the linked platform pages.

## Used in Examples

- [Open a new tab in chrome](../../examples/open_chrome_tab/index)
- [Display a Notification](../../examples/display_notification/index)
- [Stop or reboot your system](../../examples/stop_reboot/index)
- [Monitor and control Bluetooth devices](../../examples/bluetooth_monitor_control/index)

Similar to ESPHome: [Button](https://esphome.io/components/button/)
