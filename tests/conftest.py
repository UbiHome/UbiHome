from typing import Any, Generator
import pytest

from mock_file import IOMock, IOMockFactory


@pytest.fixture(scope="function")
def io_mock() -> Generator[IOMock, Any, Any]:
    mock_file = IOMock(base_path="")
    yield mock_file
    mock_file.remove()


@pytest.fixture(scope="function")
def io_mock_factory() -> Generator[IOMockFactory, Any, Any]:
    mock_file = IOMockFactory(base_path="")
    yield mock_file
    mock_file.cleanup()


import pytest


def pytest_addoption(parser: pytest.Parser) -> None:
    group = parser.getgroup("ubihome-e2e")
    group.addoption(
        "--headed",
        action="store_true",
        default=False,
        help="Run Playwright browser in headed mode for e2e tests.",
    )
    group.addoption(
        "--trace-on-failure",
        action="store_true",
        default=False,
        help="Save a Playwright trace zip when an e2e test fails.",
    )


@pytest.hookimpl(tryfirst=True, hookwrapper=True)
def pytest_runtest_makereport(item: pytest.Item, call: pytest.CallInfo[None]):
    outcome = yield
    report = outcome.get_result()
    setattr(item, f"rep_{report.when}", report)
