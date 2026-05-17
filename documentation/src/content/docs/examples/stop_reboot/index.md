---
title: 'Stop or reboot your system'
---

This example shows how to stop or reboot your system using UbiHome. This is useful if you want to stop or reboot your system based on a trigger in Home Assistant.

```yaml
ubihome:
  name: UbiHome Example

power_utils:

button:
  - platform: power_utils
    name: 'Reboot'
    action: reboot

  - platform: power_utils
    name: 'Shutdown'
    action: shutdown

  - platform: power_utils
    name: 'Hibernate'
    action: hibernate

  - platform: power_utils
    name: 'Logout'
    action: logout

  - platform: power_utils
    name: 'Sleep'
    action: sleep
```

## Related documentation

- Component: [Button](/features/components/entities/button/)
- Platform: [Power Utilities](/features/platforms/power_utils/)

Similar to ESPHome:

- [https://esphome.io/components/button/restart](https://esphome.io/components/button/restart)
- [https://esphome.io/components/button/shutdown](https://esphome.io/components/button/shutdown)
