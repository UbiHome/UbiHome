from testcontainers.mqtt import MosquittoContainer
import pytest


@pytest.fixture(scope="session")
async def mqtt_container():
    with MosquittoContainer() as mosquitto_broker:
        yield mosquitto_broker


@pytest.fixture()
def mqtt_client(mqtt_container: MosquittoContainer):
    return mqtt_container.get_client()


@pytest.fixture()
def mqtt_connection(mqtt_container):
    return {
        "host": mqtt_container.get_container_host_ip(),
        "port": int(mqtt_container.get_exposed_port(mqtt_container.MQTT_PORT)),
    }
