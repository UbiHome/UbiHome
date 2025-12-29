# Native Api

This uses the same API as ESPHome.

## Basic Configuration

```yaml
# This makes the native api available unsecured
api:
```

```yaml
api:
  # Optional: Default Port is 6053
  port: 6053
  # Secure your API:
  encryption_key: 'copy_your_generated_key_here'
```

{{ encryption_key_generator() }}

Similar to ESPHome:

- [ESPHome API](https://esphome.io/components/api.html)
