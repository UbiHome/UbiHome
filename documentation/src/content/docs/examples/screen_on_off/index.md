---
title: 'Turn Raspberry screen on or off'
description: 'Control a Raspberry Pi display power state with UbiHome.'
---

Control the screen power state of a Raspberry Pi using the `vcgencmd` command or `wlr-randr` for Wayland.

:::note
To see which one you are using run `loginctl show-session 1 | grep "Desktop"`. It will display:

| Output                     | Technology                                              | Configuration Paths                     |
| -------------------------- | ------------------------------------------------------- | --------------------------------------- |
| `Desktop=lightdm-xsession` | X11                                                     | `/etc/xdg/lxsession/LXDE-pi/`           |
| `Desktop=LXDE-pi-x`        | X11                                                     |                                         |
| `Desktop=LXDE-pi-wayfire`  | Wayland [wayfire](https://github.com/WayfireWM/wayfire) |                                         |
| `Desktop=LXDE-pi-labwc`    | Wayland [labwc](https://github.com/labwc/labwc)         | `/etc/xdg/labwc/` or `~/.config/labwc/` |

:::

## Wayland

Try it out before by running `wlr-randr --output <display> --off` to turn off the screen and `wlr-randr --output <display> --on` to turn it back on.

> Run `kmsprint` to show all connected displays.

> Set `XDG_RUNTIME_DIR=/run/user/<UID>` (find UID by running `loginctl`) and `WAYLAND_DISPLAY=<identifier>` (find out by running `ls /run/user/111/ | grep wayland`).

```yaml
shell:

switch:
  - platform: shell
    name: 'Screen'
    id: screen
    command_on: 'wlr-randr --output HDMI-A-1 --on'
    command_off: 'wlr-randr --output HDMI-A-1 --off'
    command_state: |-
      if wlr-randr --output HDMI-A-1 | grep -q "Enabled: yes"; then
          echo true
      else
          echo false
      fi
```

## X11

Try it out before by running `vcgencmd display_power 0` to turn off the screen and `vcgencmd display_power 1` to turn it back on.

> Be sure to have [`dtoverlay=vc4-fkms-v3d`](https://github.com/raspberrypi/firmware/issues/1224) activated.

```yaml
shell:

switch:
  - platform: shell
    name: 'Screen'
    id: screen
    command_on: 'vcgencmd display_power 1'
    command_off: 'vcgencmd display_power 0'
    command_state: |-
      if vcgencmd display_power | grep -q "display_power=1"; then
          echo true
      else
          echo false
      fi
```

## Related documentation

- Component: [Switch](/features/entities/switch/)
- Platform: [Shell](/features/platforms/shell/)
