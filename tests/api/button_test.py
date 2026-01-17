from mock_file import IOMock
from utils import (
    OS_PLATFORM,
    Platform,
    UbiHome,
    fnv1_hash_object_id,
)
import aioesphomeapi


async def test_run(io_mock: IOMock):
    button_id = "my_button"
    button_name = "Write Hello World"

    DEVICE_INFO_CONFIG = f"""
ubihome:
  name: test_device

api:

shell:
  type: {"shell" if OS_PLATFORM is Platform.LINUX else "powershell"}
  
button: 
 - platform: shell
   id: {button_id}
   name: {button_name}
   command: "echo 'Hello World!' > {io_mock.file}"
"""

    async with UbiHome("run", config=DEVICE_INFO_CONFIG, wait_for_api=True) as ubihome:
        api = aioesphomeapi.APIClient("127.0.0.1", ubihome.port, "")
        await api.connect(login=False)

        entities, services = await api.list_entities_services()
        assert len(entities) == 1, entities
        print("buttons", entities)
        entity = entities[0]

        assert type(entity) == aioesphomeapi.ButtonInfo
        assert entity.object_id == button_id
        assert entity.name == button_name

        api.button_command(fnv1_hash_object_id(button_id))
        assert io_mock.wait_for_mock_state("Hello World!\n")
