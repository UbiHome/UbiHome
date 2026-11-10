---
title: CLI
---

UbiHome ships as a single executable. Running it without a subcommand prints this overview:

```bash
pi@raspberrypi:~/ $ ubihome
UbiHome - <Version />

UbiHome is a system which allows you to integrate any device running an OS into your smart home.
https://github.com/UbiHome/UbiHome

Usage: ubihome [OPTIONS] <COMMAND>

Commands:
  run        Run UbiHome manually.
  validate   Validates the configuration file.
  install    Install UbiHome
  update     Update the current UbiHome executable (from GitHub).
  uninstall  Uninstall UbiHome
  help       Print this message or the help of the given subcommand(s)

Options:
  -c, --configuration <configuration_file>
          Optional configuration file. If not provided, the default configuration will be used. [default: config.yml config.yaml]
  -h, --help
          Print help
  -V, --version
          Print version
```

## Global options

These options are accepted by every command:

| Option | Description |
| --- | --- |
| `-c, --configuration <configuration_file>` | Configuration file to use. If omitted, `config.yml` is tried first, then `config.yaml`, in the current directory. |
| `--log-level <log_level>` | Overrides the configured [logger](/features/platforms/logger/) level for this invocation. |
| `--sentry <sentry>` | Sentry DSN to enable error reporting for this invocation. Also settable via the `SENTRY` environment variable. Disabled by default. |

## `run`

Starts UbiHome: loads and validates the configuration file, then loads and runs the configured platforms until it receives `Ctrl+C`/`SIGTERM`.

```bash
ubihome run
```

## `validate`

Loads and validates the configuration file the same way `run` does, but exits immediately afterwards instead of starting any platform. Useful for checking a configuration change before restarting the service.

```bash
ubihome validate
```

## `install`

Installs UbiHome as a persistent background service (systemd on Linux/macOS, a Windows Service on Windows) so it survives reboots, and starts it. Copies the current executable to the target location and points the service at it.

```bash
ubihome install [location]
```

If `location` is omitted, you're prompted for it interactively (defaults to `/usr/bin/ubihome` on Linux/macOS, `C:\Program Files\ubihome` on Windows). Running `install` again against an existing installation updates the service in place.

## `update`

Downloads and installs a newer UbiHome executable, replacing the one currently running. See [Updating](/commands/update/) for the full picture, including pre-releases and installing an unreleased pull request build.

```bash
ubihome update
```

## `uninstall`

Stops and removes the service installed by `install`, and deletes its installation directory.

```bash
ubihome uninstall [location]
```

If `location` is omitted, you're prompted for it interactively.

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/commands/update/">Updating</a>
  <a href="/features/platforms/logger/">Logger</a>
  <a href="/help/error_reporting/">Error Reporting</a>
</div>
