import os
import platform
import socket
import subprocess
import time
import json
import re
import urllib.parse
from dataclasses import dataclass
from pathlib import Path
import tempfile
from urllib.request import Request, urlopen

import pytest
from playwright.sync_api import Browser, Page, sync_playwright
from testcontainers.core.container import DockerContainer


if not os.getenv("RUN_PLAYWRIGHT_E2E"):
    pytest.skip(
        "Set RUN_PLAYWRIGHT_E2E=1 to run Playwright Home Assistant e2e tests",
        allow_module_level=True,
    )


pytestmark = [pytest.mark.e2e, pytest.mark.timeout(300)]


@dataclass
class UbiHomeRuntime:
    process: subprocess.Popen
    temp_dir: tempfile.TemporaryDirectory
    button_log: Path
    switch_log: Path


@dataclass
class HomeAssistantRuntime:
    container: DockerContainer
    base_url: str
    username: str
    password: str


@dataclass
class E2EContext:
    browser: Browser
    page: Page
    device_url: str
    button_log: Path
    switch_log: Path


def _wait_for_tcp_port(host: str, port: int, timeout_seconds: float) -> None:
    deadline = time.time() + timeout_seconds
    while time.time() < deadline:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.settimeout(1)
            if sock.connect_ex((host, port)) == 0:
                return
        time.sleep(0.2)
    raise TimeoutError(f"Port {host}:{port} did not open in time")


def _wait_for_http_ok(url: str, timeout_seconds: float) -> None:
    deadline = time.time() + timeout_seconds
    while time.time() < deadline:
        try:
            with urlopen(url, timeout=5) as response:  # nosec B310
                if 200 <= response.status < 500:
                    return
        except Exception:
            pass
        time.sleep(1)
    raise TimeoutError(f"Home Assistant did not become reachable: {url}")


def _api_request(url: str, *, method: str = "GET", data=None, headers=None, form: bool = False):
    """Send an HTTP request and return a parsed JSON payload."""
    if data is None:
        payload = None
    elif form:
        payload = urllib.parse.urlencode(data).encode()
    else:
        payload = json.dumps(data).encode()
    request = Request(url, method=method, data=payload, headers=headers or {})
    with urlopen(request, timeout=10) as response:  # nosec B310
        body = response.read().decode()
        return json.loads(body) if body else {}


@pytest.fixture(scope="session")
def ubihome_runtime() -> UbiHomeRuntime:
    if platform.system() != "Linux":
        pytest.skip("Home Assistant docker e2e tests only run on Linux")

    repo_root = Path(__file__).resolve().parents[2]
    tests_root = repo_root / "tests"
    executable = tests_root / "ubihome"
    if not executable.exists():
        raise FileNotFoundError(f"Missing executable: {executable}. Run `make prepare-test-linux` first.")

    temp_dir = tempfile.TemporaryDirectory(prefix="ubihome-ha-e2e-", dir=tempfile.gettempdir())
    base = Path(temp_dir.name)
    button_log = base / "button.log"
    switch_log = base / "switch.log"
    config_path = base / "config.yaml"

    config_path.write_text(
        f"""
ubihome:
  name: test_device

api:
  port: 6053

shell:
  type: bash

button:
 - platform: shell
   id: my_button
   name: "Write Hello World"
   command: "echo 'button' > {button_log}"

switch:
 - platform: shell
   id: my_switch
   name: "Switch it"
   command_on: "echo true > {switch_log}"
   command_off: "echo false > {switch_log}"
   command_state: "cat {switch_log} || echo false"

sensor:
 - platform: shell
   id: my_sensor
   update_interval: 1s
   name: "Test Sensor"
   accuracy_decimals: 4
   command: "echo 1.23456"
""".strip()
        + "\n",
        encoding="utf-8",
    )

    process = subprocess.Popen(
        [str(executable), "-c", str(config_path), "run"],
        cwd=str(tests_root),
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    _wait_for_tcp_port("127.0.0.1", 6053, timeout_seconds=30)

    runtime = UbiHomeRuntime(
        process=process,
        temp_dir=temp_dir,
        button_log=button_log,
        switch_log=switch_log,
    )
    yield runtime

    process.terminate()
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=5)
    temp_dir.cleanup()


