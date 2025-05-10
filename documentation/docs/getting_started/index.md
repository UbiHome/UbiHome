---
hide:
  - navigation
  - toc
---

# Getting Started

For now[^1] a single executable is provided. You can download them from the [GitHub Releases page](https://github.com/UbiHome/UbiHome/releases) or use the direct links below.


<!-- x-release-please-start-version -->

=== "Linux"

    ## Download

    | Devices         | Download                                                                                                          | Target                       |
    | -------------- | ----------------------------------------------------------------------------------------------------------------- | ---------------------------- |
    | Any Linux         | [TAR Link](https://github.com/UbiHome/UbiHome/releases/download/v0.8.10/ubihome-Linux-musl-x86_64.tar.gz)    | x86_64-unknown-linux-musl |
    | Raspberry Pi 3  | [TAR Link](https://github.com/UbiHome/UbiHome/releases/download/v0.8.10/ubihome-Linux-musleabi-armv7.tar.gz) | armv7-unknown-linux-musleabi |
    | Raspberry Pi Zero 2 | [TAR Link](https://github.com/UbiHome/UbiHome/releases/download/v0.8.10/ubihome-Linux-musleabi-arm.tar.gz) | arm-unknown-linux-musleabi |


    ## Installation


    1. Download and extract the archive and place the ubihome executable in a directory of your choice.

        ```bash
        curl -L -o ubihome.tar.gz https://github.com/UbiHome/UbiHome/releases/download/v0.8.10/ubihome-Linux-musleabi-armv7.tar.gz
        tar xvzf ./ubihome.tar.gz
        ```

    2. Edit the configuration file `config.yaml` in the same directory as the executable. It could contain the following:

        ```yaml title="config.yaml"
        --8<-- "getting_started/linux.yml"
        ```

    3.  Try it out:

        ``` bash
        ./ubihome run
        UbiHome - 0.8.10
        LogDirectory: /home/codespace/.local/share
        Config file path: /workspaces/ubihome/config.yaml
        Binary Sensor 'bluetooth_connected' output: false
        Sensor 'ram_usage' output: 38.3144
        Button 'my_button' pressed.
        Command executed successfully with no output.
        # End the process with ctrl+c
        ```

    4.  You should be able to see your device in Home Assistant now.

    5.  To persistently run the executable install it as a service:

        ``` bash
        sudo ./ubihome install
        # The CLI will ask you where to install it. (You can just hit enter to install it in the default location)
        ? Where do you want to install UbiHome? (/usr/bin/ubihome)
        ```

        > If you do this more often you can add the --install-path flag to the command to specify the path for the installation. Instead of the CLI asking for it.


    ## Uninstallation

    If you want to uninstall UbiHome you can run the following command:

    ```bash
    ./ubihome uninstall
    ```


=== "Windows"

    ## Download

    | Device         | Download                                                                                                          | Target                       |
    | -------------- | ----------------------------------------------------------------------------------------------------------------- | ---------------------------- |
    | Windows 11        | [ZIP Link](https://github.com/UbiHome/UbiHome/releases/download/v0.8.10/ubihome-Windows-msvc-x86_64.zip)    | x86_64-pc-windows-msvc |

    ## Installation

    1. Download and extract the archive and place the ubihome executable in a directory of your choice.

    ```powershell
    Invoke-WebRequest -OutFile ubihome.zip -Uri https://github.com/UbiHome/UbiHome/releases/download/v0.8.10/ubihome-Windows-msvc-x86_64.zip
    Expand-Archive -Force ubihome.zip ./
    ```

    2. Edit the configuration file `config.yaml` in the same directory as the executable. It could contain the following:

    ```yaml title="config.yaml"
    --8<-- "getting_started/windows.yml"
    ```

    3.  Try it out:

        ``` bash
        ./ubihome.exe run
        UbiHome - 0.8.10
        LogDirectory: /home/codespace/.local/share
        Config file path: /workspaces/UbiHome/config.yaml
        Binary Sensor 'bluetooth_connected' output: false
        Sensor 'ram_usage' output: 38.3144
        Button 'my_button' pressed.
        Command executed successfully with no output.
        # End the process with ctrl+c
        ```

    4.  You should be able to see your device in Home Assistant now.

    5.  To persistently run the executable install it as a service:

        > Press ++windows+x+a++ for the admin shell.

        ``` bash
        ./ubihome.exe install
        # The CLI will ask you where to install it. (You can just hit enter to install it in the default location)
        ? Where do you want to install UbiHome? (C:\Program Files\ubihome)
        ```

        > If you do this more often you can add the --install-path flag to the command to specify the path for the installation. Instead of the CLI asking for it.


    ## Uninstallation

    If you want to uninstall UbiHome you can run the following command:

    ```powershell
    ./ubihome.exe uninstall
    ```


=== "MacOS"

    ## Download

    | Device         | Download                                                                                                          | Target                       |
    | -------------- | ----------------------------------------------------------------------------------------------------------------- | ---------------------------- |
    | MacOS        | [TAR Link](https://github.com/UbiHome/UbiHome/releases/download/v0.8.10/ubihome-macOS-x86_64.tar.gz)    | x86_64-apple-darwin |

    ## Installation

    > Not yet tested. But you can try it out and create an issue if it doesn't work.

    
    1. Download and extract the archive and place the ubihome executable in a directory of your choice.

        ```bash
        curl -L -o ubihome.tar.gz https://github.com/UbiHome/UbiHome/releases/download/v0.8.10/ubihome-Linux-musleabi-armv7.tar.gz
        tar xvzf ./ubihome.tar.gz
        ```

    2. Edit the configuration file `config.yaml` in the same directory as the executable. It could contain the following:

        ```yaml title="config.yaml"
        --8<-- "getting_started/macos.yml"
        ```

    3.  Try it out:

        ``` bash
        ./ubihome run
        UbiHome - 0.8.10
        LogDirectory: /home/codespace/.local/share
        Config file path: /workspaces/ubihome/config.yaml
        Binary Sensor 'bluetooth_connected' output: false
        Sensor 'ram_usage' output: 38.3144
        Button 'my_button' pressed.
        Command executed successfully with no output.
        # End the process with ctrl+c
        ```

    4.  You should be able to see your device in Home Assistant now.

    5.  To persistently run the executable install it as a service:

        ``` bash
        sudo ./ubihome install
        # The CLI will ask you where to install it. (You can just hit enter to install it in the default location)
        ? Where do you want to install UbiHome? (/usr/bin/ubihome)
        ```

        > If you do this more often you can add the --install-path flag to the command to specify the path for the installation. Instead of the CLI asking for it.

    ## Uninstallation

    If you want to uninstall UbiHome you can run the following command:

    ```bash
    ./ubihome uninstall
    ```


<!-- x-release-please-end -->





[^1]: This will change in the future to allow for custom compilation for modular builds and custom extensions.
