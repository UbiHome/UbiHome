import os

import pytest

from utils import run_ubihome

VALID_CONFIG = """
ubihome:
  name: "valid_config"
"""

INVALID_CONFIG = """
ubihome:
  name: "invalid_config"

unknown_platform:
"""


def _remove_if_exists(path: str) -> None:
    try:
        os.remove(path)
    except FileNotFoundError:
        pass


@pytest.mark.asyncio
async def test_error_when_no_configuration_file_found():
    """Without -c and without config.yml/config.yaml present, UbiHome should
    report a clear error instead of panicking."""
    _remove_if_exists("config.yml")
    _remove_if_exists("config.yaml")

    output, error = await run_ubihome("validate", extra_logging=False)

    assert "panicked" not in output
    assert "panicked" not in error
    assert "Configuration is invalid:" in error
    assert "Configuration file not found" in error
    assert "config.yaml" in error


@pytest.mark.asyncio
async def test_uses_config_yml_when_present():
    _remove_if_exists("config.yaml")
    with open("config.yml", "w") as f:
        f.write(VALID_CONFIG)

    try:
        output, error = await run_ubihome("validate", extra_logging=False)
        assert not error, f"Unexpected error: {error}"
        assert "Configuration is valid." in output
    finally:
        _remove_if_exists("config.yml")


@pytest.mark.asyncio
async def test_falls_back_to_config_yaml_when_config_yml_absent():
    _remove_if_exists("config.yml")
    with open("config.yaml", "w") as f:
        f.write(VALID_CONFIG)

    try:
        output, error = await run_ubihome("validate", extra_logging=False)
        assert not error, f"Unexpected error: {error}"
        assert "Configuration is valid." in output
    finally:
        _remove_if_exists("config.yaml")


@pytest.mark.asyncio
async def test_prefers_config_yml_over_config_yaml():
    """When both files exist, config.yml wins - proven by making config.yaml
    invalid and asserting validation still succeeds."""
    with open("config.yml", "w") as f:
        f.write(VALID_CONFIG)
    with open("config.yaml", "w") as f:
        f.write(INVALID_CONFIG)

    try:
        output, error = await run_ubihome("validate", extra_logging=False)
        assert not error, f"Unexpected error: {error}"
        assert "Configuration is valid." in output
    finally:
        _remove_if_exists("config.yml")
        _remove_if_exists("config.yaml")
