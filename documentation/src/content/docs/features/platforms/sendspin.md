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


## Features

- Play/pause/stop
- Volume control

### Supported audio backends

- ALSA (Linux)
- PulseAudio (Linux)


## Setup

### How to find the server address?

By default UbiHome will try to discover the Sendspin server using mDNS. 
If this does not work you can specify the address manually in the configuration (e.g. `ws://192.168.178.123:8927/sendspin`).

### How to find the output device id?

Depending on the platform and audio backend, UbiHome may not be able to automatically detect the correct output device.
In this case you can specify the output device name manually in the configuration. 

#### ALSA (Linux)

To find the device name enable debug logging for Sendspin platform in UbiHome:

```yaml
logger:
  logs:
    ubihome_sendspin: debug
```

and look for the line `Devices:` in the logs. Example: 

```
DEBUG [ubihome_sendspin] Host: ALSA
DEBUG [ubihome_sendspin]   Devices:
DEBUG [ubihome_sendspin]   alsa:null - alsa:null
DEBUG [ubihome_sendspin]   alsa:hw:CARD=Dummy,DEV=0 - alsa:hw:CARD=Dummy,DEV=0
DEBUG [ubihome_sendspin]   alsa:plughw:CARD=Dummy,DEV=0 - alsa:plughw:CARD=Dummy,DEV=0
DEBUG [ubihome_sendspin]   alsa:default:CARD=Dummy - alsa:default:CARD=Dummy
DEBUG [ubihome_sendspin]   alsa:sysdefault:CARD=Dummy - alsa:sysdefault:CARD=Dummy
DEBUG [ubihome_sendspin]   alsa:dmix:CARD=Dummy,DEV=0 - alsa:dmix:CARD=Dummy,DEV=0
DEBUG [ubihome_sendspin]   alsa:dsnoop:CARD=Dummy,DEV=0 - alsa:dsnoop:CARD=Dummy,DEV=0
DEBUG [ubihome_sendspin]   alsa:hw:CARD=sndrpihifiberry,DEV=0 - alsa:hw:CARD=sndrpihifiberry,DEV=0
DEBUG [ubihome_sendspin]   alsa:plughw:CARD=sndrpihifiberry,DEV=0 - alsa:plughw:CARD=sndrpihifiberry,DEV=0
DEBUG [ubihome_sendspin]   alsa:default:CARD=sndrpihifiberry - alsa:default:CARD=sndrpihifiberry
DEBUG [ubihome_sendspin]   alsa:sysdefault:CARD=sndrpihifiberry - alsa:sysdefault:CARD=sndrpihifiberry
DEBUG [ubihome_sendspin]   alsa:dmix:CARD=sndrpihifiberry,DEV=0 - alsa:dmix:CARD=sndrpihifiberry,DEV=0
DEBUG [ubihome_sendspin]   alsa:hw:CARD=vc4hdmi,DEV=0 - alsa:hw:CARD=vc4hdmi,DEV=0
DEBUG [ubihome_sendspin]   alsa:plughw:CARD=vc4hdmi,DEV=0 - alsa:plughw:CARD=vc4hdmi,DEV=0
DEBUG [ubihome_sendspin]   alsa:default:CARD=vc4hdmi - alsa:default:CARD=vc4hdmi
DEBUG [ubihome_sendspin]   alsa:sysdefault:CARD=vc4hdmi - alsa:sysdefault:CARD=vc4hdmi
DEBUG [ubihome_sendspin]   alsa:hdmi:CARD=vc4hdmi,DEV=0 - alsa:hdmi:CARD=vc4hdmi,DEV=0
DEBUG [ubihome_sendspin]   alsa:dmix:CARD=vc4hdmi,DEV=0 - alsa:dmix:CARD=vc4hdmi,DEV=0
DEBUG [ubihome_sendspin]   alsa:hw:CARD=0,DEV=0 - alsa:hw:CARD=0,DEV=0
DEBUG [ubihome_sendspin]   alsa:plughw:CARD=0,DEV=0 - alsa:plughw:CARD=0,DEV=0
DEBUG [ubihome_sendspin]   alsa:hw:CARD=1,DEV=0 - alsa:hw:CARD=1,DEV=0
DEBUG [ubihome_sendspin]   alsa:plughw:CARD=1,DEV=0 - alsa:plughw:CARD=1,DEV=0
DEBUG [ubihome_sendspin]   alsa:hw:CARD=2,DEV=0 - alsa:hw:CARD=2,DEV=0
DEBUG [ubihome_sendspin]   alsa:plughw:CARD=2,DEV=0 - alsa:plughw:CARD=2,DEV=0
```

The output device is resolved each time playback starts (so devices connected after
startup, such as Bluetooth speakers, can be used). When a stream begins, UbiHome logs
the device it selected:

```
INFO [ubihome_sendspin] Using device: alsa:hw:CARD=sndrpihifiberry,DEV=0
```

> You may also use `aplay -l` to list the available ALSA devices.