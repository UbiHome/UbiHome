import asyncio

import pytest

from mock_file import IOMockFactory
from utils import UbiHome


async def test_parse_stderr_disabled_ignores_stderr_only_output(io_mock_factory: IOMockFactory):
    """
    A command that only writes to stderr must not be picked up unless
    `shell.parse_stderr` is enabled.
    """

    switch_mock = io_mock_factory.create_mock()
    sensor_mock = io_mock_factory.create_mock()

    DEVICE_INFO_CONFIG = f"""
ubihome:
  name: test_device

shell:

switch:
  - platform: shell
    name: "Test Switch"
    id: test_switch
    command_on: "echo true > {switch_mock}"
    command_off: "echo false > {switch_mock}"
    command_state: "cat {switch_mock} || echo false"

binary_sensor:
  - platform: shell
    name: "Test Binary Sensor"
    update_interval: 1s
    command: |-
      cat {sensor_mock} 1>&2
    on_press:
      then:
        - switch.turn_on: "test_switch"
"""
    sensor_mock.set_value("false")

    async with UbiHome("run", config=DEVICE_INFO_CONFIG):
        sensor_mock.set_value("true")

        # If the stderr-only value had been picked up, the switch would have
        # turned on; it must not, since parse_stderr is disabled.
        with pytest.raises(TimeoutError):
            await switch_mock.wait_for_mock_state("true", timeout=3)


async def test_parse_stderr_enabled_falls_back_to_stderr_when_stdout_empty(io_mock_factory: IOMockFactory):
    """
    With `shell.parse_stderr` enabled, a command that only writes to stderr is
    parsed the same way stdout-only output already is.
    """

    switch_mock = io_mock_factory.create_mock()
    sensor_mock = io_mock_factory.create_mock()

    DEVICE_INFO_CONFIG = f"""
ubihome:
  name: test_device

shell:
  parse_stderr: true

switch:
  - platform: shell
    name: "Test Switch"
    id: test_switch
    command_on: "echo true > {switch_mock}"
    command_off: "echo false > {switch_mock}"
    command_state: "cat {switch_mock} || echo false"

binary_sensor:
  - platform: shell
    name: "Test Binary Sensor"
    update_interval: 1s
    command: |-
      cat {sensor_mock} 1>&2
    on_press:
      then:
        - switch.turn_on: "test_switch"
    on_release:
      then:
        - switch.turn_off: "test_switch"
"""
    sensor_mock.set_value("false")

    async with UbiHome("run", config=DEVICE_INFO_CONFIG):
        sensor_mock.set_value("true")
        await switch_mock.wait_for_mock_state("true")
        switch_mock.remove()

        sensor_mock.set_value("false")
        await switch_mock.wait_for_mock_state("false")


async def test_stdout_preferred_over_stderr_even_when_parse_stderr_enabled(io_mock_factory: IOMockFactory):
    """
    `parse_stderr` is only a fallback for when stdout is empty: it must never
    override real stdout output with stderr content.
    """

    switch_mock = io_mock_factory.create_mock()

    DEVICE_INFO_CONFIG = f"""
ubihome:
  name: test_device

shell:
  parse_stderr: true

switch:
  - platform: shell
    name: "Test Switch"
    id: test_switch
    command_on: "echo true > {switch_mock}"
    command_off: "echo false > {switch_mock}"
    command_state: "cat {switch_mock} || echo false"

binary_sensor:
  - platform: shell
    name: "Test Binary Sensor"
    update_interval: 1s
    command: |-
      echo false
      echo true 1>&2
    on_press:
      then:
        - switch.turn_on: "test_switch"
"""

    async with UbiHome("run", config=DEVICE_INFO_CONFIG):
        # Stdout ("false") and stderr ("true") disagree; the switch must never
        # turn on, proving stderr never wins while stdout has content.
        with pytest.raises(TimeoutError):
            await switch_mock.wait_for_mock_state("true", timeout=3)


async def test_failure_logs_include_both_stdout_and_stderr():
    """
    When a command fails, the logged error must include both its stdout and
    stderr, not just stderr.
    """

    DEVICE_INFO_CONFIG = """
ubihome:
  name: test_device

shell:

text_sensor:
  - platform: shell
    name: "Failing Command"
    update_interval: 1s
    command: |-
      echo out message
      echo err message 1>&2
      exit 1
"""

    async with UbiHome("run", config=DEVICE_INFO_CONFIG) as ubihome:

        async def wait_for_log():
            while "out message" not in (ubihome.stdout or "") or "err message" not in (ubihome.stdout or ""):
                await asyncio.sleep(0.1)

        await asyncio.wait_for(wait_for_log(), timeout=5)
