---
title: 'ICMP Ping'
tags:
  - linux
  - macos
  - windows
---

Enable the platform:

```yaml
icmp_ping:
  timeout: 5s
```

## Usage

### Base Properties

| Property | Description |
| -------- | ----------- |
| timeout  | Maximum time to wait for a ping before it is treated as failed. Default is 5s. |

### Sensor

```yaml
sensor:
  - platform: icmp_ping
    name: 'Router Latency'
    ip: 192.168.1.1
    update_interval: 30s
```

### Binary Sensor

```yaml
binary_sensor:
  - platform: icmp_ping
    name: 'Router Online'
    ip: 192.168.1.1
    update_interval: 10s
    timeout: 2s
```
