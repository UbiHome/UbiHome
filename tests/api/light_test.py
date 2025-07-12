
from asyncio import sleep
from unittest.mock import Mock

import pytest
from utils import UbiHome
import aioesphomeapi
from utils import wait_and_get_file


async def test_run():
  sensor_id = "my_sensor"
  sensor_name = "Test Sensor"
  sensor_mock = "test_sensor.mock"
  DEVICE_INFO_CONFIG = f"""
ubihome:
  name: test_device

api:

shell:
  
lights:
  - platform: shell
    id: {sensor_id}
    update_interval: 1s
    name: {sensor_name}
    command: "cat {sensor_mock}"
"""
  with open(sensor_mock, "w") as f:
      f.write("0.1")

  async with UbiHome("run", DEVICE_INFO_CONFIG) as ubihome:
    api = aioesphomeapi.APIClient("127.0.0.1", 6053, "MyPassword")
    await api.connect(login=True)

    entities, services = await api.list_entities_services()
    assert len(entities) == 1, entities
    entity = entities[0]

    assert type(entity) == aioesphomeapi.SensorInfo
    assert entity.unique_id == sensor_id
    assert entity.name == sensor_name

    mock = Mock()
    # Subscribe to the state changes
    api.subscribe_states(mock)

    with open(sensor_mock, "w") as f:
      f.write("0.2")

    # Wait for the state change
    while not mock.called:
      await sleep(0.1)

    state = mock.call_args.args[0]
    assert state.state == pytest.approx(0.2)

