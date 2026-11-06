from asyncio import sleep
from unittest.mock import Mock

import aioesphomeapi
import pytest

from utils import UbiHome, fnv1_hash_object_id, run_ubihome


async def test_template_sensor_lambda():
    """
    A template sensor is read-only: its reported value always comes from a
    `globals.get` `lambda`, reporting the current value on startup and
    updating live whenever the backing `float` global changes.
    """

    sensor_id = "room_temperature"
    sensor_key = fnv1_hash_object_id(sensor_id)

    CONFIG = f"""
ubihome:
  name: test_device

api:

globals:
  - id: room_temperature_value
    type: float
    initial_value: 21.5

sensor:
  - platform: template
    name: "Room Temperature"
    id: {sensor_id}
    unit_of_measurement: "°C"
    lambda:
      globals.get: room_temperature_value

switch:
  - platform: template
    name: "Update Trigger"
    id: update_trigger
    optimistic: true
    turn_on_action:
      then:
        - globals.set:
            id: room_temperature_value
            value: 23.4
"""

    async with UbiHome("run", config=CONFIG, wait_for_api=True) as ubihome:
        api = aioesphomeapi.APIClient("127.0.0.1", ubihome.port, "")
        await api.connect(login=False)

        entities, _ = await api.list_entities_services()
        sensors = [e for e in entities if isinstance(e, aioesphomeapi.SensorInfo)]
        assert len(sensors) == 1, entities
        entity = sensors[0]
        assert entity.key == sensor_key
        assert entity.unit_of_measurement == "°C"

        mock = Mock()
        api.subscribe_states(mock)

        # Initial state comes from the global's `initial_value`.
        while not mock.called:
            await sleep(0.1)
        assert mock.call_args.args[0].state == pytest.approx(21.5)
        mock.reset_mock()

        switch_key = fnv1_hash_object_id("update_trigger")
        api.switch_command(switch_key, True)

        while not (mock.called and mock.call_args.args[0].key == sensor_key):
            await sleep(0.1)
        assert mock.call_args.args[0].state == pytest.approx(23.4)


async def test_template_sensor_validate():
    """
    A configuration using a template sensor with a `lambda` validates
    successfully.
    """

    config = """
ubihome:
  name: test_device

globals:
  - id: room_temperature_value
    type: float
    initial_value: 21.5

sensor:
  - platform: template
    name: "Room Temperature"
    id: room_temperature
    unit_of_measurement: "°C"
    lambda:
      globals.get: room_temperature_value
"""
    output, error = await run_ubihome("validate", config=config, extra_logging=False)

    assert not error, f"Unexpected error: {error}"
    assert "Configuration is valid." in output
