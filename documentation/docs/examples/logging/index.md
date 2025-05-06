# Logging

If you need more information from the program you can enable logging. 
This is done by adding the `logger` section to your configuration file:

```yaml
--8<-- "examples/logging/base_logging.yml"
```

To switch log levels per component, to enable logging for only the `ubihome_api` component, you would add the following to your configuration file:

```yaml
--8<-- "examples/logging/advanced_logging.yml"
```

You can also set the log directory to a custom location. By default, the logs are stored in the `~/.local/share/ubihome/logs` directory.
For more information about the component refer to the [logger documentation](../../features/utilities/logger.md).