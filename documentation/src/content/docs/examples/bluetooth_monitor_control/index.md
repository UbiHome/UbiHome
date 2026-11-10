---
title: 'Monitor and control Bluetooth devices'
description: 'Use Bluetooth state to monitor or control attached devices.'
---

## Linux

### Bluetoothctl

```yaml
shell:

button:
  - platform: shell
    id: disconnect_speaker
    name: 'Disconnect Speaker'
    command: bluetoothctl -- disconnect 00:12:6F:F1:FF:61

binary_sensor:
  - platform: shell
    name: 'Bluetooth Speaker connected'
    id: bluetooth_connected
    icon: 'mdi:bluetooth-settings'
    device_class: presence
    update_interval: 10s
    command: |-
      if bluetoothctl info 00:12:6F:F1:FF:61 | grep -q "Connected: yes"; then
          echo true
      else
          echo false
      fi
```

### Hcitool

```yaml
shell:

binary_sensor:
  - platform: shell
    name: 'Bluetooth Speaker connected'
    id: bluetooth_connected
    icon: 'mdi:bluetooth-settings'
    device_class: presence
    update_interval: 10s
    command: |-
      if hcitool con | grep -q "00:12:6F:F1:FF:61"; then
          echo true
      else
          echo false
      fi
```

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/features/entities/button/">Button</a>
  <a href="/features/entities/binary_sensor/">Binary Sensor</a>
  <a href="/features/platforms/shell/">Shell</a>
</div>
