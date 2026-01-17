from asyncio import sleep
from unittest.mock import Mock
from mock_file import IOMock
from utils import OS_PLATFORM, Platform, UbiHome, fnv1_hash_object_id
import aioesphomeapi


async def test_run(io_mock: IOMock):
    switch_id = "my_switch"
    switch_key = fnv1_hash_object_id(switch_id)
    switch_name = "Switch it"

    io_mock.set_value("false")

    DEVICE_INFO_CONFIG = f"""
ubihome:
  name: test_device

api:

shell:
  type: {"shell" if OS_PLATFORM is Platform.LINUX else "powershell"}
  
switch: 
 - platform: shell
   id: {switch_id}
   name: {switch_name}
   command_on: "echo true > {io_mock.file}"
   command_off: "echo false > {io_mock.file}"
   command_state: "cat {io_mock.file}"
   update_interval: 1s
"""

    async with UbiHome("run", config=DEVICE_INFO_CONFIG, wait_for_api=True) as ubihome:
        api = aioesphomeapi.APIClient("127.0.0.1", ubihome.port, "")
        await api.connect(login=False)

        entities, services = await api.list_entities_services()
        print("switches", entities, services)
        assert len(entities) == 1, entities
        entity = entities[0]

        assert type(entity) == aioesphomeapi.SwitchInfo
        assert entity.key == switch_key
        assert entity.object_id == switch_id
        assert entity.name == switch_name

        mock = Mock()
        # Subscribe to the state changes
        api.subscribe_states(mock)

        # Test switching the switch on via command
        api.switch_command(switch_key, True)
        io_mock.wait_for_mock_state("true")

        # State update should be send back
        state_switched_to_true = False
        while state_switched_to_true:
            state_switched_to_true = (
                mock.called and mock.call_args.args[0].state is True
            )
            await sleep(0.1)

        mock.reset_mock()

        # Test switching the switch off via command
        api.switch_command(switch_key, False)
        io_mock.wait_for_mock_state("false")

        # State update should be send back
        state_switched_to_false = False
        while state_switched_to_false:
            state_switched_to_false = (
                mock.called and mock.call_args.args[0].state is False
            )
            await sleep(0.1)
        mock.reset_mock()

        # Test switching the switch on via local change
        io_mock.set_value("true")

        # Wait for the state change
        while not mock.called:
            await sleep(0.1)
        state = mock.call_args.args[0]
        assert state.state is True
