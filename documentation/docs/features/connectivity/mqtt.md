# MQTT

The MQTT client connects to your MQTT broker and allows you to receive sensor updates or send actions to your device.

This is a simple configuration: 

```yaml

mqtt:
  broker: <your_broker>
  username: <your_username>
  password: <your_password>
```

Here is a more complex example:

```yaml
mqtt:
  broker: <your_broker>
  username: <your_username>
  password: <your_password>
  discovery: true
  discovery_prefix: homeassistant
  topic_prefix: oshome
```

This will connect to your MQTT broker and send all sensor updates to the `oshome/sensor/<sensor_name>` topic. It will also listen for actions on the `oshome/action/<action_name>` topic.

## Device Discovery

Device Discovery is enabled by default, so the device will automatically be discovered by Home Assistant. You can find the device in the MQTT integration in Home Assistant.

<!-- For reference: https://esphome.io/components/mqtt.html -->