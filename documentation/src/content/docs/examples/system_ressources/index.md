---
title: 'Monitor system resources'
---

## Windows

```yaml
shell:
  type: powershell

sensor:
  - platform: shell
    name: 'RAM Usage'
    id: ram_usage
    icon: mdi:memory
    state_class: 'measurement'
    unit_of_measurement: '%'
    update_interval: 30s
    command: |-
      Get-WmiObject Win32_OperatingSystem -Property * | % {([math]::Round(($_.FreePhysicalMemory)/$_.totalvisiblememorysize,2))}
```

## Linux

```yaml
shell:

sensor:
  - platform: shell
    name: 'RAM Usage'
    id: ram_usage
    icon: mdi:memory
    state_class: 'measurement'
    unit_of_measurement: '%'
    update_interval: 30s
    command: |-
      free | grep Mem | awk '{print $3/$2 * 100.0}'
```

## Related documentation

- Component: [Sensor](/features/components/entities/sensor/)
- Platform: [Shell](/features/platforms/shell/)
