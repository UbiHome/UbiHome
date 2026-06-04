import pytest
from platformdirs import user_data_dir

from utils import run_ubihome

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

  - platform: power_utils
    name: "Hibernate"
    action: hibernate
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
            'Unknown platform specified: unknown_platform\nRemove the "unknown_platform:" entry from your configuration or install the cargo crate containing the platform',  # noqa: E501
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
        assert expected in error, f"Expected error '{expected}' not found in output: {error}"


@pytest.mark.asyncio
async def test_mixed_platform_buttons_valid():
    """Test that buttons with different platforms are properly handled.

    This tests the mapper's ability to:
    - Skip items for other platforms (shell button is skipped by power_utils mapper)
    - Properly deserialize items for its own platform (power_utils button)
    """
    config = """
ubihome:
  name: test_mixed_platforms

shell:
  type: bash

power_utils:

button:
  - platform: shell
    id: shell_button
    name: "Shell Button"
    command: "echo 'test'"

  - platform: power_utils
    id: power_button
    name: "Hibernate Button"
    action: hibernate

  - platform: shell
    id: another_shell
    name: "Another Shell"
    command: "echo 'test2'"
"""
    output, error = await run_ubihome("validate", config=config, extra_logging=False)

    assert not error, f"Unexpected error: {error}"
    assert "Configuration is valid." in output


@pytest.mark.asyncio
async def test_multiple_sensors_different_platforms():
    """Test that sensors from different platforms can coexist without errors.

    This is a regression test for the mapper fix. Without the fix, having
    shell sensors followed by other platform configurations would cause
    the mapper to crash when trying to deserialize incompatible items.

    Verifies that the mapper gracefully skips items for other platforms
    and only processes items matching its target platform.
    """
    config = """
ubihome:
  name: test_sensor_mix

shell:
  type: bash

illuminance:

sensor:
  - platform: shell
    name: "RAM Usage"
    id: ram_usage
    command: "free | awk 'NR==2{print $3*100/$2}'"
    update_interval: 30s

  - platform: illuminance
    name: "Ambient Light"
    id: ambient_light
    update_interval: 30s

  - platform: shell
    name: "CPU Temp"
    id: cpu_temp
    command: "cat /sys/class/thermal/thermal_zone0/temp"
    update_interval: 60s
"""
    output, error = await run_ubihome("validate", config=config, extra_logging=False)

    # Should not error - this is the regression test
    assert not error, f"Config with mixed-platform sensors should be valid, but got: {error}"
    assert "Configuration is valid." in output


@pytest.mark.asyncio
async def test_shell_switch_with_invalid_command_type():
    """Test that deserialization errors for matching platform configs are caught.

    This verifies that the mapper DOES propagate errors when an item's platform
    matches but the configuration has wrong types/invalid values.

    Regression test: validates that we don't silently drop invalid configs
    for our platform. Only "missing field" errors (platform mismatches)
    should be silently skipped.
    """
    config = """
ubihome:
  name: test_shell_switch_error

shell:
  type: bash

switch:
  - platform: shell
    id: valid_switch
    name: "Valid Switch"
    command_on: "echo 'on'"
    command_off: "echo 'off'"

  - platform: shell
    id: invalid_switch
    name: "Invalid Switch - command_on should be string"
    command_on: 12345
    command_off: "echo 'off'"
"""
    output, error = await run_ubihome("validate", config=config, extra_logging=False)

    # Should error because command_on is a number, not a string.
    assert "Configuration is invalid:" in error, f"Should detect type error for command_on, got: {error}"
