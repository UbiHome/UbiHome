---
title: 'Status'
description: 'Report whether the UbiHome instance itself is up and running'
tags:
  - linux
  - macos
  - windows
---

The status platform provides a binary sensor that is `on` for as long as the UbiHome process is running. Unlike [Online](/features/platforms/online/), it does not check external connectivity - it simply reflects that this UbiHome instance started up successfully.

## Configuration

Enable the platform:

```yaml
status:
```

```yaml
binary_sensor:
  - platform: status
    name: 'Living Room Status'
    icon: mdi:check-network-outline
    device_class: connectivity
```

## Options

This platform has no global (`status:`) options - only the standard [Binary Sensor entity](/features/entities/binary_sensor/) options apply.

## Actions

The binary sensor exposes the standard [triggers and actions](/features/components/actions/): `on_press` fires when UbiHome starts up.

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/features/entities/binary_sensor/">Binary Sensor</a>
  <a href="/features/platforms/online/">Online</a>
</div>
