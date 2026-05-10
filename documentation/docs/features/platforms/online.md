# Online

The online platform provides a binary sensor that reports whether the device can open a TCP connection to a target host (for example a DNS server).

## Configuration

```yaml
online:
  host: 8.8.8.8
  port: 53
  update_interval: 30s
  timeout: 3s

binary_sensor:
  - platform: online
    name: "Online"
    icon: mdi:web
    device_class: connectivity
```

## Options

### Global platform options (`online:`)

| Property          | Type     | Default   | Description                         |
| ----------------- | -------- | --------- | ----------------------------------- |
| `host`            | string   | `8.8.8.8` | Target host for connectivity checks |
| `port`            | number   | `53`      | Target TCP port                     |
| `update_interval` | duration | `30s`     | Poll interval                       |
| `timeout`         | duration | `3s`      | Connection timeout                  |

### Binary sensor options (`binary_sensor`)

| Property          | Type     | Default            | Description                                      |
| ----------------- | -------- | ------------------ | ------------------------------------------------ |
| `platform`        | string   | **Required**       | Must be `online`                                 |
| `name`            | string   | **Required**       | Display name                                     |
| `host`            | string   | from `online`      | Override target host                             |
| `port`            | number   | from `online`      | Override target port                             |
| `update_interval` | duration | from `online`      | Override poll interval                           |
| `timeout`         | duration | from `online`      | Override timeout                                 |
| `icon`            | string   | `mdi:web`          | Home Assistant icon                              |
| `device_class`    | string   | `connectivity`     | Home Assistant device class                      |
