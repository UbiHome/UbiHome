from asyncio import sleep
from unittest.mock import Mock

import pytest
from mock_file import IOMock
from utils import OS_PLATFORM, Platform, UbiHome
import aioesphomeapi


async def test_run(io_mock: IOMock):
    sensor_id = "my_sensor"
    sensor_name = "Test Sensor"
    DEVICE_INFO_CONFIG = f"""
ubihome:
  name: test_device

api:

shell:
  type: {"bash" if OS_PLATFORM is Platform.LINUX else "powershell"}
  
sensor:
  - platform: shell
    id: {sensor_id}
    update_interval: 1s
    name: {sensor_name}
    command: "cat {io_mock.file}"
"""
    io_mock.set_value("0.1")

    async with UbiHome("run", config=DEVICE_INFO_CONFIG, wait_for_api=True) as ubihome:
        api = aioesphomeapi.APIClient("127.0.0.1", ubihome.port, "")
        await api.connect(login=False)

        entities, services = await api.list_entities_services()
        assert len(entities) == 1, entities
        entity = entities[0]

        assert type(entity) == aioesphomeapi.SensorInfo
        assert entity.object_id == sensor_id
        assert entity.name == sensor_name

        mock = Mock()
        # Subscribe to the state changes
        api.subscribe_states(mock)

        io_mock.set_value("0.2")

        # Wait for the state change
        while not mock.called:
            await sleep(0.1)

        state = mock.call_args.args[0]
        assert state.state == pytest.approx(0.2)
