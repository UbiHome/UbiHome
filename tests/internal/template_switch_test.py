from mock_file import IOMockFactory
from utils import UbiHome, run_ubihome


async def test_template_switch_actions(io_mock_factory: IOMockFactory):
    """
    A template switch runs its `turn_on_action` / `turn_off_action` automations
    when it is switched. Here a binary sensor toggles the template switch, and
    the switch's actions press shell buttons that write to mock files.
    """

    on_mock = io_mock_factory.create_mock()
    off_mock = io_mock_factory.create_mock()
    sensor_mock = io_mock_factory.create_mock()

    CONFIG = f"""
ubihome:
  name: test_device

shell:

button:
  - platform: shell
    name: "On Runner"
    id: on_runner
    command: "echo on > {on_mock}"
  - platform: shell
    name: "Off Runner"
    id: off_runner
    command: "echo off > {off_mock}"

switch:
  - platform: template
    name: "Template Switch"
    id: template_switch
    optimistic: true
    turn_on_action:
      - button.press: on_runner
    turn_off_action:
      - button.press: off_runner

binary_sensor:
  - platform: shell
    name: "Trigger Sensor"
    id: trigger_sensor
    update_interval: 2s
    command: |-
      cat {sensor_mock}
    on_press:
      then:
        - switch.turn_on: template_switch
    on_release:
      then:
        - switch.turn_off: template_switch
"""
    sensor_mock.set_value("false")

    async with UbiHome("run", config=CONFIG):
        # Turning the binary sensor on turns the template switch on, which runs
        # its turn_on_action (press the on_runner button).
        sensor_mock.set_value("true")
        on_mock.wait_for_mock_state("on")

        # Turning it off runs the turn_off_action (press the off_runner button).
        off_mock.remove()
        sensor_mock.set_value("false")
        off_mock.wait_for_mock_state("off")


async def test_globals_and_template_switch_validate():
    """
    A configuration using `globals` and a template switch with a `globals.set`
    action validates successfully.
    """

    config = """
ubihome:
  name: test_device

shell:

globals:
  - id: my_flag
    type: bool
    initial_value: "false"
    restore_value: true

switch:
  - platform: template
    name: "Flag Switch"
    id: flag_switch
    optimistic: true
    turn_on_action:
      - globals.set:
          id: my_flag
          value: "true"
    turn_off_action:
      - globals.set:
          id: my_flag
          value: "false"
"""
    output, error = await run_ubihome("validate", config=config, extra_logging=False)

    assert not error, f"Unexpected error: {error}"
    assert "Configuration is valid." in output


async def test_template_switch_globals_get_lambda_validate():
    """
    A template switch can read its state from a global with a `globals.get`
    lambda (written in YAML, no code).
    """

    config = """
ubihome:
  name: test_device

globals:
  - id: relay_state
    type: bool
    initial_value: "false"

switch:
  - platform: template
    name: "Relay"
    id: relay
    lambda:
      globals.get: relay_state
    turn_on_action:
      - globals.set:
          id: relay_state
          value: "true"
    turn_off_action:
      - globals.set:
          id: relay_state
          value: "false"
"""
    output, error = await run_ubihome("validate", config=config, extra_logging=False)

    assert not error, f"Unexpected error: {error}"
    assert "Configuration is valid." in output


async def test_globals_invalid_type_rejected():
    """An unknown `globals` type is a configuration error."""

    config = """
ubihome:
  name: test_device

globals:
  - id: my_flag
    type: not_a_type
"""
    _, error = await run_ubihome("validate", config=config, extra_logging=False)
    assert "Configuration is invalid:" in error
