# Text Sensor

```yaml title="Base Example"
text_sensor:
  - platform: ... #(1)!
    name: 'Host Name'
    id: host_name
    icon: mdi:identifier
    # device_class: "timestamp"
```

1.  Here the [platform](./../platforms/index.md) must be defined.

Text sensors publish string values instead of numeric values.

Similar to ESPHome:

- https://esphome.io/components/text_sensor/
