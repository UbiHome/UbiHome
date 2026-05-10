# Internet

The internet platform provides a binary sensor that reports whether the device can open a TCP connection to a target host (for example a DNS server).

## Configuration

```yaml
internet:
  host: 8.8.8.8
  port: 53
  update_interval: 30s
  timeout: 3s

binary_sensor:
  - platform: internet
    name: "Internet"
    icon: mdi:web
    device_class: connectivity
```

## Options

### Global platform options (`internet:`)

| Property          | Type     | Default   | Description                         |
| ----------------- | -------- | --------- | ----------------------------------- |
| `host`            | string   | `8.8.8.8` | Target host for connectivity checks |
| `port`            | number   | `53`      | Target TCP port                     |
| `update_interval` | duration | `30s`     | Poll interval                       |
| `timeout`         | duration | `3s`      | Connection timeout                  |

### Binary sensor options (`binary_sensor`)

| Property          | Type     | Default            | Description                                      |
| ----------------- | -------- | ------------------ | ------------------------------------------------ |
| `platform`        | string   | **Required**       | Must be `internet`                               |
| `name`            | string   | **Required**       | Display name                                     |
| `host`            | string   | from `internet`    | Override target host                             |
| `port`            | number   | from `internet`    | Override target port                             |
| `update_interval` | duration | from `internet`    | Override poll interval                           |
| `timeout`         | duration | from `internet`    | Override timeout                                 |
| `icon`            | string   | `mdi:web`          | Home Assistant icon                              |
| `device_class`    | string   | `connectivity`     | Home Assistant device class                      |
