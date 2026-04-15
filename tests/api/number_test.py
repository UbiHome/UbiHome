from asyncio import sleep
from unittest.mock import Mock

import pytest
from mock_file import IOMock
from utils import OS_PLATFORM, Platform, UbiHome, fnv1_hash_object_id
import aioesphomeapi


async def test_run(io_mock: IOMock):
    number_id = "my_number"
    number_key = fnv1_hash_object_id(number_id)
    number_name = "Test Number"
    min_value = 0.0
    max_value = 100.0
    step = 1.0
    unit = "%"

    io_mock.set_value("50.0")

    DEVICE_INFO_CONFIG = f"""
ubihome:
  name: test_device

api:

shell:
  type: {"bash" if OS_PLATFORM is Platform.LINUX else "powershell"}

number:
  - platform: shell
    id: {number_id}
    name: {number_name}
    unit_of_measurement: {unit}
    min_value: {min_value}
    max_value: {max_value}
    step: {step}
    update_interval: 1s
    command_state: "cat {io_mock.file}"
    command_set: "echo {{{{ value }}}} > {io_mock.file}"
"""

    async with UbiHome("run", config=DEVICE_INFO_CONFIG, wait_for_api=True) as ubihome:
        api = aioesphomeapi.APIClient("127.0.0.1", ubihome.port, "")
        await api.connect(login=False)

        entities, services = await api.list_entities_services()
        assert len(entities) == 1, entities
        entity = entities[0]

        assert type(entity) == aioesphomeapi.NumberInfo
        assert entity.key == number_key
        assert entity.object_id == number_id
        assert entity.name == number_name
        assert entity.min_value == pytest.approx(min_value)
        assert entity.max_value == pytest.approx(max_value)
        assert entity.step == pytest.approx(step)
        assert entity.unit_of_measurement == unit

        mock = Mock()
        # Subscribe to the state changes
        api.subscribe_states(mock)

        # Wait for the initial state to be reported
        while not mock.called:
            await sleep(0.1)

        state = mock.call_args.args[0]
        assert state.state == pytest.approx(50.0)

        mock.reset_mock()

        # Update the mock value and wait for the new state
        io_mock.set_value("75.0")

        while not mock.called:
            await sleep(0.1)

        state = mock.call_args.args[0]
        assert state.state == pytest.approx(75.0)
