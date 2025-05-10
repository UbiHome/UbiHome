# Sensor

```yaml title="Base Example"
sensor:
  - platform: ... #(1)!
    name: "My Sensor"
    id: ram_usage
    icon: mdi:memory
    # device_class: "data_size"
    state_class: "measurement"
    unit_of_measurement: "%"

```

1.  Here the [plaform](./../platforms/index.md) must be defined. 


Similar to ESPHome:

 - https://esphome.io/components/sensor/