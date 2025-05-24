# Turn Raspberry screen on or off

Control the screen power state of a Raspberry Pi using the `vcgencmd` command or `wlr-randr` for Wayland.

=== "Raspberry Pi"
    
    ## Wayland

    Try it out before by running `wlr-randr --output HDMI-A-1 --off` to turn off the screen and `wlr-randr --output HDMI-A-1 --on` to turn it back on.

    
    
    ```yaml
    --8<-- "examples/screen_on_off/raspberry_wayland.yml"
    ```

    ## X11 

    Try it out before by running `vcgencmd display_power 0` to turn off the screen and `vcgencmd display_power 1` to turn it back on.

    > Be sure to have [`dtoverlay=vc4-fkms-v3d`](https://github.com/raspberrypi/firmware/issues/1224) activated.

    ```yaml
    --8<-- "examples/screen_on_off/raspberry_x11.yml"
    ```
