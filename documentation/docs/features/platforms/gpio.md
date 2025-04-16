# GPIO

```yaml
gpio:
  device: raspberryPi

binary_sensor:
  - platform: gpio
    name: "Motion"
    icon: "mdi:motion-sensor"
    device_class: presence
    pin: 23
```