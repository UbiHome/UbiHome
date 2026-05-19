import re
from collections.abc import Mapping
from os import urandom
from typing import Any

import pytest
import yaml
from playwright.async_api import (
    Page,
    expect,
)
from playwright.async_api import (
    TimeoutError as PlaywrightTimeoutError,
)

from mock_file import IOMockFactory
from utils import OS_PLATFORM, Platform, UbiHome

pytestmark = [pytest.mark.e2e, pytest.mark.timeout(30)]


def merge_dicts(base: dict[str, Any], overrides: Mapping[str, Any] | None) -> dict[str, Any]:
    if overrides is None:
        return base

    for key, value in overrides.items():
        current = base.get(key)
        if isinstance(current, dict) and isinstance(value, Mapping):
            merge_dicts(current, value)
        else:
            base[key] = value

    return base


def apply_value_overrides(base: dict[str, Any], overrides: Mapping[str, Any] | None) -> None:
    if overrides is None:
        return

    for path, value in overrides.items():
        parts = path.split(".")
        current: Any = base
        for part in parts[:-1]:
            if isinstance(current, list):
                current = current[int(part)]
            else:
                current = current[part]

        last_part = parts[-1]
        if isinstance(current, list):
            current[int(last_part)] = value
        else:
            current[last_part] = value


async def add_esphome_integration(page: Page, port: int):
    await page.get_by_text("Settings").click()
    await page.get_by_text("Devices & services").click()
    await page.get_by_role("button", name="Add integration").click()
    await page.get_by_placeholder("Search for a brand name").fill("ESPHome")
    await page.locator("ha-integration-list-item").get_by_text("ESPHome", exact=True).first.click()

    setup_another = page.get_by_text("Set up another instance of")
    if await setup_another.count() > 0:
        await setup_another.first.click()

    await page.get_by_role("textbox", name="Host*").fill("host.docker.internal")
    await page.get_by_role("spinbutton", name="Port").fill(str(port))
    await page.get_by_role("button", name="Submit").click()
    try:
        await page.get_by_role("button", name="Skip and finish").click(timeout=5000)
    except PlaywrightTimeoutError:
        pass
    await page.wait_for_url("**/config/devices/device/**")


class UbiHomeInstance(UbiHome):
    def __init__(
        self,
        io_mock_factory: IOMockFactory,
        config_overrides: Mapping[str, Any] | None = None,
        value_overrides: Mapping[str, Any] | None = None,
    ) -> None:
        self.button_sensor_mock = io_mock_factory.create_mock("button_sensor")

        self.sensor_mock = io_mock_factory.create_mock("sensor")
        self.sensor_id = "my_sensor"
        self.sensor_name = "Test Sensor"

        self.switch_mock = io_mock_factory.create_mock("switch")

        self.binary_sensor_mock = io_mock_factory.create_mock("binary_sensor")
        self.binary_sensor_mock.set_value("false")
        self.binary_sensor_id = "my_binary_sensor"
        self.binary_sensor_name = "Test Binary Sensor"

        self.number_mock = io_mock_factory.create_mock("number")
        self.number_mock.set_value("50.0")
        self.number_set_mock = io_mock_factory.create_mock("number_set")
        self.number_name = "Test Number"

        self.text_sensor_mock = io_mock_factory.create_mock("text_sensor")
        self.text_sensor_mock.set_value("initial state")
        self.text_sensor_name = "Test Text Sensor"

        # Generate random readable device name (simply hexadecimal with 15 digits)
        self.device_name = f"test_device_{urandom(15).hex()}"
        config_dict = {
            "api": {},
            "shell": {
                "type": "bash" if OS_PLATFORM is Platform.LINUX else "powershell",
            },
            "ubihome": {
                "name": self.device_name,
            },
            "button": [
                {
                    "platform": "shell",
                    "id": "my_button",
                    "name": "Write Hello World",
                    "command": f"echo 'button' > {self.button_sensor_mock.file}",
                }
            ],
            "switch": [
                {
                    "platform": "shell",
                    "id": "my_switch",
                    "name": "Switch it",
                    "command_on": f"echo true > {self.switch_mock.file}",
                    "command_off": f"echo false > {self.switch_mock.file}",
                    "command_state": f"cat {self.switch_mock.file} || echo false",
                }
            ],
            "sensor": [
                {
                    "platform": "shell",
                    "id": self.sensor_id,
                    "update_interval": "1s",
                    "name": self.sensor_name,
                    "command": f"cat {self.sensor_mock.file}",
                }
            ],
            "binary_sensor": [
                {
                    "platform": "shell",
                    "id": self.binary_sensor_id,
                    "update_interval": "1s",
                    "name": self.binary_sensor_name,
                    "command": f"cat {self.binary_sensor_mock.file}",
                }
            ],
            "number": [
                {
                    "platform": "shell",
                    "id": "my_number",
                    "name": self.number_name,
                    "update_interval": "1s",
                    "command_state": f"cat {self.number_mock.file}",
                    "command_set": f"echo {{{{ value }}}} > {self.number_set_mock.file}",
                }
            ],
            "text_sensor": [
                {
                    "platform": "shell",
                    "id": "my_text_sensor",
                    "name": self.text_sensor_name,
                    "update_interval": "1s",
                    "command": f"cat {self.text_sensor_mock.file}",
                }
            ],
        }
        merged_config = merge_dicts(config_dict, config_overrides)
        apply_value_overrides(merged_config, value_overrides)
        config = yaml.safe_dump(merged_config, sort_keys=False)

        super().__init__("run", config=config, wait_for_api=True)


