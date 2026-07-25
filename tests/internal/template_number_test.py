from asyncio import sleep
from unittest.mock import Mock

import aioesphomeapi
import pytest

from mock_file import IOMockFactory
from utils import UbiHome, fnv1_hash_object_id, run_ubihome


async def test_template_number_optimistic_set(io_mock_factory: IOMockFactory):
    """
    An `optimistic` template number immediately reports the commanded value as
    its new state, and runs `set_action` (here pressing a shell button that
    writes to a mock file).
    """

    pressed_mock = io_mock_factory.create_mock()
    number_id = "fan_speed"
    number_key = fnv1_hash_object_id(number_id)

    CONFIG = f"""
ubihome:
  name: test_device

api:

shell:

button:
  - platform: shell
    name: "Runner"
    id: runner
    command: "echo pressed > {pressed_mock}"

number:
  - platform: template
    name: "Fan Speed"
    id: {number_id}
    min_value: 0
    max_value: 100
    step: 1
    optimistic: true
    set_action:
      then:
        - button.press: runner
"""

    async with UbiHome("run", config=CONFIG, wait_for_api=True) as ubihome:
        api = aioesphomeapi.APIClient("127.0.0.1", ubihome.port, "")
        await api.connect(login=False)

        entities, _ = await api.list_entities_services()
        numbers = [e for e in entities if isinstance(e, aioesphomeapi.NumberInfo)]
        assert len(numbers) == 1, entities
        entity = numbers[0]
        assert entity.key == number_key
        assert entity.min_value == pytest.approx(0)
        assert entity.max_value == pytest.approx(100)
        assert entity.step == pytest.approx(1)

        mock = Mock()
        api.subscribe_states(mock)

        # Initial state defaults to min_value.
        while not mock.called:
            await sleep(0.1)
        assert mock.call_args.args[0].state == pytest.approx(0)
        mock.reset_mock()

        api.number_command(number_key, 42.0)

        while not mock.called:
            await sleep(0.1)
        assert mock.call_args.args[0].state == pytest.approx(42.0)

        pressed_mock.wait_for_mock_state("pressed")


async def test_template_number_lambda_set(io_mock_factory: IOMockFactory):
    """
    A `lambda`-driven template number reports the value of its backing
    `float` global, and setting it (from the API) writes the commanded value
    back to that global.
    """

    number_id = "fan_speed"
    number_key = fnv1_hash_object_id(number_id)

    CONFIG = f"""
ubihome:
  name: test_device

api:

shell:

globals:
  - id: fan_speed_value
    type: float
    initial_value: 10

number:
  - platform: template
    name: "Fan Speed"
    id: {number_id}
    min_value: 0
    max_value: 100
    step: 1
    lambda:
      globals.get: fan_speed_value
"""

    async with UbiHome("run", config=CONFIG, wait_for_api=True) as ubihome:
        api = aioesphomeapi.APIClient("127.0.0.1", ubihome.port, "")
        await api.connect(login=False)

        mock = Mock()
        api.subscribe_states(mock)

        while not mock.called:
            await sleep(0.1)
        assert mock.call_args.args[0].state == pytest.approx(10.0)
        mock.reset_mock()

        api.number_command(number_key, 55.0)

        while not mock.called:
            await sleep(0.1)
        assert mock.call_args.args[0].state == pytest.approx(55.0)


async def test_template_number_validate():
    """
    A configuration using a template number with `set_action` and a `lambda`
    validates successfully.
    """

    config = """
ubihome:
  name: test_device

shell:

globals:
  - id: fan_speed_value
    type: float
    initial_value: 0

number:
  - platform: template
    name: "Fan Speed"
    id: fan_speed
    min_value: 0
    max_value: 100
    step: 1
    lambda:
      globals.get: fan_speed_value
    set_action:
      then:
        - globals.set:
            id: fan_speed_value
            value: 0
"""
    output, error = await run_ubihome("validate", config=config, extra_logging=False)

    assert not error, f"Unexpected error: {error}"
    assert "Configuration is valid." in output
