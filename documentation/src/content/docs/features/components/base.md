---
title: "Common Component Properties"
---

# Common Component Properties

These properties are available on all components.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | string | Yes | Friendly display name for the component. |
| `id` | string | No | Unique identifier. If omitted, one is generated from `name`. |
| `icon` | string | No | Icon used in Home Assistant and related UIs. |

Additional attributes such as `device_class` or `entity_category` are component-specific and documented on each component page.

