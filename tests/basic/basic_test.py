import asyncio

import pytest

from utils import OS_PLATFORM, Platform, UbiHome, run_ubihome


@pytest.mark.asyncio
async def test_version_help():
    version, error = await run_ubihome("--version")

    executable = "ubihome"
    if OS_PLATFORM == Platform.WINDOWS:
        executable += ".exe"

    output, error = await run_ubihome("--help")
    assert (
        output
        == f"""{version}
UbiHome is a system which allows you to integrate any device running an OS into your smart home.
Documentation: https://ubihome.github.io/
Homepage: https://github.com/UbiHome/UbiHome

Usage: {executable} [OPTIONS] <COMMAND>

Commands:
  run        Run UbiHome manually.
  validate   Validates the configuration file.
  install    Install UbiHome
  update     Update the current UbiHome executable (from GitHub).
  uninstall  Uninstall UbiHome
  help       Print this message or the help of the given subcommand(s)

Options:
  -c, --configuration <configuration_file>
          Optional configuration file. If not provided, config.yml (or config.yaml) in the current directory is used.
      --log-level <log_level>
          The log level (overwrites the config).
      --sentry <sentry>
          Sentry DSN to enable error reporting. If not provided, error reporting is disabled. [env: SENTRY=]
  -h, --help
          Print help
  -V, --version
          Print version
"""  # noqa: E501
    )


@pytest.mark.asyncio
async def test_no_parameters():
    version, error = await run_ubihome("--version")

    executable = "ubihome"
    if OS_PLATFORM == Platform.WINDOWS:
        executable += ".exe"

    output, error = await run_ubihome("")
    assert (
        error
        == f"""{version}
UbiHome is a system which allows you to integrate any device running an OS into your smart home.
Documentation: https://ubihome.github.io/
Homepage: https://github.com/UbiHome/UbiHome

Usage: {executable} [OPTIONS] <COMMAND>

Commands:
  run        Run UbiHome manually.
  validate   Validates the configuration file.
  install    Install UbiHome
  update     Update the current UbiHome executable (from GitHub).
  uninstall  Uninstall UbiHome
  help       Print this message or the help of the given subcommand(s)

Options:
  -c, --configuration <configuration_file>
          Optional configuration file. If not provided, config.yml (or config.yaml) in the current directory is used.
      --log-level <log_level>
          The log level (overwrites the config).
      --sentry <sentry>
          Sentry DSN to enable error reporting. If not provided, error reporting is disabled. [env: SENTRY=]
  -h, --help
          Print help
  -V, --version
          Print version
"""  # noqa: E501
    )


@pytest.mark.asyncio
async def test_module_failure_exits_non_zero_with_clear_error():
    """A module failing at runtime must stop UbiHome with a non-zero exit code
    and name the module, instead of exiting successfully."""
    config = """
ubihome:
  name: module_failure

mdns:
  # TEST-NET-1: never assigned to a local interface, so the lookup fails.
  ip: 192.0.2.1
"""
    async with UbiHome("run", config=config, throw_on_error=False) as ubihome:
        exit_code = await asyncio.wait_for(ubihome.process.wait(), timeout=10)
        await asyncio.sleep(0.1)

        assert exit_code == 1
        assert "mdns: Ip Address not found" in ubihome.stderr
        assert "UbiHome stopped because a task failed." in ubihome.stderr
        assert "Configuration is invalid" not in ubihome.stderr
