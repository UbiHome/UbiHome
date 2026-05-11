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

## Used in Examples

- [Motion Detection](../../examples/motion_detection/index.md)
- [Automatic Screen Power Control](../../examples/automatic_screen_power_control/index.md)
