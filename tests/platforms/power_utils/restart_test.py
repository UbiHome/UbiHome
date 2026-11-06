import asyncio
import os
import shutil

import pytest

from utils import OS_PLATFORM, Platform, UbiHome


async def test_restart_service_button():
    """
    Test that the power_utils `restart_service` button restarts the UbiHome
    process (not the machine).

    The process re-executes itself (same PID, same stdout), so the startup
    banner is printed again. `on_startup` presses the button after a short
    delay, so the banner appearing a second time proves the process restarted.
    """

    DEVICE_INFO_CONFIG = """
ubihome:
  name: test_device
  on_startup:
    then:
      - delay: 2s
      - button.press: restart_service_btn

power_utils:

button:
  - platform: power_utils
    name: "Restart UbiHome"
    id: restart_service_btn
    action: restart_service
"""

    async with UbiHome("run", config=DEVICE_INFO_CONFIG) as ubihome:

        async def wait_for_restart():
            while (ubihome.stdout or "").count("UbiHome - ") < 2:
                await asyncio.sleep(0.1)

        await asyncio.wait_for(wait_for_restart(), timeout=15)


@pytest.mark.skipif(OS_PLATFORM is Platform.WINDOWS, reason="A running executable cannot be replaced on Windows")
async def test_restart_service_button_after_binary_was_replaced(tmp_path):
    """
    Test that `restart_service` still works after `ubihome update` swapped the
    binary of the running process.

    Like `self_replace`, the running binary is replaced with a rename, so on
    Linux the process image becomes "<path> (deleted)".
    """

    DEVICE_INFO_CONFIG = """
ubihome:
  name: test_device
  on_startup:
    then:
      - delay: 2s
      - button.press: restart_service_btn

power_utils:

button:
  - platform: power_utils
    name: "Restart UbiHome"
    id: restart_service_btn
    action: restart_service
"""

    executable = tmp_path / "ubihome"
    updated_executable = tmp_path / "new_ubihome"
    shutil.copy2("ubihome", executable)
    shutil.copy2("ubihome", updated_executable)

    async with UbiHome("run", config=DEVICE_INFO_CONFIG, executable=str(executable)) as ubihome:

        async def wait_for_start(count):
            while (ubihome.stdout or "").count("UbiHome - ") < count:
                await asyncio.sleep(0.1)

        await asyncio.wait_for(wait_for_start(1), timeout=15)
        os.replace(updated_executable, executable)
        await asyncio.wait_for(wait_for_start(2), timeout=15)
