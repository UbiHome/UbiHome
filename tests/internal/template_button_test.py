from mock_file import IOMockFactory
from utils import UbiHome, run_ubihome


async def test_template_button_actions(io_mock_factory: IOMockFactory):
    """
    A template button runs its `on_press` automations when it is pressed. Here
    a binary sensor presses the template button, and the button's actions
    press a shell button that writes to a mock file.
    """

    pressed_mock = io_mock_factory.create_mock()
    sensor_mock = io_mock_factory.create_mock()

    CONFIG = f"""
ubihome:
  name: test_device

shell:

button:
  - platform: shell
    name: "Runner"
    id: runner
    command: "echo pressed > {pressed_mock}"
  - platform: template
    name: "Template Button"
    id: template_button
    on_press:
      - button.press: runner

binary_sensor:
  - platform: shell
    name: "Trigger Sensor"
    id: trigger_sensor
    update_interval: 2s
    command: |-
      cat {sensor_mock}
    on_press:
      then:
        - button.press: template_button
"""
    sensor_mock.set_value("false")

    async with UbiHome("run", config=CONFIG):
        # Pressing the binary sensor presses the template button, which runs
        # its on_press action (press the runner button).
        sensor_mock.set_value("true")
        pressed_mock.wait_for_mock_state("pressed")


async def test_template_button_validate():
    """
    A configuration using a template button with an `on_press` action
    validates successfully.
    """

    config = """
ubihome:
  name: test_device

shell:

button:
  - platform: template
    name: "Template Button"
    id: template_button
    on_press:
      - globals.set:
          id: my_flag
          value: true

globals:
  - id: my_flag
    type: bool
    initial_value: false
"""
    output, error = await run_ubihome("validate", config=config, extra_logging=False)

    assert not error, f"Unexpected error: {error}"
    assert "Configuration is valid." in output
