---
title: 'Common Properties'
sidebar:
  order: 2
---

These properties are available on all components.

| Property | Type   | Required | Description                                                  |
| -------- | ------ | -------- | ------------------------------------------------------------ |
| `name`   | string | Yes      | Friendly display name for the component.                     |
| `id`     | string | No       | Unique identifier. If omitted, one is generated from `name`. |
| `icon`   | string | No       | Icon used in Home Assistant and related UIs.                 |

Additional attributes such as `device_class` or `entity_category` are component-specific and documented on each [entities](/features/#entities) page.

Also check the page for your [chosen platform](/features/platforms) for the full list of required and optional options implemented.
