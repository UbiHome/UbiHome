# Shell

Enable the platform:

```yaml
shell: 
  type: powershell
```

## Usage

## Sensors

```yaml
sensor:
  - platform: shell
    name: "RAM Usage"
    update_interval: 30s
    command: |-
      free | grep Mem | awk '{print $3/$2 * 100.0}'
```

# Switch

```yaml
switch:
  - platform: shell
    name: "Screen"
    id: screen
    command_on: "wlr-randr --output HDMI-A-1 --on"
    command_off: "wlr-randr --output HDMI-A-1 --off"
    command_state: |-
      wlr-randr --output HDMI-A-1 | grep -q "Enabled: yes"; then
          echo true
      else
          echo false
      fi
```