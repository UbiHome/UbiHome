---
hide:
  - navigation
  - toc
---

# Getting Started

For now[^1] a single executable is provided. You can download them from the [GitHub Releases page](https://github.com/DanielHabenicht/OSHome/releases) or use the direct links below.


<!-- x-release-please-start-version -->

=== "Linux"

    ## Download 

    | Device         | Download                                                                                                          | Target                       |
    | -------------- | ----------------------------------------------------------------------------------------------------------------- | ---------------------------- |
    | Default        | [TAR Link](https://github.com/DanielHabenicht/OSHome/releases/download/v0.2.2/oshome-Linux-musl-x86_64.tar.gz)    | x86_64-unknown-linux-musl |
    | Raspberry Pi 3 | [TAR Link](https://github.com/DanielHabenicht/OSHome/releases/download/v0.2.2/oshome-Linux-musleabi-armv7.tar.gz) | armv7-unknown-linux-musleabi |

    ## Installation


    1. Download and extract the archive and place the oshome executable in a directory of your choice.

        ```bash
        curl -L -o oshome.tar.gz https://github.com/DanielHabenicht/OSHome/releases/download/v0.2.2/oshome-Linux-musleabi-armv7.tar.gz
        tar xvzf ./oshome.tar.gz
        ./oshome
        ```

    2. Create a configuration file in the same directory as the executable. The configuration file should be named `config.yaml` and should contain the following:

        ```yaml
        --8<-- "getting_started/linux.yml"
        ```

    3.  Try it out:

        ``` bash
        ./oshome run
        Starting OSHome - 0.2.2
        LogDirectory: /home/codespace/.local/share
        Config file path: /workspaces/OSHome/config.yaml
        Binary Sensor 'bluetooth_connected' output: false
        Sensor 'ram_usage' output: 38.3144
        Button 'my_button' pressed.
        Command executed successfully with no output.
        # End the process with ctrl+c
        ```

    4.  You should be able to see your device in Home Assistant now.

    5.  To persistently run the executable install it as a service:

        ``` bash
        sudo ./oshome install
        # The CLI will ask you where to install it. (You can just hit enter to install it in the default location)
        ? Where do you want to install OSHome? (/usr/bin/oshome)
        ```

        > If you do this more often you can add the --install-path flag to the command to specify the path for the installation. Instead of the CLI asking for it.


    ## Uninstallation

    If you want to uninstall OSHome you can run the following command:

    ```bash
    ./oshome uninstall
    ```


=== "Windows"

    | Device         | Download                                                                                                          | Target                       |
    | -------------- | ----------------------------------------------------------------------------------------------------------------- | ---------------------------- |
    | Windows 11        | [ZIP Link](https://github.com/DanielHabenicht/OSHome/releases/download/v0.2.2/oshome-Windows-msvc-x86_64.zip)    | x86_64-pc-windows-msvc |

    ## Installation

    1. Download and extract the archive and place the oshome executable in a directory of your choice.

    ```powershell
    Invoke-WebRequest -OutFile oshome.zip -Uri https://github.com/DanielHabenicht/OSHome/releases/download/v0.2.2/oshome-Windows-msvc-x86_64.zip
    Expand-Archive -Force oshome.zip ./
    ./oshome.exe
    ```

    2. Create a configuration file in the same directory as the executable. The configuration file should be named `config.yaml` and should contain the following:

    ```yaml
    --8<-- "getting_started/windows.yml"
    ```

    3.  Try it out:

        ``` bash
        ./oshome.exe run
        Starting OSHome - 0.2.2
        LogDirectory: /home/codespace/.local/share
        Config file path: /workspaces/OSHome/config.yaml
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
        ./oshome.exe install
        # The CLI will ask you where to install it. (You can just hit enter to install it in the default location)
        ? Where do you want to install OSHome? (C:\Program Files\oshome)
        ```

        > If you do this more often you can add the --install-path flag to the command to specify the path for the installation. Instead of the CLI asking for it.


    ## Uninstallation

    If you want to uninstall OSHome you can run the following command:

    ```powershell
    ./oshome.exe uninstall
    ```


=== "MacOS"

    | Device         | Download                                                                                                          | Target                       |
    | -------------- | ----------------------------------------------------------------------------------------------------------------- | ---------------------------- |
    | MacOS        | [TAR Link](https://github.com/DanielHabenicht/OSHome/releases/download/v0.2.2/oshome-macOS-x86_64.tar.gz)    | x86_64-apple-darwin |


    3.  Run the executable with the following command:

        === "Linux"

            ``` bash
            sudo ./oshome install
            # The CLI will ask you where to install it. (You can just hit enter to install it in the default location)
            ? Where do you want to install OSHome? (/usr/bin/oshome)
            ```

        === "Windows"

            Press ++windows+x+a++ for the admin shell and run the following command:

            ``` powershell
            ./oshome.exe install
            # The CLI will ask you where to install it. (You can just hit enter to install it in the default location)
            ? Where do you want to install OSHome? (C:\Program Files\oshome)
            ```

        > If you do this more often you can add the --install-path flag to the command to specify the path for the installation. Instead of the CLI asking for it.

    4.  After the installation is complete you should be able to see your device in Home Assistant.:


<!-- x-release-please-end -->





[^1]: This will change in the future to allow for custom compilation for modular builds and custom extensions.
