from asyncio import sleep
from unittest.mock import Mock

import aioesphomeapi
import pytest
from mock_file import IOMock
from utils import OS_PLATFORM, Platform, UbiHome


async def test_round_filter(io_mock: IOMock):
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
    filters:
      - round: 2
"""
    io_mock.set_value("0.1111")

    async with UbiHome("run", config=DEVICE_INFO_CONFIG, wait_for_api=True) as ubihome:
        api = aioesphomeapi.APIClient("127.0.0.1", ubihome.port, "")
        await api.connect(login=False)

        entities, services = await api.list_entities_services()
        assert len(entities) == 1, entities
        entity = entities[0]

        assert isinstance(entity, aioesphomeapi.SensorInfo)
        assert entity.object_id == sensor_id
        assert entity.name == sensor_name

        mock = Mock()
        api.subscribe_states(mock)

        # Round down: 1.123345 -> 1.12
        io_mock.set_value("1.123345")

        while not mock.called:
            await sleep(0.1)

        state = mock.call_args.args[0]
        assert state.state == pytest.approx(1.12)

        # Round up: 1.126 -> 1.13
        mock.reset_mock()
        io_mock.set_value("1.126")

        while not mock.called:
            await sleep(0.1)

        state = mock.call_args.args[0]
        assert state.state == pytest.approx(1.13)

        # Edge case: 1.125 rounds to 1.12 (round-half-to-even / banker's rounding in f32)
        mock.reset_mock()
        io_mock.set_value("1.125")

        while not mock.called:
            await sleep(0.1)

        state = mock.call_args.args[0]
        assert state.state == pytest.approx(1.12)


async def test_round_filter_3_decimals(io_mock: IOMock):
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
    filters:
      - round: 3
"""
    io_mock.set_value("0.1111")

    async with UbiHome("run", config=DEVICE_INFO_CONFIG, wait_for_api=True) as ubihome:
        api = aioesphomeapi.APIClient("127.0.0.1", ubihome.port, "")
        await api.connect(login=False)

        entities, services = await api.list_entities_services()
        assert len(entities) == 1, entities

        mock = Mock()
        api.subscribe_states(mock)

        # Round down: 1.12341 -> 1.123
        io_mock.set_value("1.12341")

        while not mock.called:
            await sleep(0.1)

        state = mock.call_args.args[0]
        assert state.state == pytest.approx(1.123)

        # Round up: 1.12361 -> 1.124
        mock.reset_mock()
        io_mock.set_value("1.12361")

        while not mock.called:
            await sleep(0.1)

        state = mock.call_args.args[0]
        assert state.state == pytest.approx(1.124)
