# Turn Raspberry screen on or off

Control the screen power state of a Raspberry Pi using the `vcgencmd` command or `wlr-randr` for Wayland.

!!! info
To see which one you are using run `loginctl show-session 1 | grep "Desktop"` It will display the following for each:

    | Output                        | Technology                                              | Configuration Paths                     |
    | ----------------------------- | ------------------------------------------------------- | --------------------------------------- |
    | `Desktop=lightdm-xsession`    | X11                                                     | `/etc/xdg/lxsession/LXDE-pi/`           |
    | `Desktop=LXDE-pi-x`           | X11                                                     |                                         |
    | `Desktop=LXDE-pi-wayfire`     | Wayland [wayfire](https://github.com/WayfireWM/wayfire) |                                         |
    | `Desktop=LXDE-pi-labwc`       | Wayland [labwc](https://github.com/labwc/labwc)         | `/etc/xdg/labwc/` or `~/.config/labwc/` |

=== "Raspberry Pi"

    ## Wayland

    Try it out before by running `wlr-randr --output <display> --off` to turn off the screen and `wlr-randr --output <display> --on` to turn it back on.

    > Run `kmsprint` to show all connected displays.

    > Set `XDG_RUNTIME_DIR=/run/user/<UID>` (find UID by running `loginctl`) and `WAYLAND_DISPLAY=<identifier>` (find out by running `ls /run/user/111/ | grep wayland`

    ```yaml
    --8<-- "examples/screen_on_off/raspberry_wayland.yml"
    ```

    ## X11

    Try it out before by running `vcgencmd display_power 0` to turn off the screen and `vcgencmd display_power 1` to turn it back on.

    > Be sure to have [`dtoverlay=vc4-fkms-v3d`](https://github.com/raspberrypi/firmware/issues/1224) activated.

    ```yaml
    --8<-- "examples/screen_on_off/raspberry_x11.yml"
    ```
