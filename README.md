# OSHome


[Repository](https://github.com/DanielHabenicht/OSHome) -- [Documentation](https://danielhabenicht.github.io/OSHome/) -- [Issues](https://github.com/DanielHabenicht/OSHome/issues)

---

`OSHome` is a system which allows you to integrate any device running any OS (Linux, Windows, MacOS) into your smart home (similar to ESPHome for Microcontrollers).

> Attention: This is a work in progress. The documentation is not complete and the project is still in development.
> Still many things already work - so feel free to try it out.

Simply [download](https://danielhabenicht.github.io/OSHome/getting_started/index.html) the executable and configure it.

```yaml
# Example configuration

oshome:
  name: "Raspberry Pi behind the TV"

mqtt: 
  broker: 192.168.100.23
  username: oshome-tv
  password: <secure_password>

sensor:
  - platform: shell
    name: "RAM Usage"
    id: ram_usage
    icon: mdi:memory
    state_class: "measurement"
    unit_of_measurement: "%"
    update_interval: 30s
    # Execute any command you like:
    command: |-
      free | grep Mem | awk '{print $3/$2 * 100.0}'

# You can also add binary_sensors, buttons, etc.
```

Test it out:

```bash
# Run once:
./oshome run

# Install directly as background service:
./oshome install
```

> Have a look at the [examples](https://danielhabenicht.github.io/OSHome/examples/index.html) to get started.