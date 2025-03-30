# OSHome


[Documentation](https://github.com/DanielHabenicht/OSHome) -- [Issues](https://github.com/DanielHabenicht/OSHome/issues)

---

Analog to ESPHome `OSHome` is a system which allows you to integrate any device running an OS into your smart home.

Simply [download](https://danielhabenicht.github.io/OSHome/getting_started.html) the executable and configure it.

```yaml
# Example configuration

oshome:
  name: "Raspberry Pi behind the TV"

sensor:
  - platform: shell
    name: "RAM Usage"
    id: ram_usage
    icon: mdi:memory
    # device_class: "data_size"
    state_class: "measurement"
    unit_of_measurement: "%"
    update_interval: 30s # 0 only executes it once and assumes a long running processes.
    command: |-
      free | grep Mem | awk '{print $3/$2 * 100.0}'

# You can add many more sensors: 

```

> Have a look at the [examples](https://danielhabenicht.github.io/OSHome/examples/index.html) to get started.

Run it with `oshome run` or install it as a service `oshome install`.