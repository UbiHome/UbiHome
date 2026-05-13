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
    id: my_button_1
    name: "Button 1"
    command: "echo 'Hello World!' >> test.log"

  - platform: shell
    id: my_button_2
    name: "Button 2"
    command: "echo 'Hello World!' >> test.log"

sensor:
  - platform: shell
    name: "RAM Usage 1"
    id: ram_usage_1
    update_interval: 30s
    command: |-
      cat testfile

  - platform: shell
    name: "RAM Usage 2"
    id: ram_usage_2
    update_interval: 30s
    command: |-
      cat testfile

switch:
  - platform: shell
    id: my_switch_1
    name: "Switch 1"
    command_on: "echo true > switch_state_1"
    command_off: "echo false > switch_state_1"
    command_state: "cat switch_state_1 || echo false"

  - platform: shell
    id: my_switch_2
    name: "Switch 2"
    command_on: "echo true > switch_state_2"
    command_off: "echo false > switch_state_2"
    command_state: "cat switch_state_2 || echo false"

binary_sensor:
- command: cat binary_sensor_6f2facc6375d4cee8723d9a1c6259ab9.mock
  id: my_binary_sensor1
  name: Test Binary Sensor 1
  platform: shell
  update_interval: 1s

- command: cat binary_sensor_6f2facc6375d4cee8723d9a1c6259ab9.mock
  id: my_binary_sensor2
  name: Test Binary Sensor 2
  platform: shell
  update_interval: 1s

number:
  - platform: shell
    id: my_number_1
    name: "Number 1"
    command_state: "cat number_state_1 || echo 0"
    command_set: "echo {} > number_state_1"
    update_interval: 30s

  - platform: shell
    id: my_number_2
    name: "Number 2"
    command_state: "cat number_state_2 || echo 0"
    command_set: "echo {} > number_state_2"
    update_interval: 30s

light:
  - platform: shell
    id: my_light_1
    name: "Light 1"
    command_on: "echo true > light_state_1"
    command_off: "echo false > light_state_1"
    command_state: "cat light_state_1 || echo false"

  - platform: shell
    id: my_light_2
    name: "Light 2"
    command_on: "echo true > light_state_2"
    command_off: "echo false > light_state_2"
    command_state: "cat light_state_2 || echo false"
"""


@pytest.mark.asyncio
async def test_validate_config_multiple():
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

    assert "Configuration is valid." in output
