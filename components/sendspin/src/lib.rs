use core::panic;
use log::info;
use sendspin::audio::decode::{Decoder, PcmDecoder, PcmEndian};
use sendspin::audio::{AudioBuffer, AudioFormat, Codec, SyncedPlayer};
use sendspin::protocol::client::ProtocolClient;
use sendspin::protocol::messages::{
    AudioFormatSpec, ClientHello, ClientState, ClientTime, DeviceInfo, Message, PlayerState,
    PlayerSyncState, PlayerV1Support,
};
use sendspin::sync::ClockSync;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{future::Future, pin::Pin, str};
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::Sender;
use tokio::time::interval;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use ubihome_core::internal::sensors::InternalComponent;
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
    /// Shutdown the player thread
    Shutdown,
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

#[derive(Clone, Deserialize, Debug)]
pub struct SendspinConfig {
    pub name: Option<String>,
    pub server: Option<String>,
    pub id: Option<String>,
}

config_template!(
    sendspin,
    SendspinConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig
);

#[derive(Clone, Debug)]
pub struct UbiHomeDefault {
    config: CoreConfig,
    pub sendspin_config: SendspinConfig,
}

impl Module for UbiHomeDefault {
    fn new(config_string: &String) -> Result<Self, String> {
        match serde_yaml::from_str::<CoreConfig>(config_string) {
            Ok(config) => {
                let config_clone = config.clone();
                Ok(UbiHomeDefault {
                    config: config,
                    sendspin_config: config_clone.sendspin,
                })
            }
            Err(e) => {
                return Err(format!("Failed to parse API config: {:?}", e));
            }
        }
    }

    fn components(&mut self) -> Vec<InternalComponent> {
        Vec::new()
    }

    fn run(
        &self,
        sender: Sender<ChangedMessage>,
        mut receiver: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let name = self
            .sendspin_config
            .name
            .clone()
            .unwrap_or(self.config.ubihome.name.clone());
        let id = self.sendspin_config.id.clone().unwrap_or(name.clone());

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
                match event {
                    ServiceEvent::ServiceResolved(resolved) => {
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
                    _ => {}
                }
            }
            if let Some(mdns_servier) = mdns_found_server {
                mdns_servier
            } else {
                panic!("No Sendspin server found via mDNS");
            }
        };

