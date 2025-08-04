
from asyncio import sleep
from unittest.mock import Mock

import pytest
from utils import UbiHome
import aioesphomeapi
from utils import wait_and_get_file


async def test_run():
  light_id = "my_light"
  light_name = "Test Light"
  light_mock = "test_light.mock"
  DEVICE_INFO_CONFIG = f"""
ubihome:
  name: test_device

api:

shell:
  
light:
  - platform: shell
    id: {light_id}
    update_interval: 1s
    name: {light_name}
    command_on: "echo true > {light_mock}"
    command_off: "echo false > {light_mock}"
    command_state: "cat {light_mock} || echo false"

"""
  with open(light_mock, "w") as f:
      f.write("0.1")

  async with UbiHome("run", config=DEVICE_INFO_CONFIG, wait_for_api=True) as ubihome:
    api = aioesphomeapi.APIClient("127.0.0.1", 6053, "")
    await api.connect(login=False)

    entities, services = await api.list_entities_services()
    assert len(entities) == 1, entities
    entity = entities[0]

    assert type(entity) == aioesphomeapi.LightInfo
    assert entity.unique_id == light_id
    assert entity.name == light_name

    mock = Mock()
    # Subscribe to the state changes
    api.subscribe_states(mock)

    with open(light_mock, "w") as f:
      f.write("true")

    # Wait for the state change
    while not mock.called:
      await sleep(0.1)

    state = mock.call_args.args[0]
    assert state.state is True

