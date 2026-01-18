import pytest

from utils import OS_PLATFORM, Platform, run_ubihome


@pytest.mark.asyncio
async def test_validate_config():
    config = """
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
    output = await run_ubihome("validate", config=config, extra_logging=False)

    assert (
        """LogDirectory: ./logs
Config: config"""
        in output
    )
    assert (
        """Modules to load: ["button", "gpio", "mqtt", "power_utils", "sensor", "shell", "ubihome"]
Configuration is valid.
"""
        in output
    )


# Invalid character in area
"""
ubihome:
  name: test_a_1%
  friendly_name: 'test'
  area: test123\u200b
"""


# Unknown platform
"""
ubihome:
  name: test_a_1%
  friendly_name: 'test'
  area: test123\u200b

unknown_platform:
"""

# No base platform config
"""
ubihome:
  name: test_a_1%
  friendly_name: 'test'
  area: test123\u200b

sensor:
  - platform: unknown_platform
    name: "RAM Usage"
"""
