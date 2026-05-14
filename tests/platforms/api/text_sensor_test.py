from asyncio import sleep
from unittest.mock import Mock

import aioesphomeapi

from mock_file import IOMock
from utils import OS_PLATFORM, Platform, UbiHome, fnv1_hash_object_id


async def test_run(io_mock: IOMock):
    text_sensor_id = "my_text_sensor"
    text_sensor_name = "Test Text Sensor"

    io_mock.set_value("hello world")

    DEVICE_INFO_CONFIG = f"""
ubihome:
  name: test_device

api:

shell:
  type: {"bash" if OS_PLATFORM is Platform.LINUX else "powershell"}

text_sensor:
  - platform: shell
    id: {text_sensor_id}
    name: {text_sensor_name}
    update_interval: 1s
    command: "cat {io_mock.file}"
"""

    async with UbiHome("run", config=DEVICE_INFO_CONFIG, wait_for_api=True) as ubihome:
        api = aioesphomeapi.APIClient("127.0.0.1", ubihome.port, "")
        await api.connect(login=False)

        entities, services = await api.list_entities_services()
        assert len(entities) == 1, entities
        entity = entities[0]

        assert isinstance(entity, aioesphomeapi.TextSensorInfo)
        assert entity.key == fnv1_hash_object_id(text_sensor_id)
        assert entity.object_id == text_sensor_id
        assert entity.name == text_sensor_name

        mock = Mock()
        # Subscribe to the state changes
        api.subscribe_states(mock)

        # Wait for the initial state to be reported
        while not mock.called:
            await sleep(0.1)

        state = mock.call_args.args[0]
        assert state.state == "hello world"

        mock.reset_mock()

        # Update the mock value and wait for the new state
        io_mock.set_value("updated value")

        while not mock.called:
            await sleep(0.1)

        state = mock.call_args.args[0]
        assert state.state == "updated value"
