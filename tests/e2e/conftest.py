import contextlib
import time
import json
from typing import AsyncGenerator
import urllib.parse
from collections.abc import Generator
from urllib.request import Request, urlopen

import pytest
import pytest_asyncio
from playwright.async_api import BrowserContext, Page
from pytest_playwright_asyncio.pytest_playwright import CreateContextCallback

from testcontainers.core.container import DockerContainer
from utils import HomeAssistantRuntime


def _api_request(
    url: str, *, method: str = "GET", data=None, headers=None, form: bool = False
):
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


@pytest.fixture(scope="session")
def home_assistant_runtime() -> Generator[HomeAssistantRuntime, None, None]:

    container = DockerContainer("ghcr.io/home-assistant/home-assistant:2026.4.3")
    container.with_bind_ports(8123, None)
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
        base_url=base_url,
        username=username,
        password=password,
    )
    yield runtime

    container.stop()


@pytest_asyncio.fixture(scope="function")
async def ha_page(
    new_context: CreateContextCallback,
    home_assistant_runtime: HomeAssistantRuntime,
) -> AsyncGenerator[Page, None]:
    context = await new_context()
    page = await context.new_page()

    await page.goto(f"{home_assistant_runtime.base_url}/", wait_until="networkidle")
    await page.get_by_role("textbox", name="Username").fill(
        home_assistant_runtime.username
    )
    await page.get_by_role("textbox", name="Password").fill(
        home_assistant_runtime.password
    )
    await page.get_by_role("textbox", name="Password").press("Enter")
    await page.wait_for_url("**/home/overview", timeout=60000)

    yield page
    context.close()
