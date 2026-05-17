---
title: "Text Sensor"
---

# Text Sensor

```yaml title="Base Example"
text_sensor:
  - platform: ... # [!code highlight]
    name: 'Host Name'
    id: host_name
    icon: mdi:identifier
    # device_class: "timestamp"
```


Text sensors publish string values instead of numeric values.

Similar to ESPHome:

- https://esphome.io/components/text_sensor/
