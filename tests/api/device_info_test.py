
from tests.utils import UbiHome
import aioesphomeapi


async def test_run():
  name = "test_device_info"
  DEVICE_INFO_CONFIG = f"""
ubihome:
  name: {name}


# Remove:
sensor:
binary_sensor:
gpio:
  device: raspberryPi
shell:
mqtt:
  broker: 127.0.0.1
  username: test
  password: test
power_utils:
"""

  async with UbiHome("run", DEVICE_INFO_CONFIG) as ubihome:
    api = aioesphomeapi.APIClient("127.0.0.1", 6053, "MyPassword")
    await api.connect(login=True)

    # Show device details
    device_info = await api.device_info()
    assert device_info.name == name