        Box::pin(async move {
            let hello = ClientHello {
                client_id: id.clone(),
                name: name.clone(),
                version: 1,
                supported_roles: vec!["player@v1".to_string()],
                device_info: Some(DeviceInfo {
                    product_name: Some(name.clone()),
                    manufacturer: Some("Sendspin".to_string()),
                    software_version: Some("0.1.0".to_string()),
                }),
                player_v1_support: Some(PlayerV1Support {
                    supported_formats: vec![AudioFormatSpec {
                        codec: "pcm".to_string(),
                        channels: 2,
                        sample_rate: 48000,
                        bit_depth: 16,
                    }],
                    buffer_capacity: 50 * 1024 * 1024, // 50 MB
                    supported_commands: vec!["volume".to_string(), "mute".to_string()],
                }),
                artwork_v1_support: None,
                visualizer_v1_support: None,
            };

            info!("Connecting to {}...", server);
            let mut request = server.into_client_request().unwrap();
            let client = ProtocolClient::connect(request, hello).await.unwrap();
            info!("Connected!");

            // Split client into separate receivers for concurrent processing
            let (mut message_rx, mut audio_rx, clock_sync, ws_tx) = client.split();

            //Send initial client/state message (handshake step 3)
            let client_state = Message::ClientState(ClientState {
                player: Some(PlayerState {
                    state: PlayerSyncState::Synchronized,
                    volume: Some(100),
                    muted: Some(false),
                }),
            });
            ws_tx.send_message(client_state).await.unwrap();
            info!("Sent initial client/state");

            // Send immediate initial clock sync
            let client_transmitted = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as i64;
            let time_msg = Message::ClientTime(ClientTime { client_transmitted });
            ws_tx.send_message(time_msg).await.unwrap();
            info!("Sent initial client/time for clock sync");

            info!("Waiting for stream to start...");

            // Spawn clock sync task that sends client/time every 5 seconds
            tokio::spawn(async move {
                let mut interval = interval(Duration::from_secs(5));
                loop {
                    interval.tick().await;

                    // Get current Unix epoch microseconds
                    let client_transmitted = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_micros() as i64;

                    let time_msg = Message::ClientTime(ClientTime { client_transmitted });

                    // Send time sync message
                    if let Err(e) = ws_tx.send_message(time_msg).await {
                        eprintln!("Failed to send time sync: {}", e);
                        break;
                    }
                }
            });

            // // Configuration from environment variables
            let start_buffer_ms = env_u64("SS_PLAY_START_BUFFER_MS", 500);
            let log_lead = env_bool("SS_LOG_LEAD");
            info!(
                "Player config: start_buffer={}ms, log_lead={}",
                start_buffer_ms, log_lead
            );

            // Create a channel for sending commands to the audio player thread
            let (player_tx, player_rx) = std::sync::mpsc::channel::<PlayerCommand>();

            // Spawn a dedicated thread for audio playback (SyncedPlayer is not Send)
            std::thread::spawn(move || {
                let mut synced_player: Option<SyncedPlayer> = None;

                while let Ok(cmd) = player_rx.recv() {
                    match cmd {
                        PlayerCommand::Init(fmt, clock_sync) => {
                            match SyncedPlayer::new(fmt, clock_sync, None) {
                                Ok(player) => {
                                    info!("Synced audio output initialized");
                                    synced_player = Some(player);
                                }
                                Err(e) => {
                                    eprintln!("Failed to create synced output: {}", e);
                                }
                            }
                        }
                        PlayerCommand::Enqueue(buffer) => {
                            if let Some(ref player) = synced_player {
                                player.enqueue(buffer);
                            }
                        }
                        PlayerCommand::Clear => {
                            if let Some(ref player) = synced_player {
                                player.clear();
                            }
                        }
                        PlayerCommand::Shutdown => {
                            break;
                        }
                    }
                }
            });

            // Message handling variables
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
                                    println!(
                                        "Stream starting: codec='{}' {}Hz {}ch {}bit",
                                        player_config.codec,
                                        player_config.sample_rate,
                                        player_config.channels,
                                        player_config.bit_depth
                                    );

                                    // Validate codec before proceeding
                                    if player_config.codec != "pcm" {
                                        eprintln!("ERROR: Unsupported codec '{}' - only 'pcm' is supported!", player_config.codec);
                                        eprintln!("Server is sending compressed audio that we can't decode!");
                                        continue;
                                    }

                                    if player_config.bit_depth != 16 && player_config.bit_depth != 24 {
                                        eprintln!("ERROR: Unsupported bit depth {} - only 16 or 24-bit PCM supported!", player_config.bit_depth);
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
                                    println!("Waiting for first audio chunk to auto-detect endianness...");
                                } else {
                                    println!("Received stream/start without player config");
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
                                    println!(
                                        "Clock sync updated: RTT={:.2}ms, quality={:?}",
                                        rtt as f64 / 1000.0,
                                        quality
                                    );
                                }
                            }
                            Message::StreamEnd(stream_end) => {
                                println!("Stream ended: {:?}", stream_end.roles);
                                let _ = player_tx.send(PlayerCommand::Clear);
                                buffered_duration_us = 0;
                                playback_started = false;
                                first_chunk_logged = false;
                                player_initialized = false;
                            }
                            Message::StreamClear(stream_clear) => {
                                println!("Stream cleared: {:?}", stream_clear.roles);
                                let _ = player_tx.send(PlayerCommand::Clear);
                                buffered_duration_us = 0;
                                playback_started = false;
                                first_chunk_logged = false;
                                player_initialized = false;
                            }
                            _ => {
                                println!("Received message: {:?}", msg);
                            }
                        }
                    }
                    Some(chunk) = audio_rx.recv() => {
                        // Log first chunk bytes for diagnostics
                        if !first_chunk_logged {
                            println!("\n=== FIRST AUDIO CHUNK DIAGNOSTICS ===");
                            println!("Chunk timestamp: {} µs", chunk.timestamp);
                            println!("Chunk data length: {} bytes", chunk.data.len());
                            let preview_len = chunk.data.len().min(32);
                            print!("First {} bytes (hex): ", preview_len);
                            for byte in &chunk.data[..preview_len] {
                                print!("{:02X} ", byte);
                            }
                            println!("\n=====================================\n");
                            first_chunk_logged = true;
                        }

                        if let Some(ref fmt) = audio_format {
                            // Frame sanity check
                            let bytes_per_sample = match fmt.bit_depth {
                                16 => 2,
                                24 => 3,
                                _ => {
                                    eprintln!("Unsupported bit depth: {}", fmt.bit_depth);
                                    continue;
                                }
                            } as usize;
                            let frame_size = bytes_per_sample * fmt.channels as usize;

                            if chunk.data.len() % frame_size != 0 {
                                eprintln!(
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
                                println!("Using Little-Endian PCM (standard for modern systems)");
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
                                        println!(
                                            "Prebuffering complete ({:.1}ms buffered), starting playback!",
                                            buffered_duration_us as f64 / 1000.0
                                        );
                                    }

                                    // Track and log lead time
                                    if log_lead {
                                        println!(
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
                                    }

                                    let buffer = AudioBuffer {
                                        timestamp: chunk.timestamp,
                                        play_at: Instant::now(),
                                        samples,
                                        format: fmt.clone(),
                                    };
                                    let _ = player_tx.send(PlayerCommand::Enqueue(buffer));
                                }
                                Err(e) => {
                                    eprintln!("Decode error: {}", e);
                                }
                            }
                        }
                    }
                    else => {
                        // Both channels closed
                        break;
                    }
                }
            }

            // Shutdown the player thread
            let _ = player_tx.send(PlayerCommand::Shutdown);

            Ok(())
        })
    }
}
