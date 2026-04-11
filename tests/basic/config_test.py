import pytest

from utils import OS_PLATFORM, Platform, run_ubihome
from platformdirs import user_data_dir

CONFIG = """
ubihome:
  name: new_awesome

shell: 
  type: bash

button: 
 - platform: shell
   id: my_button
   name: "Write Hello World"
   command: "echo 'Hello World!' >> test.log"

sensor:
  - platform: shell
    name: "RAM Usage"
    id: ram_usage
    icon: mdi:memory
    state_class: "measurement"
    unit_of_measurement: "%"
    update_interval: 30s
    command: |-
      cat testfile

gpio:
  device: raspberryPi

mqtt:
  broker: 127.0.0.1
  username: test
  password: test

power_utils:
"""


@pytest.mark.asyncio
async def test_validate_config():
    output, error = await run_ubihome("validate", config=CONFIG, extra_logging=False)
    log_dir = user_data_dir()

    assert not error
    assert (
        """LogDirectory: ./logs
Config: config"""
        in output
    ) or (
        f"""LogDirectory: {log_dir}
Config: config"""
        in output
    )

    assert (
        """Platforms to load: ["gpio", "mqtt", "power_utils", "shell"]
Configuration is valid.
"""
        in output
    )


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "config,expected_error",
    [
        (
            # Invalid character in area
            """
ubihome:
  name: test_a_1%
  friendly_name: 'test'
  area: test123\u200b
""",
            "validation error: This string contains non-printable characters:",
        ),
        (
            # Unknown platform
            """
ubihome:
  name: test_a_1%
  friendly_name: 'test'
  area: test123

unknown_platform:
""",
            'Unknown platform specified: unknown_platform\nRemove the "unknown_platform:" entry from your configuration or install the cargo crate containing the platform',
        ),
        (
            # No base platform config
            """
ubihome:
  name: test_a_1%
  friendly_name: 'test'
  area: test123

shell:

sensor:
  - platform: unknown_platform
    name: "RAM Usage"
""",
            [
                "Platform 'unknown_platform' is not configured in the",
                "Allowed platforms are: shell for `sensor[0].platform`",
            ],
        ),
    ],
)
async def test_validate_config_error(config, expected_error: str | list[str]):
    output, error = await run_ubihome("validate", config=config, extra_logging=False)
    assert "Configuration is invalid:" in error

    if isinstance(expected_error, str):
        expected_error = [expected_error]

    for expected in expected_error:
        assert expected in error, (
            f"Expected error '{expected}' not found in output: {error}"
        )
