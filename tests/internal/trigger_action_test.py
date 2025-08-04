
import os
from utils import UbiHome, wait_for_mock_state


async def test_binary_sensor_triggers():
  """
  Test that Binary Sensor triggers are working by turning on/off a switch.
  """

  switch_mock = "testswitch.mock"
  sensor_mock = "test_sensor.mock"

  DEVICE_INFO_CONFIG = f"""
ubihome:
  name: test_device

shell:

switch:
  - platform: shell
    name: "Test Switch"
    id: test_switch
    command_on: "echo true > {switch_mock}"
    command_off: "echo false > {switch_mock}"
    command_state: "cat {switch_mock} || echo false"

binary_sensor:
  - platform: shell
    name: "Test Binary Sensor"
    update_interval: 2s
    command: |-
      cat {sensor_mock}
    on_press:
      then:
        - switch.turn_on: "test_switch"
    on_release:
      then:
        - switch.turn_off: "test_switch"
"""
  
  with open(sensor_mock, "w") as f:
      f.write("false")
  async with UbiHome("run", config=DEVICE_INFO_CONFIG) as ubihome:
    
    with open(sensor_mock, "w") as f:
        f.write("true")
    assert wait_for_mock_state(switch_mock, "true\n")
    os.remove(switch_mock)


    with open(sensor_mock, "w") as f:
        f.write("false")
    assert wait_for_mock_state(switch_mock, "false\n")

    os.remove(sensor_mock)
    os.remove(switch_mock)