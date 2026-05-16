# Shell

Enable the platform:

```yaml
shell:
  type: powershell
```

## Usage

### Base Properties

| Property        | Description                                   |
| --------------- | --------------------------------------------- |
| update_interval | How often to run the command. Default is 60s. |

> In the future an update interval of `0` will allow you to stream the output of long running commands (e.g. a json log line by line).

### Sensors

```yaml
sensor:
  - platform: shell
    name: 'RAM Usage'
    update_interval: 30s
    command: |-
      free | grep Mem | awk '{print $3/$2 * 100.0}'
```

### Switch

```yaml
switch:
  - platform: shell
    name: 'Screen'
    id: screen
    command_on: 'wlr-randr --output HDMI-A-1 --on'
    command_off: 'wlr-randr --output HDMI-A-1 --off'
    command_state: |-
      wlr-randr --output HDMI-A-1 | grep -q "Enabled: yes"; then
          echo true
      else
          echo false
      fi
```

### Number

```yaml
number:
  - platform: shell
    name: 'Display Brightness'
    id: display_brightness
    unit_of_measurement: '%'
    min_value: 0.0
    max_value: 100.0
    step: 1.0
    update_interval: 5s
    command_state: "cat /tmp/display_brightness"
    command_set: "echo {{ value }} > /tmp/display_brightness"
```

### Text Sensor

```yaml
text_sensor:
  - platform: shell
    name: 'Host Name'
    id: host_name
    update_interval: 30s
    command: 'hostname'
```
