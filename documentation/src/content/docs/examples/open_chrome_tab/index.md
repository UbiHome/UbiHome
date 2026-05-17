---
title: "Open a new tab in chrome"
---

# Open a new tab in chrome


## Windows


```yaml
shell:

button:
  - platform: shell
    name: 'Open new Chrome Tab'
    command: "start chrome \"https://ubihome.github.io/\""
```

## Linux


```yaml
shell:

button:
  - platform: shell
    name: 'Open new Chrome Tab'
    command: "chrome \"https://ubihome.github.io/\""
```

## Related documentation

- Component: [Button](../../features/components/button)
- Platform: [Shell](../../features/platforms/shell)
