import aioesphomeapi
import asyncio

async def main():
    """Connect to an ESPHome device and get details."""

    # Establish connection
    api = aioesphomeapi.APIClient("localhost", 6053, "MyPassword")
    await api.connect(login=False)

    # api = aioesphomeapi.APIClient("bluetooth-proxy.local", 6053, "test", noise_psk="aMzEtMWQtR3zQ0hq3Ll3dnJ7emjpkC+Gm7oxf4heFiI=")
    # await api.connect(login=False)


    # Get API version of the device's firmware
    print(api.api_version)

    # Show device details
    device_info = await api.device_info()
    print(device_info)

    # List all entities of the device
    # entities = await api.list_entities_services()
    # print(entities)

loop = asyncio.get_event_loop()
loop.run_until_complete(main())