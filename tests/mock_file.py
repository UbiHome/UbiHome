import os
from uuid import uuid4
import time
import platform


class IOMock:

    def __init__(self, base_path: str | None = None) -> None:
        self._base_path = base_path
        self._file = f"{uuid4()}.mock"
        self._full_path = self._file  # os.path.join(base_path, self._file)

    def __str__(self) -> str:
        return self._full_path

    @property
    def file(self) -> str:
        return self._full_path

    def set_value(self, content: str) -> None:
        with open(self._full_path, "w") as f:
            f.write(content)

    def wait_and_get_file(self, timeout=5):
        """
        Wait for a file to be created or modified.
        """
        start_time = time.time()
        while not os.path.exists(self._full_path):
            if time.time() - start_time > timeout:
                raise TimeoutError(
                    f"File {self._full_path} was not created within {timeout} seconds."
                )
            time.sleep(0.1)
        return open(self._full_path, "r").read()

    def wait_for_mock_state(self, expected_state, timeout=5):
        """
        Wait for a file to be created or modified.
        """
        state: str = ""
        start_time = time.time()
        while expected_state in state:
            if time.time() - start_time > timeout:
                raise TimeoutError(
                    f"State does not match within {timeout} seconds: {expected_state} != {state}."
                )
            while not os.path.exists(self._full_path):
                if time.time() - start_time > timeout:
                    raise TimeoutError(
                        f"File {self._full_path} was not created within {timeout} seconds."
                    )
                time.sleep(0.1)

            if platform.system() == "Windows":
                # On Windows, read with utf-16 encoding
                try:
                    state = open(self._full_path, "r", encoding="utf-16").read()
                except UnicodeError:
                    state = open(self._full_path, "r").read()
                state = state.encode("utf-8").decode("utf-8")
            else:
                state = open(self._full_path, "r").read()
            time.sleep(0.1)

        return True

    def remove(self) -> None:
        """Remove the mock file."""
        if os.path.exists(self._full_path):
            os.remove(self._full_path)


class IOMockFactory:
    def __init__(self, base_path: str | None = None) -> None:
        self._base_path = base_path
        self.mock_files: list[IOMock] = []

    def create_mock(self) -> IOMock:
        return IOMock(self._base_path)

    def cleanup(self) -> None:
        for mock_file in self.mock_files:
            mock_file.remove()
