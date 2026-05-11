# Button

```yaml title="Base Example"
button:
  - platform: ... #(1)!
    name: "Write Hello World"
```

1. Here the [platform](../platforms/index.md) must be defined.

!!! info "Platform-specific properties"
    Each platform requires additional properties not shown above. Check the [Platforms documentation](../platforms/index.md) and the page for your chosen platform for the full list of required and optional options.

## Attributes

Common attributes are documented in [Common Component Properties](./base.md).

| Property | Description |
| --- | --- |
| _(none)_ | Button currently has no additional component-specific attributes. |

## Supported Filters

| Filter | Description |
| --- | --- |
| _(none)_ | Button does not currently support filters. |

## Supported Platforms

| Platform | Notes |
| --- | --- |
| [Shell](../platforms/shell.md) | Triggers a command when the button is pressed. |
| [Power Utilities](../platforms/power_utils.md) | Triggers device power actions (shutdown/reboot/etc.). |

For platform-specific configuration options, use the linked platform pages.

## Used in Examples

- [Open a new tab in chrome](../../examples/open_chrome_tab/index.md)
- [Display a Notification](../../examples/display_notification/index.md)
- [Stop or reboot your system](../../examples/stop_reboot/index.md)
- [Monitor and control Bluetooth devices](../../examples/bluetooth_monitor_control/index.md)

Similar to ESPHome: [Button](https://esphome.io/components/button/)
