# OSHome
Analog to ESPHome OSHome is a system which allows you to integrate any device running an OS into your smart home.



# Roadmap
- [] Templates and Services
- [ ] CLI for automatic generation of executables
- [ ] Builder Component similar to ESP Home
- [ ] Self update (https://github.com/jaemk/self_update)
- [ ] Auto installation
  - [ ] Windows (https://github.com/mullvad/windows-service-rs) https://medium.com/@aleksej.gudkov/rust-windows-service-example-building-a-windows-service-in-rust-907be67d2287
  - [ ] Linux Service
    - debian (https://github.com/kornelski/cargo-deb/blob/fc34c45fafc3904cadf652473ff7e9e0344c605c/systemd.md)
  - [ ] MacOS?

- [ ] Additional Components:
  - [ ] HTTP and Web Enpoint
  - [ ] BLE (https://github.com/deviceplug/btleplug)
  - [ ] Bluetooth Proxy (https://esphome.io/components/bluetooth_proxy.html)

- [ ] Homeassistant Native API


Rust clippy: 
https://github.com/rust-lang/rust-clippy



## Development

## Windows

```powershell

winget install Rustlang.Rustup Rustlang.Rust.GNU Rustlang.Rust.MSVC
```

## Linux

Just use the devcontainer setup.


## Current Pitfalls

Logs are in `C:\Windows\System32\config\systemprofile\AppData\Local` as the service is running as `SYSTEM` user.