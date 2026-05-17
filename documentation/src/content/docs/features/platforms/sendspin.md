# Sendspin Music Streaming

UbiHome can be used as a client for [Sendspin](https://www.sendspin-audio.com/), e.g. for [Music Assistant](https://www.music-assistant.io/) which natively integrates with Home Assistant.

```yaml
sendspin:
  # Optional: Address of the Sendspin server
  # server: ws://
  # Optional: Name of this client in Sendspin (default: ubihome device name)
  # name:
  # Optional: Unique ID of this client in Sendspin (default: ubihome device name)
  # client_id:
  # output_id: Optional name of the output device (defaults to first device found)
```

On Linux you may need to specify the output device name manually, as UbiHome may detect the default device incorrectly.
To find the device name enable debug logging for UbiHome and look for the line `Devices:` in the logs.
