use core::panic;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::Device;
use log::{debug, error, info, trace, warn};
use sendspin::audio::decode::{Decoder, PcmDecoder, PcmEndian};
use sendspin::audio::{AudioBuffer, AudioFormat, Codec, SyncedPlayer, SyncedPlayerConfig};
use sendspin::protocol::messages::{
    AudioFormatSpec, ClientState, Message, PlayerCommandType, PlayerState, PlayerV1Support,
};
use sendspin::sync::ClockSync;
use sendspin::ProtocolClientBuilder;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{future::Future, pin::Pin, str};
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::Sender;
use ubihome_core::internal::sensors::UbiComponent;
use ubihome_core::NoConfig;
use ubihome_core::{config_template, ChangedMessage, Module, PublishedMessage};

/// Type alias for the clock sync reference
type ClockSyncRef = Arc<parking_lot::Mutex<ClockSync>>;

/// Commands sent to the audio player thread
enum PlayerCommand {
    /// Enqueue an audio buffer for playback
    Enqueue(AudioBuffer),
    /// Clear the player buffer
    Clear,
    /// Initialize the player with the given format and clock sync
    Init(AudioFormat, ClockSyncRef),
    /// Set the player volume (0-100)
    SetVolume(u8),
    /// Mute or unmute the player
    SetMute(bool),
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_bool(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

/// Enumerate the available audio hosts and pick the output device to use.
///
/// Enumeration runs fresh on every call, so devices that connect after
/// startup (e.g. a Bluetooth speaker) are picked up. When `output_id` is set,
/// the device with a matching id is selected; otherwise the first host's
/// default output device is used.
fn resolve_output_device(output_id: Option<&str>) -> Option<Device> {
    let mut selected_device: Option<Device> = None;
    let available_hosts = cpal::available_hosts();

    for host_id in available_hosts {
        let host = cpal::host_from_id(host_id).unwrap();
        debug!("Host: {}", host_id.name());

        if selected_device.is_none() {
            selected_device = host.default_output_device();
        }

        let devices = host.devices().unwrap();
        debug!("  Devices: ");
        for device in devices {
            let id = device
                .id() // id()
                .map_or("Unknown Id".to_string(), |id| id.to_string());
            let description = device.description().unwrap(); // description
            debug!("  {id} - {description}");

            // Output configs
            if let Ok(conf) = device.default_output_config() {
                trace!("    Default output stream config:");
                trace!("      {conf:?}");
            }

            if let Ok(configs) = device.supported_output_configs() {
                trace!("    Supported output stream config:\n");
                for conf in configs {
                    trace!("      {conf:?}");
                }
            }

            if let Some(output_id) = output_id {
                if id == output_id {
                    selected_device = Some(device)
                }
            }
        }
    }

    selected_device
}

#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct SendspinConfig {
    pub name: Option<String>,
    pub server: Option<String>,
    pub id: Option<String>,
    pub output_id: Option<String>,
    pub bit_depth: Option<u8>,
    pub sample_rate: Option<u32>,
    /// Default playback volume (0-100) applied when the player is initialized.
    /// The server can still change it at runtime via volume commands. Default: 100.
    #[garde(range(min = 0, max = 100))]
    pub volume: Option<u8>,
    /// Whether the player starts muted. The server can still change it at
    /// runtime via mute commands. Default: false.
    pub muted: Option<bool>,
    /// ALSA buffer size in frames. Increase this on slow hardware (e.g. Raspberry Pi Zero W)
    /// to prevent buffer underruns ("Broken pipe" errors). At 48 kHz stereo,
    /// 4096 frames ≈ 85 ms. Default: system default (typically 512-1024 frames).
    pub buffer_size: Option<u32>,
}

config_template!(
    sendspin,
    SendspinConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig
);

#[derive(Clone, Debug)]
pub struct UbiHomePlatform {
    config: CoreConfig,
    pub sendspin_config: SendspinConfig,
}

impl Module for UbiHomePlatform {
    fn new(config_string: &str, config_path: &str) -> Result<Self, String> {
        let config =
            ubihome_core::validation::validate_config::<CoreConfig>(config_string, config_path)?;

        let config_clone = config.clone();
        Ok(UbiHomePlatform {
            config,
            sendspin_config: config_clone.sendspin,
        })
    }

    fn components(&mut self) -> Vec<UbiComponent> {
        Vec::new()
    }

