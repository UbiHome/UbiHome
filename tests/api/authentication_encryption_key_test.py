import asyncio
import os

from pytest import mark
from utils import UbiHome
import aioesphomeapi
import base64


async def test_right_key():
  encryption_key = base64.b64encode(os.urandom(32)).decode("utf-8")

  DEVICE_INFO_CONFIG = f"""
ubihome:
  name: test_device
api:
  encryption_key: "{encryption_key}"

"""

  async with UbiHome("run", DEVICE_INFO_CONFIG) as ubihome:
    api = aioesphomeapi.APIClient("127.0.0.1", 6053, None, noise_psk=encryption_key)
    await api.connect(login=False)

    entities, services = await api.list_entities_services()
    print("switches", entities, services)
    assert len(entities) == 0, entities

    await api.disconnect()


@mark.skip()
async def test_wrong_key():
  encryption_key = "px7tsbK3C7bpXHr2OevEV2ZMg/FrNBw2+O2pNPbedtA="

  DEVICE_INFO_CONFIG = f"""
ubihome:
  name: test_device
api:
  encryption_key: "{encryption_key}"

"""

  async with UbiHome("run", DEVICE_INFO_CONFIG, throw_on_error=False) as ubihome:
    api = aioesphomeapi.APIClient("127.0.0.1", 6053, None, noise_psk="RcaiIwmN008EoAE7KkN2qCXic+hm540EhLvD30EnhhE=")
    await api.connect(login=False)

    entities, services = await api.list_entities_services()
    print("switches", entities, services)
    assert len(entities) == 0, entities

    await api.disconnect()