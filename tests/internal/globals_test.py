import re

from utils import run_ubihome


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
    initial_value: false

switch:
  - platform: template
    name: "Flag Switch"
    id: flag_switch
    optimistic: true
    turn_on_action:
      then:
        - globals.set:
            id: my_flag
            value: true
    turn_off_action:
      then:
        - globals.set:
            id: my_flag
            value: false
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
    initial_value: false

switch:
  - platform: template
    name: "Relay"
    id: relay
    lambda:
      globals.get: relay_state
    turn_on_action:
      then:
        - globals.set:
            id: relay_state
            value: true
    turn_off_action:
      then:
        - globals.set:
            id: relay_state
            value: false
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


async def test_globals_initial_value_type_mismatch_rejected():
    """
    An `initial_value` that doesn't match the global's declared `type` is a
    configuration error, e.g. a number for a `bool` global.
    """

    config = """
ubihome:
  name: test_device

globals:
  - id: light_on
    type: bool
    initial_value: 42
"""
    _, error = await run_ubihome("validate", config=config, extra_logging=False)
    assert "Configuration is invalid:" in error
    assert "globals[0].initial_value" in error
    # A source line/column (e.g. `[config.yaml:8:20]`) should be reported, not
    # just a bare message, since this is wired into the same garde/miette
    # diagnostics pipeline as every other config validation error.
    assert re.search(r"\.yaml:\d+:\d+\]", error), error
