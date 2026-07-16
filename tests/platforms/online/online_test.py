import asyncio
from asyncio import sleep
from unittest.mock import Mock

import aioesphomeapi
import pytest

from utils import UbiHome


async def _wait_for_state(mock: Mock, expected: bool, timeout: float = 15.0) -> None:
    """Wait until the latest binary sensor state equals ``expected``."""
    elapsed = 0.0
    while elapsed < timeout:
        if mock.called and mock.call_args.args[0].state is expected:
            return
        await sleep(0.2)
        elapsed += 0.2
    raise AssertionError(f"online binary sensor never reached state {expected}")


async def _start_tcp_target():
    """A TCP server; reachable while listening. Returns (port, close, is_open)."""

    async def handle(reader, writer):
        writer.close()

    server = await asyncio.start_server(handle, "127.0.0.1", 0)
    port = server.sockets[0].getsockname()[1]

    async def close():
        server.close()
        await server.wait_closed()

    return port, close, server.is_serving


async def _start_dns_target():
    """A UDP responder; the online DNS check needs a datagram back to be reachable."""

    class _Responder(asyncio.DatagramProtocol):
        def connection_made(self, transport):
            self.transport = transport

        def datagram_received(self, data, addr):
            # Any response makes the online DNS check's recv() succeed.
            self.transport.sendto(b"\x00", addr)

    loop = asyncio.get_running_loop()
    transport, _ = await loop.create_datagram_endpoint(
        _Responder, local_addr=("127.0.0.1", 0)
    )
    port = transport.get_extra_info("sockname")[1]
    state = {"open": True}

    async def close():
        transport.close()
        state["open"] = False

    return port, close, lambda: state["open"]


@pytest.mark.parametrize(
    "protocol,start_target",
    [("tcp", _start_tcp_target), ("dns", _start_dns_target)],
)
async def test_online_connectivity_check(protocol, start_target):
    """
    The online binary sensor reflects real reachability of its target:
    online while a local server responds, offline once it is closed.

    `dns` targets default to port 53; here we override it to reach the local
    responder on an ephemeral port.
    """

    port, close_target, is_open = await start_target()

    CONFIG = f"""
ubihome:
  name: test_device

api:

online:
  update_interval: 1s
  timeout: 1s
  targets:
    - host: 127.0.0.1
      port: {port}
      protocol: {protocol}

binary_sensor:
  - platform: online
    id: internet
    name: Internet
"""

    try:
        async with UbiHome("run", config=CONFIG, wait_for_api=True) as ubihome:
            api = aioesphomeapi.APIClient("127.0.0.1", ubihome.port, "")
            await api.connect(login=False)

            entities, _ = await api.list_entities_services()
            assert len(entities) == 1, entities
            assert isinstance(entities[0], aioesphomeapi.BinarySensorInfo)
            assert entities[0].object_id == "internet"

            mock = Mock()
            api.subscribe_states(mock)

            # Target is responding -> online.
            await _wait_for_state(mock, True)

            # Stop the target -> unreachable -> offline.
            await close_target()

            await _wait_for_state(mock, False)
    finally:
        if is_open():
            await close_target()
