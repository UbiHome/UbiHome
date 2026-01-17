from asyncio import sleep
import json
import os
from unittest.mock import Mock
from mock_file import IOMock
from utils import UbiHome
from paho.mqtt.client import Client


async def test_button_triggered(mqtt_client: Client, mqtt_connection, io_mock: IOMock):
    name = "test_device"
    button_id = "my_button"
    button_name = "Write Hello World"
    DEVICE_INFO_CONFIG = f"""
ubihome:
  name: {name}

mqtt:
  broker: {mqtt_connection['host']}
  port: {mqtt_connection['port']}

shell:
  
button: 
 - platform: shell
   id: {button_id}
   name: {button_name}
   command: "echo 'Hello World!' > {io_mock}"

"""

    mqtt_client.subscribe("#")
    mock = Mock()
    mqtt_client._on_message = mock
    async with UbiHome("run", config=DEVICE_INFO_CONFIG) as ubihome:
        await sleep(1)
        mock.assert_called_once()
        message = mock.call_args.args[2]
        assert message.topic == f"homeassistant/device/{name}/config"
        config_message = json.loads(message.payload)
        print("config_message", config_message)
        components = config_message["components"]
        assert len(components) == 1
        assert button_id in components
        entity = components[button_id]
        assert entity["name"] == button_name
        command_topic = entity["cmd_t"]

        publish = mqtt_client.publish(command_topic, "ON")
        publish.wait_for_publish()

        io_mock.wait_for_mock_state("Hello World!\n")
