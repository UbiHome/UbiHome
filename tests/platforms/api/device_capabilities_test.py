import asyncio
import base64
import os

import aioesphomeapi

from utils import UbiHome


async def test_device_capabilities():
    encryption_key = base64.b64encode(os.urandom(32)).decode("utf-8")

    DEVICE_CAPABILITIES_CONFIG = f"""
ubihome:
  name: test_device_capabilities

api:
  encryption:
    key: "{encryption_key}"
"""

    async with UbiHome(
        "run", config=DEVICE_CAPABILITIES_CONFIG, wait_for_api=True
    ) as ubihome:
        api = aioesphomeapi.APIClient(
            "127.0.0.1", ubihome.port, None, noise_psk=encryption_key
        )
        await api.connect(login=False)

        # Home Assistant waits for this response before subscribing to states.
        capabilities = await asyncio.wait_for(api.device_capabilities(), timeout=5)
        assert capabilities.bluetooth_proxy is not None
        assert capabilities.bluetooth_proxy.feature_flags == 1

        await api.disconnect()
