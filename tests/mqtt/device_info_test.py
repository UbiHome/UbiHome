
import json
from unittest.mock import Mock
from utils import UbiHome
from paho.mqtt.client import Client

async def test_run(mqtt_client: Client, mqtt_connection):
  name = "test_device_info"
  DEVICE_INFO_CONFIG = f"""
ubihome:
  name: {name}

mqtt:
  broker: {mqtt_connection['host']}
  port: {mqtt_connection['port']}
"""

  mqtt_client.subscribe("#")
  mock = Mock()
  mqtt_client._on_message = mock
  async with UbiHome("run", DEVICE_INFO_CONFIG) as ubihome:
    mock.assert_called_once()
    message = mock.call_args.args[2]
    assert message.topic == f"homeassistant/device/{name}/config"
    config_message = json.loads(message.payload)
    assert config_message["device"]["name"] == name