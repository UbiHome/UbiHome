
from asyncio import sleep
import os
from unittest.mock import Mock
from utils import UbiHome
import aioesphomeapi
from utils import wait_and_get_file


async def test_run():
  button_id = "my_switch"
  button_name = "Switch it"
  switch_mock = "test_switch.mock"
  DEVICE_INFO_CONFIG = f"""
ubihome:
  name: test_device

api:

shell:
  
switch: 
 - platform: shell
   id: {button_id}
   name: {button_name}
   command_on: "echo true > {switch_mock}"
   command_off: "echo false > {switch_mock}"
   command_state: "cat {switch_mock} || echo false"
"""

  async with UbiHome("run", DEVICE_INFO_CONFIG) as ubihome:
    api = aioesphomeapi.APIClient("127.0.0.1", 6053, "MyPassword")
    await api.connect(login=True)

    entities, services = await api.list_entities_services()
    print("switches", entities, services)
    assert len(entities) == 1, entities
    entity = entities[0]

    assert type(entity) == aioesphomeapi.SwitchInfo
    assert entity.unique_id == button_id
    assert entity.name == button_name

    mock = Mock()
    # Subscribe to the state changes
    api.subscribe_states(mock)
    
    api.switch_command(0, True)
    assert wait_and_get_file(switch_mock) == "true\n"
    os.remove(switch_mock)

    api.switch_command(0, False)
    assert wait_and_get_file(switch_mock) == "false\n"
    os.remove(switch_mock)

    # # TODO: Switch State!
    # with open(sensor_mock, "w") as f:
    #   f.write("true")

    # Wait for the state change
    while not mock.called:
      await sleep(0.1)

    state = mock.call_args.args[0]
    assert state.state == True

    # with open(sensor_mock, "w") as f:
    #   f.write("false")

    # mock.reset_mock()
    # # Wait for the state change
    # while not mock.called:
    #   await sleep(0.1)

    # state = mock.call_args.args[0]
    # assert state.state == False