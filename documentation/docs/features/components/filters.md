# Filters

On any entity you can add filters to modify the entity's data. 

```yaml
sensor:
  - platform: shell
    name: "RAM Usage"
    command: echo 1.123345
    filters:
      - round: 2

# Entity will be shown as 1.12
```

Filters are specific to the entity for now.
Look at each [entity](./index.md#entities) for the available filters.