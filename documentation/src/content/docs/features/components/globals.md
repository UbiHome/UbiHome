---
title: 'Globals'
---

Global variables store a single value that [actions](/features/components/actions/)
can update at runtime with `globals.set`. `globals` is built into UbiHome.

```yaml
globals:
  - id: door_open
    type: bool
    initial_value: 'false'
    # restore_value is accepted for ESPHome compatibility but not yet persisted.
    restore_value: false
```

## Attributes

| Property        | Description                                             | Example   |
| --------------- | ------------------------------------------------------- | --------- |
| `id`            | Identifier used to reference the global.                | `door_open` |
| `type`          | Value type: `bool`, `int`, `float` or `string`.         | `bool`    |
| `initial_value` | Value the global starts with. Defaults per type.        | `'false'` |
| `restore_value` | Accepted for compatibility; persistence is not implemented yet. | `false` |

Globals are written via the `globals.set` [action](/features/components/actions/).
A `bool` global can also be read as the state of a
[template switch](/features/platforms/template/) using a `globals.get` lambda —
written in YAML, not code:

```yaml
switch:
  - platform: template
    name: 'Relay'
    lambda:
      globals.get: door_open
```

C++ lambdas are not supported, so globals cannot yet be read back in arbitrary
expressions.

Similar to ESPHome: [Globals](https://esphome.io/components/globals/)

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/features/components/actions/">Triggers and Actions</a>
  <a href="/features/platforms/template/">Template</a>
</div>
