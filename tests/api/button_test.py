
from asyncio import sleep
import os
from tests.utils import UbiHome
import aioesphomeapi


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

  async with UbiHome("run", DEVICE_INFO_CONFIG) as ubihome:
    api = aioesphomeapi.APIClient("127.0.0.1", 6053, "MyPassword")
    await api.connect(login=True)

    entities, services = await api.list_entities_services()
    assert len(entities) == 1, entities
    print("buttons", entities)
    entity = entities[0]

    assert entity.unique_id == button_id
    assert entity.name == button_name

    api.button_command(0)
    await sleep(1)
    assert open("test.log", "r").read() == "Hello World!\n"
    os.remove("test.log")