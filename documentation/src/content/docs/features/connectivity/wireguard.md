---
title: 'WireGuard'
tags: ['networking', 'wireguard']
---

The `wireguard` platform opens a built-in [WireGuard](https://www.wireguard.com/) tunnel to your home network entirely in userspace — no root, no `wg`/`tun` device, and no changes to the operating system's routing table. Unlike the [ESPHome WireGuard component](https://esphome.io/components/wireguard/) (whose configuration this mirrors), only the connections listed under `forwards` are sent through the tunnel, so it coexists with a corporate VPN or any other networking on the device.

This is useful when UbiHome runs on a device on the public internet (or a managed corporate network) but its MQTT broker and/or Home Assistant live at home:

- **inbound** forwards let a peer at home (e.g. Home Assistant) reach a service UbiHome exposes, such as the [API](/features/connectivity/api/) — the tunnel listens on your tunnel IP and bridges connections to a local port.
- **outbound** forwards let UbiHome reach a service at home, such as an [MQTT](/features/connectivity/mqtt/) broker — a local listener is opened and traffic is tunneled to the remote address.

The peer (your home WireGuard server) must have this device's public key and tunnel IP registered.

```yaml
wireguard:
  address: 10.9.0.2 # this device's address inside the tunnel
  netmask: 255.255.255.0 # optional, default 255.255.255.255
  private_key: <this device's base64 private key>
  peer_endpoint: home.example.dyndns.net
  peer_port: 51820 # optional, default 51820
  peer_public_key: <home server's base64 public key>
  peer_preshared_key: <optional base64 pre-shared key>
  peer_persistent_keepalive: 25s # recommended when behind NAT
  peer_allowed_ips:
    - 0.0.0.0/0 # networks reachable through the tunnel
  forwards:
    # Home Assistant at home -> this device's API server (inbound):
    - direction: inbound
      listen: '10.9.0.2:6053' # must be the tunnel `address` IP
      remote: '127.0.0.1:6053' # the local API server
    # This device -> MQTT broker at home (outbound):
    - direction: outbound
      listen: '127.0.0.1:1883' # local address to bind
      remote: '10.0.0.10:1883' # broker on the home network
```

Point the connections you want tunneled at the forward's local side — no other configuration changes are needed. For the example above:

```yaml
api:
  port: 6053 # reached from home via the inbound forward on 10.9.0.2:6053

mqtt:
  broker: '127.0.0.1' # the outbound forward tunnels this to 10.0.0.10:1883
  port: 1883
```

## Options

| Option                      | Description                                                                                                    |
| --------------------------- | -------------------------------------------------------------------------------------------------------------- |
| `address`                   | This device's IPv4 address inside the tunnel, e.g. `10.9.0.2`.                                                  |
| `netmask`                   | Network mask for `address`. Defaults to `255.255.255.255`.                                                      |
| `private_key`               | Base64 WireGuard private key of this device.                                                                    |
| `peer_endpoint`             | Hostname or IP of the home WireGuard server. Re-resolved on each connection.                                    |
| `peer_port`                 | UDP port of the peer. Defaults to `51820`.                                                                      |
| `peer_public_key`           | Base64 public key of the home WireGuard server.                                                                 |
| `peer_preshared_key`        | Optional base64 pre-shared key.                                                                                 |
| `peer_persistent_keepalive` | Keep-alive interval. Disabled by default; set e.g. `25s` when this device is behind NAT so inbound can reach it. |
| `peer_allowed_ips`          | Networks reachable through the tunnel, in CIDR notation (e.g. `10.9.0.0/24` or `0.0.0.0/0`).                    |
| `update_interval`           | Status-entity refresh interval. Defaults to `10s`.                                                             |
| `forwards[].direction`      | `inbound` (home peer → local service) or `outbound` (local listener → home service).                           |
| `forwards[].listen`         | Address to accept connections on. For `inbound` this must be the tunnel `address` IP; for `outbound` a local IP. |
| `forwards[].remote`         | Address to forward to. For `inbound` the local service; for `outbound` the address on the home network.        |

The platform exposes a `WireGuard Status` [binary sensor](/features/entities/binary_sensor/) (connectivity) and a `WireGuard Handshake Age` diagnostic sensor so you can monitor the tunnel's health.

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/features/connectivity/api/">API</a>
  <a href="/features/connectivity/mqtt/">MQTT</a>
  <a href="/features/entities/binary_sensor/">Binary Sensor</a>
</div>
