import os
import platform
import time
from asyncio import sleep
from unittest.mock import Mock

import aioesphomeapi
import pytest

from mock_file import IOMock, IOMockFactory
from utils import UbiHome, fnv1_hash_object_id, run_ubihome


def _read_mock_content(mock: IOMock) -> str:
    """
    Mirrors `IOMock.wait_for_mock_state`'s platform-specific file reading, but
    just returns the current content instead of waiting for a substring - so
    callers can count occurrences instead of matching an exact (whitespace-
    sensitive) string.
    """
    if platform.system() == "Windows":
        try:
            with open(mock.file, encoding="utf-16") as f:
                content = f.read()
        except UnicodeError:
            with open(mock.file) as f:
                content = f.read()
        return content.encode("utf-8").decode("utf-8")
    with open(mock.file) as f:
        return f.read()


async def _wait_for_occurrences(mock: IOMock, substring: str, count: int, timeout=5) -> str:
    """
    Waits until `substring` occurs at least `count` times in the mock file.
    Uses `asyncio.sleep` (not a blocking `time.sleep`) so it doesn't stall the
    event loop the API client relies on to keep its connection alive.
    """
    start_time = time.time()
    content = ""
    while content.count(substring) < count:
        if time.time() - start_time > timeout:
            raise TimeoutError(
                f"'{substring}' occurred {content.count(substring)} times, "
                f"expected {count}, within {timeout} seconds: {content!r}"
            )
        if os.path.exists(mock.file):
            content = _read_mock_content(mock)
        await sleep(0.1)
    return content


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
    A `lambda`-driven template number reports the value of a `float` global
    read via `id()`. Its `set_action` runs a `lambda` action that writes the
    commanded value (`x`) back to that global with `set_global`, so the
    reported state tracks the command.
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
    lambda: |-
      return id(fan_speed_value)
    set_action:
      then:
        - lambda: |
            set_global('fan_speed_value', x)
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


async def test_template_number_lambda_set_action_presses_buttons(
    io_mock_factory: IOMockFactory,
):
    """
    A `set_action` `lambda` action can read the commanded value as `x`,
    compare it against a global with `id()`, and press other entities'
    buttons (`id(...).press()`) - the pattern used to nudge a real device's
    volume up/down one step at a time towards a commanded target.
    """

    up_mock = io_mock_factory.create_mock()
    down_mock = io_mock_factory.create_mock()

    CONFIG = f"""
ubihome:
  name: test_device

api:

shell:

globals:
  - id: global_volume
    type: float
    initial_value: 10

button:
  - platform: shell
    id: volume_up_button
    command: "echo up >> {up_mock}"
  - platform: shell
    id: volume_down_button
    command: "echo down >> {down_mock}"

number:
  - platform: template
    name: "Volume"
    id: volume
    min_value: 0
    max_value: 100
    step: 1
    lambda: |-
      return id(global_volume)
    set_action:
      then:
        - lambda: |
            let difference = id(global_volume) - x
            while (difference != 0) {{
              if (difference < 0) {{
                id(volume_up_button).press();
                difference = difference + 1
              }} else {{
                id(volume_down_button).press();
                difference = difference - 1
              }}
            }}
            set_global('global_volume', x)
"""

    number_id = "volume"
    number_key = fnv1_hash_object_id(number_id)

    async with UbiHome("run", config=CONFIG, wait_for_api=True) as ubihome:
        api = aioesphomeapi.APIClient("127.0.0.1", ubihome.port, "")
        await api.connect(login=False)

        # global_volume starts at 10; commanding 13 should press "up" 3 times.
        api.number_command(number_key, 13.0)
        up_content = await _wait_for_occurrences(up_mock, "up", 3)
        assert up_content.count("up") == 3

        # Commanding back down to 11 should press "down" twice.
        api.number_command(number_key, 11.0)
        down_content = await _wait_for_occurrences(down_mock, "down", 2)
        assert down_content.count("down") == 2


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
    lambda: |-
      return id(fan_speed_value)
    set_action:
      then:
        - globals.set:
            id: fan_speed_value
            value: 0
        - lambda: |
            log('commanded value: ' + x);
"""
    output, error = await run_ubihome("validate", config=config, extra_logging=False)

    assert not error, f"Unexpected error: {error}"
    assert "Configuration is valid." in output
