# Power Utilities

```yaml
power_utils:

button:
  - platform: power_utils
    name: 'Shutdown'
    action: shutdown

  - platform: power_utils
    name: 'Reboot'
    action: reboot

  - platform: power_utils
    name: 'Logout'
    action: logout

  - platform: power_utils
    name: 'Hibernate'
    action: hibernate

  - platform: power_utils
    name: 'Sleep'
    action: sleep
```

## Used in Examples

- [Stop or reboot your system](../../examples/stop_reboot/index.md)

Similar to ESPHome:

- https://esphome.io/components/button/restart.html
- https://esphome.io/components/button/shutdown
