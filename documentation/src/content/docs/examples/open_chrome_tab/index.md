---
title: 'Open a new tab in chrome'
description: 'Launch Chrome and open a new tab with a button action.'
---

## Windows

```yaml
shell:

button:
  - platform: shell
    name: 'Open new Chrome Tab'
    command: 'start chrome "https://ubihome.github.io/"'
```

## Linux

```yaml
shell:

button:
  - platform: shell
    name: 'Open new Chrome Tab'
    command: 'chrome "https://ubihome.github.io/"'
```

## Related documentation

- Component: [Button](/features/entities/button/)
- Platform: [Shell](/features/platforms/shell/)
