# UbiHome


[Repository](https://github.com/UbiHome/UbiHome) -- [Documentation](https://ubihome.github.io/UbiHome/) -- [Issues](https://github.com/UbiHome/UbiHome/issues)

---

`UbiHome` is a ubiquitous system which allows you to integrate a device running any OS (Linux, Windows, MacOS) into your smart home (similar to ESPHome for Microcontrollers).

> Attention: This is a work in progress. The documentation is not complete and the project is still in development.
> Still many things already work - so feel free to try it out.

Simply [download](https://ubihome.github.io/getting_started/index.html) the executable and configure it.

```yaml
# Example configuration

ubihome:
  name: "Raspberry Pi behind the TV"

mqtt: 
  broker: 192.168.100.23
  username: ubihome-tv
  password: <secure_password>

sensor:
  - platform: shell
    name: "RAM Usage"
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
./ubihome run

# Install directly as background service:
./ubihome install
```

Have a look at the [examples](https://ubihome.github.io/examples/index.html) to get started.
