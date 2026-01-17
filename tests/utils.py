import asyncio
from asyncio.subprocess import Process
from enum import Enum
import logging
import os
import re
import signal
import socket
import platform
import time
from typing import Optional
import yaml
import random


def represent_none(self, _):
    return self.represent_scalar("tag:yaml.org,2002:null", "")


yaml.add_representer(type(None), represent_none)


class Platform(Enum):
    WINDOWS = "Windows"
    LINUX = "Linux"
    MACOS = "Darwin"


def os_platform() -> Platform:
    platform_str = platform.system()
    if platform_str == "Linux":
        return Platform.LINUX
    elif platform_str == "Darwin":
        return Platform.MACOS
    elif platform_str == "Windows":
        return Platform.WINDOWS
    else:
        raise ValueError(f"Unsupported platform: {platform_str}")


OS_PLATFORM = os_platform()

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
    """Context manager to run UbiHome with a given configuration."""

    process: Optional[Process] = None
    _stdout_task = None
    _stderr_task = None
    stdout: str | None = None
    stderr: str | None = None
    port: int | None = None

    def __init__(
        self,
        *arguments,
        config=None,
        throw_on_error=True,
        wait_for_api=False,
        # executable=None,
    ):
        self.arguments = arguments
        self.configuration_file = f"config{os.getpid()}.yaml"
        self.throw_on_error = throw_on_error
        self.wait_for_api = wait_for_api
        if platform.system() == "Windows":
            file = "ubihome.exe"
        else:
            file = "ubihome"
        # self.executable = os.path.join(os.getcwd(), "..", "target", "debug", file)
        self.executable = os.path.join(os.getcwd(), file)
        logging.info("Using UbiHome executable: %s", self.executable)

        config_yaml = yaml.safe_load(config if config else DEFAULT_CONFIG)
        if "api" in config_yaml:
            if config_yaml["api"] is None:
                config_yaml["api"] = {}
            self.port = random.randint(1024, 65535)
            config_yaml["api"]["port"] = self.port

        self.config = yaml.dump(config_yaml)

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

        if self.port and self.wait_for_api:
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


# FROM https://github.com/esphome/esphome/blob/58a9e30017b7094c9cf8bfb0739b610ba5bcd450/esphome/helpers.py#L65

FNV1_OFFSET_BASIS = 2166136261
FNV1_PRIME = 16777619


def snake_case(value):
    """Same behaviour as `helpers.cpp` method `str_snake_case`."""
    return value.replace(" ", "_").lower()


_DISALLOWED_CHARS = re.compile(r"[^a-zA-Z0-9-_]")


def sanitize(value):
    """Same behaviour as `helpers.cpp` method `str_sanitize`."""
    return _DISALLOWED_CHARS.sub("_", value)


def fnv1_hash(string: str) -> int:
    """FNV-1 32-bit hash function (multiply then XOR)."""
    hash_value = FNV1_OFFSET_BASIS
    for char in string:
        hash_value = (hash_value * FNV1_PRIME) & 0xFFFFFFFF
        hash_value ^= ord(char)
    return hash_value


def fnv1_hash_object_id(name: str) -> int:
    """Compute FNV-1 hash of name with snake_case + sanitize transformations.

    IMPORTANT: Must produce same result as C++ fnv1_hash_object_id() in helpers.h.
    Used for pre-computing entity object_id hashes at code generation time.
    """
    return fnv1_hash(sanitize(snake_case(name)))
