---
title: 'Reboot on internet outage'
description: 'Automatically reboot the device after it loses internet connectivity for 5 minutes.'
---

This example reboots the device automatically once it has lost internet connectivity for five minutes. It combines the [online](/features/platforms/online/) binary sensor with a [power_utils](/features/platforms/power_utils/) reboot button.

The `delayed_off: 5min` filter makes sure the device only reboots after the connection has been down continuously for five minutes; a short blip that recovers within that window is ignored. When the sensor finally reports "offline", the `on_release` trigger presses the reboot button via the `button.press` action.

```yaml
ubihome:
  name: UbiHome Example

online:
  update_interval: 10s

power_utils:

binary_sensor:
  - platform: online
    name: 'Internet'
    id: internet
    filters:
      # Only report "offline" once the outage has lasted 5 minutes.
      - delayed_off: 5min
    on_release:
      then:
        - button.press: reboot_on_outage

button:
  - platform: power_utils
    name: 'Reboot on outage'
    id: reboot_on_outage
    action: reboot
```

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/features/entities/binary_sensor/">Binary Sensor</a>
  <a href="/features/components/actions/">Triggers and Actions</a>
  <a href="/features/platforms/online/">Online</a>
  <a href="/features/platforms/power_utils/">Power Utilities</a>
</div>
