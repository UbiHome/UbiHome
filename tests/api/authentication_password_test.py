
from asyncio import sleep
import contextlib
import os
from unittest.mock import Mock

import pytest
from utils import UbiHome
import aioesphomeapi
from utils import wait_and_get_file


async def test_right_password():
  password = "SecretPassword"
  name = "Passwordprotected Device"

  DEVICE_INFO_CONFIG = f"""
ubihome:
  name: {name}

api:
  password: "{password}"

"""

  async with UbiHome("run", DEVICE_INFO_CONFIG) as ubihome:
    api = aioesphomeapi.APIClient("127.0.0.1", 6053, password)
    await api.connect(login=True)

    info = await api.device_info()
    assert info.name == name

    entities, services = await api.list_entities_services()
    print("switches", entities, services)
    assert len(entities) == 0, entities


async def test_wrong_password():
  password = "SecretPassword"
  name = "Passwordprotected Device"

  DEVICE_INFO_CONFIG = f"""
ubihome:
  name: {name}

api:
  password: "{password}"

"""

  async with UbiHome("run", DEVICE_INFO_CONFIG) as ubihome:
    api = aioesphomeapi.APIClient("127.0.0.1", 6053, "WrongPassword")
    with pytest.raises(aioesphomeapi.core.InvalidAuthAPIError):
      await api.connect(login=True)