    fn run(
        &self,
        _sender: Sender<ChangedMessage>,
        mut _receiver: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let name = self
            .sendspin_config
            .name
            .clone()
            .unwrap_or(self.config.ubihome.name.clone());
        let id = self.sendspin_config.id.clone().unwrap_or(name.clone());
        let bit_depth = self.sendspin_config.bit_depth.unwrap_or(16);
        let sample_rate = self.sendspin_config.sample_rate.unwrap_or(48000);
        let buffer_size = self.sendspin_config.buffer_size;
        let volume = self.sendspin_config.volume.unwrap_or(100);
        let muted = self.sendspin_config.muted.unwrap_or(false);
        // Device selection is deferred until playback starts (see PlayerCommand::Init)
        // so devices connected after startup (e.g. Bluetooth) can be used.
        let output_id = self.sendspin_config.output_id.clone();

        // Enumerate devices once at startup so users can discover output ids from
        // the `Devices:` log lines. Only when debug logging is enabled, since the
        // authoritative selection happens per playback below.
        if log::log_enabled!(log::Level::Debug) {
            resolve_output_device(output_id.as_deref());
        }

        let server = if let Some(s) = self.sendspin_config.server.clone() {
            info!("Using configured Sendspin server at {}", s);
            s
        } else {
            use mdns_sd::{ServiceDaemon, ServiceEvent};
            // Create a daemon
            let mdns = ServiceDaemon::new().expect("Failed to create daemon");
            let mut mdns_found_server = None;
            let service_type = "_sendspin-server._tcp.local.";
            // Browse for sendspin servers
            let receiver = mdns.browse(service_type).expect("Failed to browse");
            while let Ok(event) = receiver.recv() {
                if let ServiceEvent::ServiceResolved(resolved) = event {
                    info!("Resolved a new service: {:?}", resolved);
                    let path = resolved
                        .txt_properties
                        .get_property_val("path")
                        .and_then(|opt_v| opt_v.and_then(|v| str::from_utf8(v).ok()))
                        .unwrap_or("");
                    let address = resolved.get_addresses().iter().next();

                    if let Some(addr) = address {
                        let server = format!("ws://{}:{}{}", addr, resolved.get_port(), path);
                        info!("Using Sendspin server at {}", server);
                        mdns_found_server = Some(server);
                        break;
                    } else {
                        info!("No address found for resolved service.");
                        continue;
                    }
                }
            }
            if let Some(mdns_servier) = mdns_found_server {
                mdns_servier
            } else {
                panic!("No Sendspin server found via mDNS");
            }
        };

        Box::pin(async move {
            // Configuration from environment variables
            let start_buffer_ms = env_u64("SS_PLAY_START_BUFFER_MS", 500);
            let log_lead = env_bool("SS_LOG_LEAD");
            info!(
                "Player config: start_buffer={}ms, log_lead={}",
                start_buffer_ms, log_lead
            );

            // Create a channel for sending commands to the audio player thread.
            // The player thread outlives individual server connections, so it is
            // created once, before the reconnect loop.
            let (player_tx, player_rx) = std::sync::mpsc::channel::<PlayerCommand>();

            // Spawn a dedicated thread for audio playback (SyncedPlayer is not Send)
            std::thread::spawn(move || {
                let mut synced_player: Option<SyncedPlayer> = None;

                while let Ok(cmd) = player_rx.recv() {
                    match cmd {
                        PlayerCommand::Init(fmt, clock_sync) => {
                            // Re-resolve the output device for each playback so that
                            // devices connected after startup (e.g. Bluetooth) are used.
                            let device = resolve_output_device(output_id.as_deref());
                            match &device {
                                Some(d) => info!(
                                    "Using device: {}",
                                    d.id().map_or("Unknown Id".to_string(), |id| id.to_string())
                                ),
                                None => info!(
                                    "No output device resolved; falling back to system default"
                                ),
                            }
                            match SyncedPlayer::new(
                                fmt,
                                clock_sync,
                                SyncedPlayerConfig {
                                    device,
                                    volume,
                                    muted,
                                    buffer_size,
                                },
                            ) {
                                Ok(player) => {
                                    debug!("Synced audio output initialized");
                                    synced_player = Some(player);
                                }
                                Err(e) => {
                                    error!("Failed to create synced output: {}", e);
                                }
                            }
                        }
                        PlayerCommand::Enqueue(buffer) => {
                            if let Some(ref player) = synced_player {
                                player.enqueue(buffer);
                            } else {
                                log::warn!("Player not initialized yet, dropping audio buffer");
                            }
                        }
                        PlayerCommand::Clear => {
                            if let Some(ref player) = synced_player {
                                player.clear();
                            } else {
                                log::warn!("Player not initialized yet, cannot clear");
                            }
                        }
                        PlayerCommand::SetVolume(vol) => {
                            if let Some(ref mut player) = synced_player {
                                player.set_volume(vol);
                            } else {
                                log::warn!("Player not initialized yet, cannot set volume");
                            }
                        }
                        PlayerCommand::SetMute(muted) => {
                            if let Some(ref mut player) = synced_player {
                                player.set_mute(muted);
                            } else {
                                log::warn!("Player not initialized yet, cannot set mute");
                            }
                        }
                    }
                }
                debug!("Audio player thread exited");
            });

            // Reconnect loop: a Sendspin connection can drop silently (server
            // restart, network blip, idle timeout). Reconnect with exponential
            // backoff instead of exiting the module, which would drop this player
            // until UbiHome is restarted.
            let initial_backoff = std::time::Duration::from_secs(1);
            let max_backoff = std::time::Duration::from_secs(30);
            // A connection must stay up at least this long to be treated as stable.
            // Connections that drop sooner keep escalating the backoff, so a server
            // that accepts the connection and then drops the session immediately is
            // not reconnected roughly once per second.
            let stable_connection = std::time::Duration::from_secs(10);
            let mut backoff = initial_backoff;
            // Whether a player has ever been initialized. Used to avoid clearing a
            // player that does not exist yet (e.g. on the very first connection).
            let mut player_ever_initialized = false;

            loop {
                info!("Connecting to Sendspin server at {}...", server);
                let test = ProtocolClientBuilder::builder()
                    .client_id(id.clone())
                    .name(name.clone())
                    .player_v1_support(PlayerV1Support {
                        supported_formats: vec![AudioFormatSpec {
                            codec: "pcm".to_string(),
                            channels: 2,
                            sample_rate,
                            bit_depth,
                        }],
                        buffer_capacity: 50 * 1024 * 1024, // 50 MB
                        supported_commands: vec!["volume".to_string(), "mute".to_string()],
                    })
                    .initial_player_state(PlayerState {
                        volume: Some(volume),
                        muted: Some(muted),
                        static_delay_ms: Some(0),
                        supported_commands: None,
                        min_buffer_ms: None,
                        required_lead_time_ms: None,
                    })
                    .build();

                let client = match test.connect(&server).await {
                    Ok(client) => client,
                    Err(e) => {
                        error!(
                            "Failed to connect to Sendspin server at {}: {}. Retrying in {:?}",
                            server, e, backoff
                        );
                        tokio::time::sleep(backoff).await;
                        backoff = (backoff * 2).min(max_backoff);
                        continue;
                    }
                };
                info!("Connected to Sendspin server at {}", server);
                let connected_at = std::time::Instant::now();

                let conn = client.split();
                let mut message_rx = conn.messages;
                let mut audio_rx = conn.audio;
                let clock_sync = conn.clock_sync;
                let sender = conn.sender;
                let _guard = conn.guard;

                info!("Waiting for stream to start...");

                // Clear any audio left buffered from a previous connection. Skip it
                // until a player exists, otherwise this warns on every startup.
                if player_ever_initialized {
                    let _ = player_tx.send(PlayerCommand::Clear);
                }

                // Message handling variables (reset for each connection)
                let mut decoder: Option<PcmDecoder> = None;
                let mut audio_format: Option<AudioFormat> = None;
                let mut endian_locked: Option<PcmEndian> = None; // Auto-detect on first chunk
                let mut buffered_duration_us: u64 = 0; // Track buffered audio duration in microseconds
                let mut playback_started = false; // Track if we've started playback
                let mut first_chunk_logged = false; // Track if we've logged the first chunk
                let mut player_initialized = false; // Track if player has been initialized

                loop {
                    // Process messages and audio chunks concurrently
                    tokio::select! {
                        Some(msg) = message_rx.recv() => {
                            match msg {
                                Message::StreamStart(stream_start) => {
                                    if let Some(ref player_config) = stream_start.player {
                                        debug!(
                                            "Stream starting: codec='{}' {}Hz {}ch {}bit",
                                            player_config.codec,
                                            player_config.sample_rate,
                                            player_config.channels,
                                            player_config.bit_depth
                                        );

                                        // Validate codec before proceeding
                                        if player_config.codec != "pcm" {
                                            error!("ERROR: Unsupported codec '{}' - only 'pcm' is supported!", player_config.codec);
                                            error!("Server is sending compressed audio that we can't decode!");
                                            continue;
                                        }

                                        if player_config.bit_depth != 16 && player_config.bit_depth != 24 {
                                            error!("ERROR: Unsupported bit depth {} - only 16 or 24-bit PCM supported!", player_config.bit_depth);
                                            continue;
                                        }

                                        audio_format = Some(AudioFormat {
                                            codec: Codec::Pcm,
                                            sample_rate: player_config.sample_rate,
                                            channels: player_config.channels,
                                            bit_depth: player_config.bit_depth,
                                            codec_header: None,
                                        });

                                        // Decoder will be created on first chunk after auto-detecting endianness
                                        decoder = None;
                                        endian_locked = None;
                                        buffered_duration_us = 0; // Reset on new stream
                                        playback_started = false;
                                        first_chunk_logged = false; // Reset for new stream
                                        debug!("Waiting for first audio chunk to auto-detect endianness...");
                                    } else {
                                        debug!("Received stream/start without player config");
                                    }
                                }
                                Message::ServerTime(server_time) => {
                                    // Get t4 (client receive time) in Unix microseconds
                                    let t4 = SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap()
                                        .as_micros() as i64;

                                    // Update clock sync with all four timestamps
                                    let t1 = server_time.client_transmitted;
                                    let t2 = server_time.server_received;
                                    let t3 = server_time.server_transmitted;

                                    clock_sync.lock().update(t1, t2, t3, t4);

                                    // Log sync quality
                                    let sync = clock_sync.lock();
                                    if let Some(rtt) = sync.rtt_micros() {
                                        let quality = sync.quality();
                                        debug!(
                                            "Clock sync updated: RTT={:.2}ms, quality={:?}",
                                            rtt as f64 / 1000.0,
                                            quality
                                        );
                                    }
                                }
                                Message::StreamEnd(stream_end) => {
                                    debug!("Stream ended: {:?}", stream_end.roles);
                                    let _ = player_tx.send(PlayerCommand::Clear);
                                    buffered_duration_us = 0;
                                    playback_started = false;
                                    first_chunk_logged = false;
                                    player_initialized = false;
                                }
                                Message::StreamClear(stream_clear) => {
                                    debug!("Stream cleared: {:?}", stream_clear.roles);
                                    let _ = player_tx.send(PlayerCommand::Clear);
                                    buffered_duration_us = 0;
                                    playback_started = false;
                                    first_chunk_logged = false;
                                    player_initialized = false;
                                }
                                Message::ServerCommand(cmd) => {
                                    // debug!("Received server command: {}", );
                                    // Set Volume
                                    if !player_initialized {
                                        log::warn!("Received server command before player initialized: {:?}", cmd);
                                        continue;
                                    }
                                    match cmd.player {
                                        None => {
                                            log::warn!("Received server command without player field: {:?}", cmd);
                                            continue;
                                        }
                                        Some(player) => {
                                            match player.command {
                                                PlayerCommandType::Volume => {
                                                    match player.volume {
                                                        None => {
                                                            log::warn!("Received server command without volume field");
                                                        }
                                                        Some(vol) => {
                                                            info!("Setting player volume to {}", vol);
                                                            let _ = player_tx.send(PlayerCommand::SetVolume(vol));
                                                        }
                                                    }
                                                }
                                                PlayerCommandType::Mute => {
                                                    match player.mute {
                                                        None => {
                                                            log::warn!("Received server command without muted field");
                                                        }
                                                        Some(mute) => {
                                                            log::info!("Received mute command: {}", mute);
                                                            let _ = player_tx.send(PlayerCommand::SetMute(mute));
                                                            if let Err(e) = sender.send_message(Message::ClientState(
                                                                ClientState {
                                                                    state: None,
                                                                    player: Some(PlayerState {
                                                                        volume: None,
                                                                        muted: Some(mute),
                                                                        static_delay_ms: None,
                                                                        supported_commands: None,
                                                                        required_lead_time_ms: None,
                                                                        min_buffer_ms: None
                                                                    }),
                                                                }
                                                            )).await {
                                                                warn!("Failed to send mute state to server: {}", e);
                                                            }
                                                        }
                                                    }
                                                }
                                                _ => {
                                                    debug!("Received unsupported server command: {:?}", player.command);
                                                    continue;
                                                }
                                            }

                                        }
                                    }
                                }
                                _ => {
                                    debug!("Received message: {:?}", msg);
                                }
                            }
                        }
                        Some(chunk) = audio_rx.recv() => {
                            // Log first chunk bytes for diagnostics
                            if !first_chunk_logged {
                                let preview_len = chunk.data.len().min(32);
                                let hex_string = chunk.data[..preview_len]
                                    .iter()
                                    .map(|b| format!("{:02X}", b))
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                debug!("First {} bytes (hex): {}", preview_len, hex_string);
                                first_chunk_logged = true;
                            }

                            if let Some(ref fmt) = audio_format {
                                // Frame sanity check
                                let bytes_per_sample = match fmt.bit_depth {
                                    16 => 2,
                                    24 => 3,
                                    _ => {
                                        error!("Unsupported bit depth: {}", fmt.bit_depth);
                                        continue;
                                    }
                                } as usize;
                                let frame_size = bytes_per_sample * fmt.channels as usize;

                                if chunk.data.len() % frame_size != 0 {
                                    error!(
                                        "BAD FRAME: {} bytes not multiple of frame size {} ({}-bit, {}ch)",
                                        chunk.data.len(), frame_size, fmt.bit_depth, fmt.channels
                                    );
                                    continue; // Don't decode garbage
                                }

                                // One-time endianness setup on first chunk
                                // Per spec: macOS and most systems use Little-Endian PCM
                                // Only use Big-Endian if explicitly signaled by server
                                if endian_locked.is_none() {
                                    // Default to Little-Endian (standard for macOS/Windows/Linux)
                                    let endian = PcmEndian::Little;
                                    endian_locked = Some(endian);
                                    decoder = Some(PcmDecoder::with_endian(fmt.bit_depth, endian));
                                    debug!("Using Little-Endian PCM (standard for modern systems)");
                                }
                            }

                            if let (Some(ref dec), Some(ref fmt)) = (&decoder, &audio_format) {
                                match dec.decode(&chunk.data) {
                                    Ok(samples) => {
                                        // Calculate chunk duration in microseconds
                                        // samples.len() includes all channels
                                        let frames = samples.len() / fmt.channels as usize;
                                        let duration_micros = (frames as u64 * 1_000_000) / fmt.sample_rate as u64;
                                        // Track buffered duration
                                        buffered_duration_us += duration_micros;

                                        // Check if we've buffered enough to start playback
                                        if !playback_started && buffered_duration_us >= start_buffer_ms * 1000 {
                                            playback_started = true;
                                            debug!(
                                                "Prebuffering complete ({:.1}ms buffered), starting playback!",
                                                buffered_duration_us as f64 / 1000.0
                                            );
                                        }

                                        // Track and log lead time
                                        if log_lead {
                                            trace!(
                                                "Enqueued chunk ts={} buffered={:.1}ms len={} bytes",
                                                chunk.timestamp,
                                                buffered_duration_us as f64 / 1000.0,
                                                chunk.data.len()
                                            );
                                        }

                                        if !player_initialized {
                                            let _ = player_tx.send(PlayerCommand::Init(
                                                fmt.clone(),
                                                Arc::clone(&clock_sync),
                                            ));
                                            player_initialized = true;
                                            player_ever_initialized = true;
                                        }

                                        let buffer = AudioBuffer {
                                            timestamp: chunk.timestamp,
                                            samples,
                                            format: fmt.clone(),
                                        };
                                        let _ = player_tx.send(PlayerCommand::Enqueue(buffer));
                                    }
                                    Err(e) => {
                                        error!("Decode error: {}", e);
                                    }
                                }
                            }
                        }
                        else => {
                            // Both channels closed: the connection dropped, so
                            // leave the inner loop to reconnect.
                            debug!("Message and audio channels closed; reconnecting");
                            break;
                        }
                    }
                }

                // Reset the backoff only if the connection stayed up long enough to
                // be considered stable, so a brief blip reconnects quickly while a
                // connection that drops immediately keeps backing off.
                if connected_at.elapsed() >= stable_connection {
                    backoff = initial_backoff;
                }
                warn!(
                    "Sendspin connection to {} closed; reconnecting in {:?}",
                    server, backoff
                );
                if player_ever_initialized {
                    let _ = player_tx.send(PlayerCommand::Clear);
                }
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(max_backoff);
            }
        })
    }
}
