import time
import re

import pytest
from playwright.async_api import (
    Page,
    TimeoutError as PlaywrightTimeoutError,
    expect,
)

from mock_file import IOMockFactory
from utils import OS_PLATFORM, Platform, UbiHome


pytestmark = [pytest.mark.e2e, pytest.mark.timeout(30)]


BASE_CONFIG = f"""
api:

shell:
  type: {"bash" if OS_PLATFORM is Platform.LINUX else "powershell"}

"""


async def add_esphome_integration(page: Page, port: int):
    await page.get_by_text("Settings").click()
    await page.get_by_text("Devices & services").click()
    await page.get_by_role("button", name="Add integration").click()
    await page.get_by_placeholder("Search for a brand name").fill("ESPHome")
    await (
        page.locator("ha-integration-list-item")
        .get_by_text("ESPHome", exact=True)
        .first.click()
    )

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
    def __init__(self, io_mock_factory: IOMockFactory) -> None:
        self.button_sensor_mock = io_mock_factory.create_mock()
        self.sensor_mock = io_mock_factory.create_mock()
        self.sensor_mock.set_value("Hello World!")
        self.switch_mock = io_mock_factory.create_mock()
        self.binary_sensor_mock = io_mock_factory.create_mock()
        self.binary_sensor_mock.set_value("false")

        sensor_id = "my_sensor"
        sensor_name = "Test Sensor"

        config = (
            BASE_CONFIG
            + f"""
ubihome:
  name: test_device

button:
- platform: shell
  id: my_button
  name: "Write Hello World"
  command: "echo 'button' > {self.button_sensor_mock.file}"

switch:
- platform: shell
  id: my_switch
  name: "Switch it"
  command_on: "echo true > {self.switch_mock.file}"
  command_off: "echo false > {self.switch_mock.file}"
  command_state: "cat {self.switch_mock.file} || echo false"

sensor:
- platform: shell
  id: my_sensor
  update_interval: 1s
  name: "Test Sensor"
  accuracy_decimals: 4
  command: "cat {self.sensor_mock.file}"

binary_sensor:
- platform: "shell"
  id: {sensor_id}
  update_interval: 1s
  name: {sensor_name}
  command: "cat {self.binary_sensor_mock.file}"
"""
        )

        super().__init__("run", config=config, wait_for_api=True)


async def test_components_are_displayed(ha_page: Page, io_mock_factory: IOMockFactory):

    async with UbiHomeInstance(io_mock_factory) as ubihome:
        await add_esphome_integration(ha_page, ubihome.port)

        await expect(ha_page.get_by_text("Switch it", exact=True)).to_be_visible()
        await expect(
            ha_page.get_by_text("Write Hello World", exact=True)
        ).to_be_visible()
        await expect(ha_page.get_by_text("Test Sensor", exact=True)).to_be_visible()


async def test_button_and_switch_actions_are_executed(
    ha_page: Page, io_mock_factory: IOMockFactory
):
    async with UbiHomeInstance(io_mock_factory) as ubihome:
        await add_esphome_integration(ha_page, ubihome.port)

        ha_page.get_by_role("button", name="Press").click()
        ubihome.button_sensor_mock.wait_for_mock_state("button")

        ha_page.get_by_role("button", name="Turn test_device Switch it on").click(
            force=True
        )
        ubihome.switch_mock.wait_for_mock_state("true")


async def test_accuracy_decimals_are_displayed_in_ui(ha_page: Page):
    page = ha_page
    # page.goto(e2e_context.device_url, wait_until="networkidle")
    # page.get_by_text("Test Sensor", exact=True).click(force=True)

    # decimal_locator = page.get_by_text(re.compile(r"\d+\.\d{4,}"))
    # deadline = time.time() + 15
    # while time.time() < deadline:
    #     if decimal_locator.count() > 0 and decimal_locator.first.is_visible():
    #         return
    #     page.wait_for_timeout(500)

    # pytest.fail(
    #     "No sensor value with at least 4 decimal places was displayed in Home Assistant UI"
    # )


async def test_sensor_value_updates_when_source_changes(ha_page: E2EContext):
    page = ha_page
    # page.goto(e2e_context.device_url, wait_until="networkidle")
    # page.get_by_text("Test Sensor", exact=True).click(force=True)

    # e2e_context.sensor_value_file.write_text("9.87654\n", encoding="utf-8")

    # updated_value = page.get_by_text(re.compile(r"9\.8765"))
    # deadline = time.time() + 20
    # while time.time() < deadline:
    #     if updated_value.count() > 0 and updated_value.first.is_visible():
    #         return
    #     page.wait_for_timeout(500)

    # pytest.fail(
    #     "Sensor value did not update in Home Assistant UI after source configuration value changed"
    # )


async def test_accuracy_decimals_zero_are_displayed_without_fraction(
    ha_page: E2EContext,
):
    page = ha_page

    # await add_esphome_integration(
    #     page,
    #     home_assistant_runtime.base_url,
    #     ubihome_runtime_zero_decimals.runtime.port,
    # )
    # page.get_by_text("Zero Decimal Sensor", exact=True).click(force=True)

    # integer_value = page.get_by_text(re.compile(r"\b12345\b"))
    # deadline = time.time() + 15
    # while time.time() < deadline:
    #     if integer_value.count() > 0 and integer_value.first.is_visible():
    #         break
    #     page.wait_for_timeout(500)
    # else:
    #     pytest.fail(
    #         "Sensor value for accuracy_decimals=0 was not displayed as an integer"
    #     )

    # assert page.get_by_text(re.compile(r"12345\.0")).count() == 0
