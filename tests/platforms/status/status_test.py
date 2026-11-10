from asyncio import sleep
from unittest.mock import Mock

import aioesphomeapi

from utils import UbiHome


async def _wait_for_state(mock: Mock, expected: bool, timeout: float = 15.0) -> None:
    """Wait until the latest binary sensor state equals ``expected``."""
    elapsed = 0.0
    while elapsed < timeout:
        if mock.called and mock.call_args.args[0].state is expected:
            return
        await sleep(0.2)
        elapsed += 0.2
    raise AssertionError(f"status binary sensor never reached state {expected}")


async def test_status_reports_on():
    """
    The status binary sensor reports "on" as soon as UbiHome has started up,
    reflecting that the UbiHome instance itself is running.
    """

    CONFIG = """
ubihome:
  name: test_device

api:

status:

binary_sensor:
  - platform: status
    id: status
    name: Status
"""

    async with UbiHome("run", config=CONFIG, wait_for_api=True) as ubihome:
        api = aioesphomeapi.APIClient("127.0.0.1", ubihome.port, "")
        await api.connect(login=False)

        entities, _ = await api.list_entities_services()
        assert len(entities) == 1, entities
        assert isinstance(entities[0], aioesphomeapi.BinarySensorInfo)
        assert entities[0].object_id == "status"
        assert entities[0].entity_category == aioesphomeapi.EntityCategory.DIAGNOSTIC

        mock = Mock()
        api.subscribe_states(mock)

        await _wait_for_state(mock, True)
