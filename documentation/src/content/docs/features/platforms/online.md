# Online

The online platform provides a binary sensor that reports whether the device can reach any of a list of connectivity targets. By default it queries four well-known DNS servers over UDP, which is lightweight and requires no persistent connection.

## Configuration

```yaml
online:
  update_interval: 30s
  timeout: 3s
  # Defaults to the four DNS servers below; override to use your own targets.
  targets:
    - host: 8.8.8.8
      port: 53
      protocol: udp   # "udp" (default) or "tcp"
    - host: 8.8.4.4
      port: 53
    - host: 1.1.1.1
      port: 53
    - host: 1.0.0.1
      port: 53

binary_sensor:
  - platform: online
    name: "Online"
    icon: mdi:web
    device_class: connectivity
```

## Options

### Global platform options (`online:`)

| Property          | Type                  | Default                              | Description                                        |
| ----------------- | --------------------- | ------------------------------------ | -------------------------------------------------- |
| `targets`         | list of targets       | Google & Cloudflare DNS via UDP/53   | Connectivity targets (see Target options below)    |
| `update_interval` | duration              | `30s`                                | Poll interval                                      |
| `timeout`         | duration              | `3s`                                 | Default connection/response timeout per target     |

### Target options

Each entry in the `targets` list supports the following properties:

| Property   | Type   | Default    | Description                                                                         |
| ---------- | ------ | ---------- | ----------------------------------------------------------------------------------- |
| `host`     | string | **Required** | Hostname or IP address to check                                                   |
| `port`     | number | **Required** | Port number                                                                        |
| `protocol` | string | `udp`      | Transport protocol: `udp` sends a minimal DNS query; `tcp` opens a TCP connection  |
| `timeout`  | duration | from `online` | Per-target timeout override                                                    |

### Binary sensor options (`binary_sensor`)

| Property          | Type              | Default       | Description                                                     |
| ----------------- | ----------------- | ------------- | --------------------------------------------------------------- |
| `platform`        | string            | **Required**  | Must be `online`                                                |
| `name`            | string            | **Required**  | Display name                                                    |
| `targets`         | list of targets   | from `online` | Override the targets list for this sensor                       |
| `update_interval` | duration          | from `online` | Override poll interval                                          |
| `timeout`         | duration          | from `online` | Override default per-target timeout                             |
| `icon`            | string            | `mdi:web`     | Home Assistant icon                                             |
| `device_class`    | string            | `connectivity`| Home Assistant device class                                     |

## Example: mixed UDP and TCP targets

```yaml
online:
  timeout: 5s

binary_sensor:
  - platform: online
    name: "Internet"
    targets:
      - host: 8.8.8.8
        port: 53
        protocol: udp
      - host: example.com
        port: 443
        protocol: tcp
        timeout: 10s
```

