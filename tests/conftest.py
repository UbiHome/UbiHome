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
