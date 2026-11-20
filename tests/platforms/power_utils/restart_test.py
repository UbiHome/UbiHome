import asyncio

from utils import UbiHome


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
