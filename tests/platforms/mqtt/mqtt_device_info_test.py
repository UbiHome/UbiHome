import json
from asyncio import sleep
from unittest.mock import Mock

import pytest
from paho.mqtt.client import Client

from utils import UbiHome


@pytest.mark.timeout(60)
async def test_run(mqtt_client: Client, mqtt_connection):
    name = "test_device_info"
    DEVICE_INFO_CONFIG = f"""
ubihome:
  name: {name}

mqtt:
  broker: {mqtt_connection["host"]}
  port: {mqtt_connection["port"]}
"""

    mqtt_client.subscribe("#")
    mock = Mock()
    mqtt_client._on_message = mock
    async with UbiHome("run", config=DEVICE_INFO_CONFIG):
        await sleep(1)
        mock.assert_called_once()
        message = mock.call_args.args[2]
        assert message.topic == f"homeassistant/device/{name}/config"
        config_message = json.loads(message.payload)
        assert config_message["device"]["name"] == name
