---
hide:
  - navigation
---

# Getting Started

For now[^1] a single executable is provided. You can download them from the releases page.

## Donwload

### Linux Based

<!-- x-release-please-start-version -->
| Device         | Download                                                                                                          | Target                       |
| -------------- | ----------------------------------------------------------------------------------------------------------------- | ---------------------------- | ---------------------------- |
| Default         | [TAR Link](https://github.com/DanielHabenicht/OSHome/releases/download/v0.1.3/oshome-Linux-musl-x86_64.tar.gz) | arm-unknown-linux-musleabihf                                                                                                          |                              | arm-unknown-linux-musleabihf |
| Raspberry Pi 3 | [TAR Link](https://github.com/DanielHabenicht/OSHome/releases/download/v0.1.3/oshome-Linux-musleabi-armv7.tar.gz) | arm-unknown-linux-musleabihf |
<!-- x-release-please-end -->

### Windows

<!-- x-release-please-start-version -->
[Download the ZIP](https://github.com/DanielHabenicht/OSHome/releases/download/v0.1.3/oshome-Windows-msvc-x86_64.zip).
<!-- x-release-please-end -->

### MacOS

<!-- x-release-please-start-version -->
[Download the TAR](https://github.com/DanielHabenicht/OSHome/releases/download/v0.1.3/oshome-macOS-x86_64.tar.gz)
<!-- x-release-please-end -->

## Installation

1. After the download, extract the archive and place the oshome executable in a directory of your choice.
2. Create a configuration file in the same directory as the executable. The configuration file should be named `config.yaml` and should contain the following:

```yaml
oshome:
  name: new_awesome

mqtt:
  broker: 192.168.178.167
  username: test
  password: a887aeda-7248-4b0f-a2e7-6f4ee0026f5e

shell:
  type: bash
  timeout: 5

button:
  - platform: shell
    id: my_button
    name: 'Write Hello World'
    command: "echo 'Hello World Daniel' >> test.log"

sensor:
  - platform: shell
    name: 'RAM Usage'
    id: ram_usage
    icon: 'mdi:bluetooth-settings'
    state_class: 'measurement'
    unit_of_measurement: '%'
    update_interval: 30s # 0 only executes it once and assumes a long running processes.
    command: |-
      Get-WmiObject Win32_OperatingSystem -Property * | % {([math]::Round(($_.FreePhysicalMemory)/$_.totalvisiblememorysize,2))}
```

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

## Uninstalltion

If you want to uninstall OSHome you can run the following command:

=== "Linux"
`bash
    ./oshome uninstall
   `
=== "Windows"
`powershell
    ./oshome.exe uninstall
   `

[^1]: This will change in the future to allow for custom compilation for modular builds and custom extensions.