@pytest.fixture(scope="session")
def home_assistant_runtime() -> HomeAssistantRuntime:
    config_dir = tempfile.mkdtemp(prefix="home-assistant-e2e-", dir=tempfile.gettempdir())

    container = DockerContainer("ghcr.io/home-assistant/home-assistant:stable")
    container.with_bind_ports(8123, None)
    container.with_volume_mapping(config_dir, "/config", mode="rw")
    container.with_kwargs(extra_hosts={"host.docker.internal": "host-gateway"})
    container.start()

    host = container.get_container_host_ip()
    port = int(container.get_exposed_port(8123))
    base_url = f"http://{host}:{port}"
    _wait_for_http_ok(f"{base_url}/onboarding.html", timeout_seconds=180)

    username = "testuser"
    password = "testpass123!"

    onboarding_user = _api_request(
        f"{base_url}/api/onboarding/users",
        method="POST",
        data={
            "client_id": f"{base_url}/",
            "name": "Test User",
            "username": username,
            "password": password,
            "language": "en",
        },
        headers={"Content-Type": "application/json"},
    )
    token_response = _api_request(
        f"{base_url}/auth/token",
        method="POST",
        data={
            "grant_type": "authorization_code",
            "code": onboarding_user["auth_code"],
            "client_id": f"{base_url}/",
        },
        headers={"Content-Type": "application/x-www-form-urlencoded"},
        form=True,
    )
    auth_headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {token_response['access_token']}",
    }
    _api_request(
        f"{base_url}/api/onboarding/core_config",
        method="POST",
        data={
            "location_name": "Home",
            "latitude": 52.52,
            "longitude": 13.405,
            "elevation": 34,
            "unit_system": "metric",
            "time_zone": "Europe/Berlin",
            "country": "DE",
            "currency": "EUR",
        },
        headers=auth_headers,
    )
    _api_request(
        f"{base_url}/api/onboarding/analytics",
        method="POST",
        data={"base": False, "diagnostics": False, "usage": False, "statistics": False},
        headers=auth_headers,
    )
    _api_request(
        f"{base_url}/api/onboarding/integration",
        method="POST",
        data={"client_id": f"{base_url}/", "redirect_uri": f"{base_url}/"},
        headers=auth_headers,
    )

    runtime = HomeAssistantRuntime(
        container=container,
        base_url=base_url,
        username=username,
        password=password,
    )
    yield runtime

    container.stop()


@pytest.fixture(scope="session")
def e2e_context(
    ubihome_runtime: UbiHomeRuntime,
    home_assistant_runtime: HomeAssistantRuntime,
) -> E2EContext:
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(headless=True)
        page = browser.new_page()

        page.goto(f"{home_assistant_runtime.base_url}/", wait_until="networkidle")
        page.locator("input[name='username']").fill(home_assistant_runtime.username)
        page.locator("input[name='password']").fill(home_assistant_runtime.password)
        page.locator("input[name='password']").press("Enter")
        page.wait_for_url("**/home/overview", timeout=60000)

        page.goto(f"{home_assistant_runtime.base_url}/config/integrations/dashboard", wait_until="networkidle")
        page.get_by_role("button", name="Add integration").click()
        page.get_by_placeholder("Search for a brand name").fill("ESPHome")
        page.get_by_text("ESPHome", exact=True).first.click()

        page.get_by_role("textbox", name="Host*").fill("host.docker.internal")
        page.get_by_role("spinbutton", name="Port").fill("6053")
        page.get_by_role("button", name="Submit").click()
        page.get_by_role("button", name="Skip and finish").click()

        page.wait_for_url("**/config/devices/device/**")

        yield E2EContext(
            browser=browser,
            page=page,
            device_url=page.url,
            button_log=ubihome_runtime.button_log,
            switch_log=ubihome_runtime.switch_log,
        )

        browser.close()


def test_components_are_displayed(e2e_context: E2EContext):
    page = e2e_context.page
    page.goto(e2e_context.device_url, wait_until="networkidle")

    assert page.get_by_text("Switch it", exact=True).is_visible()
    assert page.get_by_text("Write Hello World", exact=True).is_visible()
    assert page.get_by_text("Test Sensor", exact=True).is_visible()


def test_button_and_switch_actions_are_executed(e2e_context: E2EContext):
    page = e2e_context.page
    page.goto(e2e_context.device_url, wait_until="networkidle")

    if e2e_context.button_log.exists():
        e2e_context.button_log.unlink()
    if e2e_context.switch_log.exists():
        e2e_context.switch_log.unlink()

    page.get_by_role("button", name="Press").click()

    deadline = time.time() + 10
    while time.time() < deadline and not e2e_context.button_log.exists():
        time.sleep(0.2)
    assert e2e_context.button_log.exists(), "Button action did not create log file"

    page.get_by_role("button", name="Turn test_device Switch it on").click(force=True)
    deadline = time.time() + 10
    while time.time() < deadline:
        if e2e_context.switch_log.exists() and e2e_context.switch_log.read_text(encoding="utf-8").strip() == "true":
            break
        time.sleep(0.2)
    assert e2e_context.switch_log.exists(), "Switch action did not create log file"
    assert e2e_context.switch_log.read_text(encoding="utf-8").strip() == "true"


def test_accuracy_decimals_are_displayed_in_ui(e2e_context: E2EContext):
    page = e2e_context.page
    page.goto(e2e_context.device_url, wait_until="networkidle")
    page.get_by_text("Test Sensor", exact=True).click(force=True)

    decimal_locator = page.get_by_text(re.compile(r"\d+\.\d{4,}"))
    deadline = time.time() + 15
    while time.time() < deadline:
        if decimal_locator.count() > 0 and decimal_locator.first.is_visible():
            return
        page.wait_for_timeout(500)

    pytest.fail("No sensor value with at least 4 decimal places was displayed in Home Assistant UI")
