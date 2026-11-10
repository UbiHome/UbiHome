---
title: 'Power Utilities'
description: 'Shutdown, reboot, logout, hibernate or sleep the device'
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
  <a href="/features/entities/button/">Button</a>
  <a href="/examples/stop_reboot/">Stop or reboot your system</a>
  <a href="/examples/reboot_on_connectivity_loss/">Reboot on internet outage</a>
</div>
