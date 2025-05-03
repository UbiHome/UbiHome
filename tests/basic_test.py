from asyncio import sleep
from pprint import pp
from tests.utils import run_ubihome


def test_help():
    output = run_ubihome("--help")

    assert output == """UbiHome - 0.5.1

UbiHome is a system which allows you to integrate any device running an OS into your smart home.
Documentation: https://ubihome.github.io/
Homepage: https://github.com/UbiHome/UbiHome

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
          Optional configuration file. If not provided, the default configuration will be used. [default: config.yaml]
  -h, --help
          Print help
  -V, --version
          Print version
"""


  