from asyncio import sleep
from unittest.mock import Mock

import pytest
from mock_file import IOMock
from utils import OS_PLATFORM, Platform, UbiHome
import aioesphomeapi


async def test_run(io_mock: IOMock):
    light_id = "my_light"
    light_name = "Test Light"
    io_mock.set_value("false")
    DEVICE_INFO_CONFIG = f"""
ubihome:
  name: test_device

api:

shell:
  type: {"bash" if OS_PLATFORM is Platform.LINUX else "powershell"}
  
light:
  - platform: shell
    id: {light_id}
    update_interval: 1s
    name: {light_name}
    command_on: "echo true > {io_mock.file}"
    command_off: "echo false > {io_mock.file}"
    command_state: "cat {io_mock.file}"

"""

    async with UbiHome("run", config=DEVICE_INFO_CONFIG, wait_for_api=True) as ubihome:
        api = aioesphomeapi.APIClient("127.0.0.1", ubihome.port, "")
        await api.connect(login=False)

        entities, services = await api.list_entities_services()
        assert len(entities) == 1, entities
        entity = entities[0]

        assert type(entity) == aioesphomeapi.LightInfo
        assert entity.object_id == light_id
        assert entity.name == light_name

        mock = Mock()
        # Subscribe to the state changes
        api.subscribe_states(mock)

        io_mock.set_value("true")

        # Wait for the state change
        while not mock.called:
            await sleep(0.1)

        state = mock.call_args.args[0]
        assert state.state is True
