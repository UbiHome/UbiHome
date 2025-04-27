# Test with

```bash
sudo apt-get install avahi-utils

avahi-browse -at

```

In Homeassistant add the following to `configuration.yaml`: 

```yaml
logger:
  logs:
    zeroconf: debug
```

