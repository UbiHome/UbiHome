---
title: 'Sendspin - Music Streaming'
description: 'Stream music via Sendspin, e.g. Music Assistant'
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

Each `media_player` entry is its own Sendspin player, with its own connection to the server. Settings that only make sense per-player (client name/id, output device, volume, mute) live on the `media_player` entry; settings shared by every player (server address, audio format, buffer size) stay in the top-level `sendspin:` section.

```yaml
sendspin:
  # Optional: Address of the Sendspin server (default: automatically discovered via mDNS)
  # server: ws://
  # Optional: Audio format used by every player (defaults shown)
  # bit_depth: 16
  # sample_rate: 48000
  # Optional: ALSA buffer size in frames, shared by every player (default: system default)
  # buffer_size:
  # Optional: Milliseconds of audio to pre-buffer before starting playback (default: 500)
  # start_buffer_ms: 500

media_player:
  - platform: sendspin
    name: 'Living Room Speaker'
    # Optional: Unique ID of this client in Sendspin (default: this entity's id)
    # id: living_room_speaker
    # Optional: ID of the output device (defaults to first device found)
    # output_id:
    # Optional: Default playback volume 0-100, applied on init (default: 100)
    # volume: 100
    # Optional: Start muted (default: false)
    # muted: false
    # Optional: Apply server volume commands to the software player (default: true)
    # Disable if on_volume_change already drives a hardware volume, to avoid
    # applying the volume change twice.
    # software_volume: true
    # on_play / on_pause / on_volume_change / on_mute_change: see Media Player
```

On Linux you may need to specify the output device name manually, as UbiHome may detect the default device incorrectly.
To find the device name enable debug logging for UbiHome and look for the line `Devices:` in the logs.

## Features

- Multiple `media_player` entries, each with its own connection to the server and its own output device
- Play/pause/stop, with triggers for custom automations
- Volume control, with a trigger on volume change

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

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/features/entities/media_player/">Media Player</a>
</div>
