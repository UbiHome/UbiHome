
from asyncio import sleep
import contextlib
import os
from unittest.mock import Mock
from utils import UbiHome
import aioesphomeapi
from utils import wait_and_get_file


async def test_run():
  button_id = "my_switch"
  button_name = "Switch it"
  switch_mock = "switch_test.mock"
  with contextlib.suppress(FileNotFoundError):
    os.remove(switch_mock)

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
   update_interval: 1s
"""

  async with UbiHome("run", config=DEVICE_INFO_CONFIG, wait_for_api=True) as ubihome:
    api = aioesphomeapi.APIClient("127.0.0.1", 6053, "")
    await api.connect(login=False)

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
    
    # Test switching the switch on via command
    api.switch_command(0, True)
    assert wait_and_get_file(switch_mock) == "true\n"

    # State update should be send back
    while not mock.called:
      await sleep(0.1)
    state = mock.call_args.args[0]
    assert state.state == True
    os.remove(switch_mock)
    mock.reset_mock()

    # Test switching the switch off via command
    api.switch_command(0, False)
    assert wait_and_get_file(switch_mock) == "false\n"
    # State update should be send back
    while not mock.called:
      await sleep(0.1)
    state = mock.call_args.args[0]
    assert state.state == False
    mock.reset_mock()
    os.remove(switch_mock)

    # Test switching the switch on via local change
    with open(switch_mock, "w") as f:
      f.write("true")

    # Wait for the state change
    while not mock.called:
      await sleep(0.1)
    state = mock.call_args.args[0]
    assert state.state == True

