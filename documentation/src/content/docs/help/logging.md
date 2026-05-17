---
title: 'Logging'
---

If you need more information from the program you can enable logging.
This is done by adding the `logger` section to your configuration file:

```yaml
logger:
  level: INFO
```

To switch log levels per component use the `logs` section. For example to enable `debug` logging for only the `ubihome_api` component, you would add the following to your configuration file:

```yaml
logger:
  level: INFO
  logs:
    ubihome_api: debug
```

You can also set the log directory to a custom location. The default logging locations are document on the [logger documentation page](/features/utilities/logger/).

```yaml
logger:
  level: INFO
  directory: ./logs
```
