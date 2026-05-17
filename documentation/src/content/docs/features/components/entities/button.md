---
title: 'Button'
sidebar:
  order: 1
---

```yaml title="Base Example"
button:
  - platform: ... # [!code highlight]
    name: 'Write Hello World'
```

:::note[Platform-specific properties]
Each platform requires additional properties not shown above. Check the [Platforms documentation](/features/platforms/) and the page for your chosen platform for the full list of required and optional options.
:::

## Attributes

Common attributes are documented in [Common Component Properties](/features/components/base/).

| Property | Description                                                       | Example |
| -------- | ----------------------------------------------------------------- | ------- |
| _(none)_ | Button currently has no additional component-specific attributes. | _(n/a)_ |

## Supported Filters

| Filter   | Description                                |
| -------- | ------------------------------------------ |
| _(none)_ | Button does not currently support filters. |

## Supported Platforms

| Platform                                            | Notes                                                 |
| --------------------------------------------------- | ----------------------------------------------------- |
| [Shell](/features/platforms/shell/)                 | Triggers a command when the button is pressed.        |
| [Power Utilities](/features/platforms/power_utils/) | Triggers device power actions (shutdown/reboot/etc.). |

For platform-specific configuration options, use the linked platform pages.

## Used in Examples

- [Open a new tab in chrome](/examples/open_chrome_tab/)
- [Display a Notification](/examples/display_notification/)
- [Stop or reboot your system](/examples/stop_reboot/)
- [Monitor and control Bluetooth devices](/examples/bluetooth_monitor_control/)

Similar to ESPHome: [Button](https://esphome.io/components/button/)
