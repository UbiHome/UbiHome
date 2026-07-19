---
title: 'Common Properties'
sidebar:
  order: 2
---

These properties are available on all components.

| Property | Type   | Required | Description                                                  |
| -------- | ------ | -------- | ------------------------------------------------------------ |
| `name`   | string | See note | Friendly display name for the component.                     |
| `id`     | string | See note | Unique identifier. If omitted, one is generated from `name`. |
| `icon`   | string | No       | Icon used in Home Assistant and related UIs.                 |

At least one of `name` or `id` must be provided; supplying both is also allowed.

A component configured with only an `id` (no `name`) is treated as **internal**:
it still participates in internal wiring such as [filters](/features/components/filters),
[actions](/features/components/actions) and state routing, but it is **not exposed**
to connectivity components like the API, MQTT or the web server.

Additional attributes such as `device_class` or `entity_category` are component-specific and documented on each [entities](/features/#entities) page.

Also check the page for your [chosen platform](/features/platforms) for the full list of required and optional options implemented.
