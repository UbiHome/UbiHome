from mock_file import IOMockFactory
from utils import UbiHome


async def test_binary_sensor_triggers(io_mock_factory: IOMockFactory):
    """
    Test that Binary Sensor triggers are working by turning on/off a switch.
    """

    switch_mock = io_mock_factory.create_mock()
    sensor_mock = io_mock_factory.create_mock()

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
    sensor_mock.set_value("false")

    async with UbiHome("run", config=DEVICE_INFO_CONFIG) as ubihome:

        sensor_mock.set_value("true")
        switch_mock.wait_for_mock_state("true")
        switch_mock.remove()

        sensor_mock.set_value("false")
        switch_mock.wait_for_mock_state("false")
