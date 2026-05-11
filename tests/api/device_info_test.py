import aioesphomeapi
from utils import UbiHome


async def test_run():
    name = "test_device_info"
    DEVICE_INFO_CONFIG = f"""
ubihome:
  name: {name}

api:
"""

    # TODO: Remove throw_on_error=False once ubihome respects the SubscribeStatesRequest
    async with UbiHome("run", config=DEVICE_INFO_CONFIG, wait_for_api=True, throw_on_error=False) as ubihome:
        api = aioesphomeapi.APIClient("127.0.0.1", ubihome.port, "")
        await api.connect(login=False)

        # Show device details
        device_info = await api.device_info()
        assert device_info.name == name
