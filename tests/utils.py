import asyncio
from asyncio.subprocess import Process
import logging
import os
import signal
import socket
import platform
import time
from typing import Optional

DEFAULT_CONFIG = """
ubihome:
  name: new_awesome

logger:
  level: DEBUG

shell: 
  type: bash

button: 
 - platform: shell
   id: my_button
   name: "Write Hello World"
   command: "echo 'Hello World!' >> test.log"

sensor:
  - platform: shell
    name: "RAM Usage"
    id: ram_usage
    icon: mdi:memory
    state_class: "measurement"
    unit_of_measurement: "%"
    update_interval: 30s
    command: |-
      cat testfile

gpio:
  device: raspberryPi

mqtt:
  broker: 127.0.0.1
  username: test
  password: test

power_utils:

api:
  encryption:
    key: "token"
"""


class UbiHome(object):
    process: Optional[Process] = None
    _stdout_task = None
    _stderr_task = None
    stdout: str | None = None
    stderr: str | None = None

    def __init__(
        self,
        *arguments,
        port=6053,
        config=None,
        throw_on_error=True,
        wait_for_api=False,
        # executable=None,
    ):
        self.port = port
        self.config = config if config else DEFAULT_CONFIG
        self.arguments = arguments
        self.configuration_file = f"config{os.getpid()}.yaml"
        self.throw_on_error = throw_on_error
        self.wait_for_api = wait_for_api
        if platform.system() == "Windows":
            file = "ubihome.exe"
        else:
            file = "ubihome"
        self.executable = os.path.join(os.getcwd(), "..", "target", "debug", file)
        logging.info(f"Using UbiHome executable: {self.executable}")

    async def __aenter__(self):
        my_env = os.environ.copy()
        my_env["RUST_LOG"] = "TRACE"
        my_env["RUST_BACKTRACE"] = "1"
        my_env["RUSTFLAGS"] = "-Awarnings"
        with open(self.configuration_file, "w") as f:
            f.write(self.config)

        if os.path.exists(self.executable):
            # Use pre-build binaries
            self.process = await asyncio.create_subprocess_exec(
                self.executable,
                "-c",
                self.configuration_file,
                *self.arguments,
                env=my_env,
                stderr=asyncio.subprocess.PIPE,
                stdout=asyncio.subprocess.PIPE,
            )
        else:
            raise Exception(
                f"{self.executable} does not exist. Please build UbiHome first."
            )

        self._stdout_task = asyncio.create_task(self._read_stdout())
        self._stderr_task = asyncio.create_task(self._read_stderr())

        if self.wait_for_api:
            print("Waiting for server to start...")
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            while True:
                result = sock.connect_ex(("127.0.0.1", self.port))
                if result == 0:
                    print("Port is open")
                    break
                else:
                    await asyncio.sleep(0.1)
            sock.close()

        return self

    async def __aexit__(self, exctype, value, tb):
        os.remove(self.configuration_file)
        if self.process:

            # Try to terminate gracefully
            try:
                self.process.terminate()
            except ProcessLookupError:
                pass
            try:
                await asyncio.wait_for(self.process.wait(), timeout=5)
            except asyncio.TimeoutError:
                self.process.kill()
                await self.process.wait()

            # Cancel the stdout/stderr reading tasks
            if self._stdout_task:
                self._stdout_task.cancel()
                try:
                    await self._stdout_task
                except asyncio.CancelledError:
                    pass

            if self._stderr_task:
                self._stderr_task.cancel()
                try:
                    await self._stderr_task
                except asyncio.CancelledError:
                    pass
            # Works on windows?!
            if platform.system() == "Windows":
                os.kill(self.process.pid, signal.CTRL_BREAK_EVENT)

            # self.process = None

    async def _read_stdout(self):
        """Read and print stdout from the server process."""
        self.stdout = ""

        while True:
            if not self.process or not self.process.stdout:
                return
            line = await self.process.stdout.readline()
            if not line:
                break
            message = line.decode("utf-8")
            self.stdout += message
            print(message.rstrip())

    async def _read_stderr(self):
        """Read and print stderr from the server process."""
        self.stderr = ""
        while True:
            if not self.process or not self.process.stderr:
                return
            line = await self.process.stderr.readline()
            if not line:
                break
            message = line.decode("utf-8")
            self.stderr += message
            print(message.rstrip())


async def run_ubihome(*arguments, config=None) -> str:
    async with UbiHome(*arguments, config=config) as ubihome:
        await asyncio.sleep(1)
        return ubihome.stdout or ""


def wait_and_get_file(file_path, timeout=5):
    """
    Wait for a file to be created or modified.
    """
    start_time = time.time()
    while not os.path.exists(file_path):
        if time.time() - start_time > timeout:
            raise TimeoutError(
                f"File {file_path} was not created within {timeout} seconds."
            )
        time.sleep(0.1)
    return open(file_path, "r").read()


def wait_for_mock_state(file_path, expected_state, timeout=5):
    """
    Wait for a file to be created or modified.
    """
    state = None
    start_time = time.time()
    while expected_state != state:
        if time.time() - start_time > timeout:
            raise TimeoutError(
                f"State does not match within {timeout} seconds: {expected_state} != {state}."
            )
        while not os.path.exists(file_path):
            if time.time() - start_time > timeout:
                raise TimeoutError(
                    f"File {file_path} was not created within {timeout} seconds."
                )
            time.sleep(0.1)

        state = open(file_path, "r").read()
        time.sleep(0.1)

    return True