async def test_components_are_displayed(ha_page: Page, io_mock_factory: IOMockFactory):

    async with UbiHomeInstance(io_mock_factory) as ubihome:
        await add_esphome_integration(ha_page, ubihome.port)

        await expect(ha_page.get_by_text("Switch it", exact=True)).to_be_visible()
        await expect(ha_page.get_by_text("Write Hello World", exact=True)).to_be_visible()
        await expect(ha_page.get_by_text("Test Sensor", exact=True)).to_be_visible()
        await expect(ha_page.get_by_text("Test Binary Sensor", exact=True)).to_be_visible()
        await expect(ha_page.get_by_text("Test Number", exact=True)).to_be_visible()
        await expect(
            ha_page.get_by_text("Test Text Sensor", exact=True)
        ).to_be_visible()


async def test_button_and_switch_actions_are_executed(ha_page: Page, io_mock_factory: IOMockFactory):
    async with UbiHomeInstance(io_mock_factory) as ubihome:
        await add_esphome_integration(ha_page, ubihome.port)

        await ha_page.get_by_role("button", name="Press").click()
        ubihome.button_sensor_mock.wait_for_mock_state("button")

        await ha_page.get_by_role("button", name=f"Turn {ubihome.device_name} Switch it on").click()
        ubihome.switch_mock.wait_for_mock_state("true")


async def test_number_action_is_executed(ha_page: Page, io_mock_factory: IOMockFactory):
    async with UbiHomeInstance(io_mock_factory) as ubihome:
        await add_esphome_integration(ha_page, ubihome.port)

        await ha_page.get_by_text(ubihome.number_name, exact=True).click()

        number_input = ha_page.locator("state-card-number input")
        await number_input.fill("25")
        await number_input.press("Enter")
        ubihome.number_set_mock.wait_for_mock_state("25")


@pytest.mark.parametrize(
    "accuracy_decimals, sensor_value, expected_pattern, forbidden_pattern",
    [
        (4, "9.87654", r"9\.8765", None),
        (0, "123.45", r"\b123\b", r"\.45"),
        (1, "0.8", r"0\.8", r"0\.80"),
    ],
)
async def test_accuracy_decimals_are_displayed_in_ui(
    ha_page: Page,
    io_mock_factory: IOMockFactory,
    accuracy_decimals: int,
    sensor_value: str,
    expected_pattern: str,
    forbidden_pattern: str | None,
):
    async with UbiHomeInstance(
        io_mock_factory,
        value_overrides={"sensor.0.accuracy_decimals": accuracy_decimals},
    ) as ubihome:
        ubihome.sensor_mock.set_value(sensor_value)
        await add_esphome_integration(ha_page, ubihome.port)

        await expect(ha_page.get_by_text("Switch it", exact=True)).to_be_visible()
        await expect(ha_page.get_by_text("Write Hello World", exact=True)).to_be_visible()
        await expect(ha_page.get_by_text("Test Sensor", exact=True)).to_be_visible()

        await ha_page.get_by_text(ubihome.sensor_name, exact=True).click(force=True)

        expected_value = ha_page.get_by_text(re.compile(expected_pattern)).first
        await expect(expected_value).to_be_visible(timeout=15000)

        if forbidden_pattern:
            await expect(ha_page.get_by_text(re.compile(forbidden_pattern))).to_have_count(0)


async def test_text_sensor_state_is_displayed(
    ha_page: Page, io_mock_factory: IOMockFactory
):
    async with UbiHomeInstance(io_mock_factory) as ubihome:
        ubihome.text_sensor_mock.set_value("hello world")
        await add_esphome_integration(ha_page, ubihome.port)

        await expect(
            ha_page.get_by_text(ubihome.text_sensor_name, exact=True)
        ).to_be_visible()

        await ha_page.get_by_text(ubihome.text_sensor_name, exact=True).click(
            force=True
        )

        await expect(
            ha_page.get_by_text("hello world", exact=True).first
        ).to_be_visible(timeout=15000)

        ubihome.text_sensor_mock.set_value("updated state")

        await expect(
            ha_page.get_by_text("updated state", exact=True).first
        ).to_be_visible(timeout=15000)
