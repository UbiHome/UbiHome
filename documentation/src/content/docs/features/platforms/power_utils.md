---
title: 'Power Utilities'
tags:
  - linux
  - macos
  - windows
---

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

Similar to ESPHome:

- https://esphome.io/components/button/restart.html
- https://esphome.io/components/button/shutdown

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/examples/stop_reboot/">Stop or reboot your system</a>
</div>