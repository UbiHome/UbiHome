"""Whole-file validation: the configuration is parsed and validated in a single
pass, so references between entities (actions -> switches/buttons/globals,
template lambdas -> globals) and platform availability are checked at
validate time, with the error pointing at the offending line."""

import re

import pytest

from utils import run_ubihome

VALID_CROSS_PLATFORM = """
ubihome:
  name: test_refs_valid

shell:
  type: bash

globals:
  - id: power
    type: bool
  - id: level
    type: float

switch:
  - platform: template
    id: power_switch
    lambda:
      globals.get: power
  - platform: shell
    id: fan
    command_on: "true"
    command_off: "true"

button:
  - platform: template
    id: toggle_all
    on_press:
      then:
        - switch.turn_on: fan
        - globals.set:
            id: power
            value: true
        - globals.set:
            id: level
            value: 3

number:
  - platform: template
    id: level_number
    lambda:
      globals.get: level

binary_sensor:
  - platform: shell
    name: Door
    command: "echo ON"
    on_press:
      then:
        - switch.turn_off: power_switch
        - button.press: toggle_all
"""


async def validate(config: str) -> str:
    """Run `validate` and return stderr with miette's line wrapping undone."""
    _, error = await run_ubihome("validate", config=config, extra_logging=False)
    return re.sub(r"\s*\n\s*│\s*", " ", error)


def underlined(error: str, value: str) -> bool:
    """Whether the report's source snippet points at `value` (the line holding
    it is followed by an underline starting at its column)."""
    lines = error.splitlines()
    for line, marker in zip(lines, lines[1:], strict=False):
        column = line.find(value)
        if column >= 0 and marker[column : column + 1] in ("─", "┬"):
            return True
    return False


@pytest.mark.asyncio
async def test_valid_cross_platform_references():
    """A shell binary sensor may drive a template switch/button and vice versa."""
    output, error = await run_ubihome("validate", config=VALID_CROSS_PLATFORM, extra_logging=False)
    assert not error, f"Unexpected error: {error}"
    assert 'Platforms to load: ["shell"]' in output
    assert "Configuration is valid." in output


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "replace,expected,points_at",
    [
        (
            ("switch.turn_off: power_switch", "switch.turn_off: mispelled_switcch"),
            [
                "Unknown switch 'mispelled_switcch'. Configured switches are: fan, power_switch",
                "`binary_sensor[0].on_press.then[0].switch.turn_off`",
            ],
            "mispelled_switcch",
        ),
        (
            ("button.press: toggle_all", "button.press: ghost"),
            [
                "Unknown button 'ghost'. Configured buttons are: toggle_all",
                "`binary_sensor[0].on_press.then[1].button.press`",
            ],
            "ghost",
        ),
        (
            ("value: true", "value: 'true'"),
            [
                "Invalid value for global 'power' of type 'bool': expected a boolean, got 'true'",
                "`button[0].on_press.then[1].globals.set.value`",
            ],
            "'true'",
        ),
        (
            ("id: level\n            value: 3", "id: nope\n            value: 3"),
            [
                "Unknown global 'nope'. Configured globals are: level, power",
                "`button[0].on_press.then[2].globals.set.id`",
            ],
            "nope",
        ),
        (
            ("globals.get: power", "globals.get: level"),
            [
                "Global 'level' has type 'float', but a template switch lambda needs a global of type 'bool'",
                "`switch[0].lambda`",
            ],
            "globals.get",
        ),
    ],
)
async def test_invalid_references(replace, expected, points_at):
    old, new = replace
    assert old in VALID_CROSS_PLATFORM
    error = await validate(VALID_CROSS_PLATFORM.replace(old, new, 1))
    assert "Configuration is invalid:" in error
    for text in expected:
        assert text in error, f"Expected '{text}' in: {error}"
    # The snippet points at the offending value.
    assert underlined(error, points_at), error


@pytest.mark.asyncio
async def test_all_errors_reported_in_one_run():
    """Errors from different platforms/sections are collected in one report
    instead of stopping at the first failing module."""
    config = (
        VALID_CROSS_PLATFORM.replace("switch.turn_off: power_switch", "switch.turn_off: nope_switch")
        .replace("button.press: toggle_all", "button.press: nope_button")
        .replace("globals.get: level", "globals.get: nope_global")
    )
    error = await validate(config)
    assert "validation failed (multiple errors)" in error
    for text in ["nope_switch", "nope_button", "nope_global"]:
        assert text in error, f"Expected '{text}' in: {error}"


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "config,expected",
    [
        (
            # mqtt is configured, but has no buttons
            """
ubihome:
  name: test_unsupported

mqtt:
  broker: 127.0.0.1

button:
  - platform: mqtt
    name: Bad Button
""",
            ["Platform 'mqtt' does not provide 'button' entities", "`button[0].platform`"],
        ),
        (
            # gpio is compiled in, but not configured
            """
ubihome:
  name: test_not_configured

shell:

switch:
  - platform: gpio
    id: relay
    pin: 7
""",
            [
                "Platform 'gpio' is not configured in the configuration file. Allowed platforms are: shell, template",
                "`switch[0].platform`",
            ],
        ),
        (
            """
ubihome:
  name: test_duplicates

shell:

button:
  - platform: shell
    id: dup
    command: "true"
  - platform: shell
    id: dup
    command: "true"
""",
            ["Duplicate button id 'dup' (already used by button[0])", "`button[1]`"],
        ),
    ],
)
async def test_invalid_platform_usage(config, expected):
    error = await validate(config)
    assert "Configuration is invalid:" in error
    for text in expected:
        assert text in error, f"Expected '{text}' in: {error}"


@pytest.mark.asyncio
async def test_deserialization_error_points_at_value():
    """Entity fields are streamed to the platform config type, so a type error
    points at the offending value rather than the end of the entity."""
    config = """
ubihome:
  name: test_location

shell:

sensor:
  - platform: shell
    name: RAM
    command: "free"
    update_interval: notaduration
    unit_of_measurement: "%"
"""
    error = await validate(config)
    assert "Configuration is invalid:" in error
    assert underlined(error, "notaduration"), error
