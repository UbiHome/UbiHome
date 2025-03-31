# Contributing to OSHome

We welcome any contributions to the OSHome suite of code and documentation!

Please follow the semantic commit message format for all commits. 

## Development

## Windows

```powershell

winget install Rustlang.Rustup Rustlang.Rust.GNU Rustlang.Rust.MSVC Google.Protobuf
```

## Linux

Just use the devcontainer setup.

```bash
sudo apt install -y protobuf-compiler
```


## Current Pitfalls

Logs are in `C:\Windows\System32\config\systemprofile\AppData\Local` as the service is running as `SYSTEM` user.


## Process

- Validate config.yaml
  - Before Build (not yet implemented)
  - During Runtime
