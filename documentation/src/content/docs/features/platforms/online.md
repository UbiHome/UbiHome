---
title: 'Online'
description: 'Check if the device can reach the internet or a router'
tags:
  - linux
  - macos
  - windows
---

The online platform provides a binary sensor that reports whether the device can reach any of a list of connectivity targets. By default it queries four well-known DNS servers, which is lightweight and requires no persistent connection.

## Configuration

```yaml
online:
  update_interval: 30s
  timeout: 3s
  # Defaults to the four DNS servers below; override to use your own targets.
  targets:
    - host: 8.8.8.8 # protocol defaults to "dns" (always port 53)
    - host: 8.8.4.4
    - host: 1.1.1.1
    - host: 1.0.0.1

binary_sensor:
  - platform: online
    name: 'Online'
    icon: mdi:web
    device_class: connectivity
```

## Options

### Global platform options (`online:`)

| Property          | Type            | Default                          | Description                                     |
| ----------------- | --------------- | -------------------------------- | ----------------------------------------------- |
| `targets`         | list of targets | Google & Cloudflare DNS on `:53` | Connectivity targets (see Target options below) |
| `update_interval` | duration        | `30s`                            | Poll interval                                   |
| `timeout`         | duration        | `3s`                             | Default connection/response timeout per target  |

#### Target options

Each entry in the `targets` list supports the following properties:

| Property   | Type     | Default       | Description                                                                                |
| ---------- | -------- | ------------- | ------------------------------------------------------------------------------------------ |
| `host`     | string   | **Required**  | Hostname or IP address to check                                                            |
| `protocol` | string   | `dns`         | Probe type: `dns` sends a minimal DNS query (always port 53); `tcp` opens a TCP connection |
| `port`     | number   | —             | Port number. Required for `tcp`; optional for `dns` (defaults to 53)                       |
| `timeout`  | duration | from `online` | Per-target timeout override                                                                |

## Actions

The binary sensor exposes the standard [triggers and actions](/features/components/actions/): `on_press` fires when connectivity is restored and `on_release` when it is lost.

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/features/entities/binary_sensor/">Binary Sensor</a>
  <a href="/examples/reboot_on_connectivity_loss/">Reboot on internet outage</a>
</div>
