import pytest

from utils import OS_PLATFORM, Platform, run_ubihome


@pytest.mark.asyncio
async def test_version_help():
    version = await run_ubihome("--version")

    executable = "ubihome"
    if OS_PLATFORM == Platform.WINDOWS:
        executable += ".exe"

    output = await run_ubihome("--help")
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
          Optional configuration file. If not provided, the default configuration will be used. [default: config.yml config.yaml]
      --log-level <log_level>
          The log level (overwrites the config).
  -h, --help
          Print help
  -V, --version
          Print version
"""
    )
