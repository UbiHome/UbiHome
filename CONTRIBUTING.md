# Contributing to UbiHome

We welcome any contributions to the UbiHome suite of code and documentation!

Please follow the semantic commit message format for all commits. 

## Development

Just use the devcontainer setup.

```bash
sudo apt install -y libdbus-1-dev pkg-config
```

Vendored:
```
sudo apt-get install -y musl-tools
```


## Current Pitfalls

Logs are in `C:\Windows\System32\config\systemprofile\AppData\Local` as the service is running as `SYSTEM` user.


## Process

- Validate config.yaml
  - Before Build (not yet implemented)
  - During Runtime
