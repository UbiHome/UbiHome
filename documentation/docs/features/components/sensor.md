# Sensor

```yaml
sensor:
  - platform: shell
    name: "RAM Usage"
    id: ram_usage
    icon: "mdi:bluetooth-settings"
    # device_class: "data_size"
    state_class: "measurement"
    unit_of_measurement: "%"
    update_interval: 30s # 0 only executes it once and assumes a long running processes.
    command: |-
      free | grep Mem | awk '{print $3/$2 * 100.0}'

```



<!-- Reference: https://esphome.io/components/sensor/ -->