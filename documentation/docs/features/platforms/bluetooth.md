# Bluetooth

Scans for Bluetooth devices and reports them to HomeAssistant via its native API.

> The Bluetooth Proxy only support proxying for now.

```yaml title="Base Configuration"
ubihome:
  name: Test Bluetooth Proxy

api:

bluetooth_proxy:
```

TODOs:

- [ ] Allow pausing the bluetooth proxy (e.g. while media is played)


Similar to ESPHome:

- [https://esphome.io/components/bluetooth_proxy.html](https://esphome.io/components/bluetooth_proxy.html)