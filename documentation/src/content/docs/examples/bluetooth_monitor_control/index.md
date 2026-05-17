---
title: "Monitor and control Bluetooth devices"
---

# Monitor and control Bluetooth devices

## Linux


## Bluetoothctl

```yaml
shell: 

button:
 - platform: shell
   id: disconnect_speaker
   name: "Disconnect Speaker"
   command: bluetoothctl -- disconnect 00:12:6F:F1:FF:61


binary_sensor:
  - platform: shell
    name: "Bluetooth Speaker connected"
    id: bluetooth_connected
    icon: "mdi:bluetooth-settings"
    device_class: presence
    update_interval: 10s
    command: |-
      if bluetoothctl info 00:12:6F:F1:FF:61 | grep -q "Connected: yes"; then
          echo true
      else
          echo false
      fi
```

## Hcitool

```yaml
shell: 

binary_sensor:
  - platform: shell
    name: "Bluetooth Speaker connected"
    id: bluetooth_connected
    icon: "mdi:bluetooth-settings"
    device_class: presence
    update_interval: 10s
    command: |-
      if hcitool con | grep -q "00:12:6F:F1:FF:61"; then
          echo true
      else
          echo false
      fi
```

## Related documentation

- Components: [Button](/features/components/entities/button/), [Binary Sensor](/features/components/entities/binary_sensor/)
- Platform: [Shell](/features/platforms/shell/)






