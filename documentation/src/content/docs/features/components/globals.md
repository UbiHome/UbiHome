---
title: 'Globals'
---

Global variables store a single value that [actions](/features/components/actions/)
can update at runtime with `globals.set`. `globals` is built into UbiHome.

```yaml
globals:
  - id: door_open
    type: bool
    initial_value: false
```

## Attributes

| Property        | Description                                             | Example   |
| --------------- | ------------------------------------------------------- | --------- |
| `id`            | Identifier used to reference the global.                | `door_open` |
| `type`          | Value type: `bool`, `int`, `float` or `string`.         | `bool`    |
| `initial_value` | Value the global starts with. Defaults per type.        | `false`   |

`initial_value` and `globals.set` accept the value as a plain YAML scalar
matching `type` (e.g. an unquoted `false` or `42`) or as a quoted string
(e.g. `'false'`) — both are reconciled against the global's declared `type`.

Globals are written via the `globals.set` [action](/features/components/actions/),
which takes `id`/`value` arguments instead of a single id:

```yaml
- globals.set:
    id: door_open
    value: true
```

A `lambda` [action](/features/components/actions/) can instead read/write a
global from JavaScript with `id()`/`set_global()`.

A global can also be read as the state of a
[template](/features/platforms/template/) switch or number using `id()` in a
`lambda`:

```yaml
switch:
  - platform: template
    name: 'Relay'
    lambda: |-
      return id(door_open)
```

Similar to ESPHome: [Globals](https://esphome.io/components/globals/)

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/features/components/actions/">Triggers and Actions</a>
  <a href="/features/platforms/template/">Template</a>
</div>
