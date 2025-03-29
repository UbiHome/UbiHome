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