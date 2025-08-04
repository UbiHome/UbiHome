
from asyncio import sleep
import os
from utils import UbiHome
import aioesphomeapi
from utils import wait_and_get_file


async def test_run():
  button_id = "my_button"
  button_name = "Write Hello World"
  DEVICE_INFO_CONFIG = f"""
ubihome:
  name: test_device

api:

shell:
  
button: 
 - platform: shell
   id: {button_id}
   name: {button_name}
   command: "echo 'Hello World!' > test.log"
"""

  async with UbiHome("run", config=DEVICE_INFO_CONFIG, wait_for_api=True) as ubihome:
    api = aioesphomeapi.APIClient("127.0.0.1", 6053, "")
    await api.connect(login=False)

    entities, services = await api.list_entities_services()
    assert len(entities) == 1, entities
    print("buttons", entities)
    entity = entities[0]

    assert type(entity) == aioesphomeapi.ButtonInfo
    assert entity.unique_id == button_id
    assert entity.name == button_name

    api.button_command(0)
    assert wait_and_get_file("test.log") == "Hello World!\n"
    os.remove("test.log")