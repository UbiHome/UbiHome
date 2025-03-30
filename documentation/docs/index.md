---
hide:
  - navigation
  - toc
---

# Welcome to OSHome!

OSHome is a single executable that allows you to integrate any device running an OS into your smart home. 
It is designed to be lightweight and easy to use - similar to ESPHome.

- Execute a command on a device based on a trigger in home assistant. 
- Monitor the status of a device with a custom command. 
- Integrate all of your _one off python scripts^TM^_ without thinking about connectivity or setting up yet another service.

Future plans:
- Monitor connected bluetooth devices and maybe even proxy them to home assistant.

## Examples

Monitor a device with a custom command:
```yaml

```


## Roadmap

- [ ] Auto installation
  - [ ] Windows (https://github.com/mullvad/windows-service-rs) https://medium.com/@aleksej.gudkov/rust-windows-service-example-building-a-windows-service-in-rust-907be67d2287
  - [ ] Linux Service
    - debian (https://github.com/kornelski/cargo-deb/blob/fc34c45fafc3904cadf652473ff7e9e0344c605c/systemd.md)
  - [ ] MacOS?
- [ ] Templates and Services
- [ ] Additional Components:
  - [ ] HTTP and Web Enpoint
  - [ ] BLE (https://github.com/deviceplug/btleplug)
  - [ ] Bluetooth Proxy (https://esphome.io/components/bluetooth_proxy.html)

- [ ] Custom compilation for modular builds and custom extensions.
- [ ] Homeassistant Native API
- [ ] CLI for automatic generation of executables
- [ ] Builder Component similar to ESP Home
- [ ] Self update (https://github.com/jaemk/self_update)


Rust clippy: 
https://github.com/rust-lang/rust-clippy



