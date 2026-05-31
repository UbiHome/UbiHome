---
title: 'Sendspin - Music Streaming'
sidebar:
  badge:
    text: Experimental
    variant: caution
tags: 
  - windows
  - linux
  - macos

---

UbiHome can be used as a client for [Sendspin](https://www.sendspin-audio.com/), e.g. for [Music Assistant](https://www.music-assistant.io/) which natively integrates with Home Assistant.

```yaml
sendspin:
  # Optional: Address of the Sendspin server (default: automatically discovered via mDNS)
  # server: ws://
  # Optional: Name of this client in Sendspin (default: ubihome device name)
  # name:
  # Optional: Unique ID of this client in Sendspin (default: ubihome device name)
  # client_id:
  # Optional: ID of the output device (defaults to first device found)
  # output_id:
```

On Linux you may need to specify the output device name manually, as UbiHome may detect the default device incorrectly.
To find the device name enable debug logging for UbiHome and look for the line `Devices:` in the logs.


## Supported features

- Play/pause/stop
- Volume control

## Supported audio backends

- ALSA (Linux)
- PulseAudio (Linux)
