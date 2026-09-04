use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use boringtun::x25519::{PublicKey, StaticSecret};
use log::{debug, info};
use serde::{Deserialize, Deserializer};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::mpsc;
use tokio::sync::Notify;
use tokio::time;

use ubihome_core::internal::sensors::{UbiBinarySensor, UbiComponent, UbiSensor};
use ubihome_core::{config_template, ChangedMessage, Module, NoConfig, PublishedMessage};

mod config;
mod device;
mod iface;
mod wg;

use config::{decode_key, WireGuardConfig};
use device::VirtualIpDevice;
use iface::TcpVirtualInterface;
use wg::WireGuardTunnel;

config_template!(
    wireguard,
    WireGuardConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig
);

/// Object ids of the status entities exposed by this platform.
const STATUS_CONNECTED_ID: &str = "wireguard_status";
const STATUS_HANDSHAKE_AGE_ID: &str = "wireguard_handshake_age";

/// Default status refresh interval (ESPHome's `update_interval` default).
const DEFAULT_STATUS_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Clone, Debug)]
pub struct UbiHomePlatform {
    config: CoreConfig,
    components: Vec<UbiComponent>,
}

impl Module for UbiHomePlatform {
    fn new(config_string: &str, config_path: &str) -> Result<Self, String> {
        let config =
            ubihome_core::validation::validate_config::<CoreConfig>(config_string, config_path)?;

        // Validate keys and address eagerly so misconfiguration fails fast.
        decode_key(&config.wireguard.private_key, "private_key")?;
        decode_key(&config.wireguard.peer_public_key, "peer_public_key")?;
        if let Some(psk) = &config.wireguard.peer_preshared_key {
            decode_key(psk, "peer_preshared_key")?;
        }
        let source_peer_ip = config.wireguard.tunnel_ip()?;
        config.wireguard.tunnel_prefix()?;
        for rule in config.wireguard.inbound_rules() {
            if rule.listen.ip() != source_peer_ip {
                return Err(format!(
                    "inbound forward listen address {} must use the tunnel address {}",
                    rule.listen.ip(),
                    source_peer_ip
                ));
            }
        }

        let components = vec![
            UbiComponent::BinarySensor(UbiBinarySensor {
                platform: "sensor".to_string(),
                icon: Some("mdi:vpn".to_string()),
                device_class: Some("connectivity".to_string()),
                name: "WireGuard Status".to_string(),
                id: STATUS_CONNECTED_ID.to_string(),
                on_press: None,
                on_release: None,
                filters: None,
            }),
            // TODO: mark this sensor as diagnostic once core `UbiSensor` gains an
            // `entity_category` field (see the `entity-category-diagnostic` branch);
            // then set `entity_category: Some("diagnostic".to_string())` here.
            UbiComponent::Sensor(UbiSensor {
                platform: "sensor".to_string(),
                icon: Some("mdi:timer-outline".to_string()),
                name: "WireGuard Handshake Age".to_string(),
                id: STATUS_HANDSHAKE_AGE_ID.to_string(),
                state_class: Some("measurement".to_string()),
                device_class: Some("duration".to_string()),
                unit_of_measurement: Some("s".to_string()),
                accuracy_decimals: Some(0),
                filters: None,
            }),
        ];

        Ok(UbiHomePlatform { config, components })
    }

    fn components(&mut self) -> Vec<UbiComponent> {
        self.components.clone()
    }

    fn run(
        &self,
        sender: Sender<ChangedMessage>,
        _receiver: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let config = self.config.wireguard.clone();

        Box::pin(async move {
            run_tunnel(config, sender, None)
                .await
                .map_err(|e| -> Box<dyn std::error::Error> { e.into() })
        })
    }
}

async fn run_tunnel(
    config: WireGuardConfig,
    sender: Sender<ChangedMessage>,
    bind_override: Option<SocketAddr>,
) -> Result<(), String> {
    let private = StaticSecret::from(decode_key(&config.private_key, "private_key")?);
    let public = PublicKey::from(decode_key(&config.peer_public_key, "peer_public_key")?);
    let preshared = config
        .peer_preshared_key
        .as_deref()
        .map(|psk| decode_key(psk, "peer_preshared_key"))
        .transpose()?;

    let endpoint = config.resolve_endpoint()?;
    let bind: SocketAddr = bind_override.unwrap_or_else(|| {
        if endpoint.is_ipv4() {
            "0.0.0.0:0".parse().unwrap()
        } else {
            "[::]:0".parse().unwrap()
        }
    });
    let source_peer_ip = config.tunnel_ip()?;
    let tunnel_prefix = config.tunnel_prefix()?;
    let keepalive = config.keepalive_seconds();
    let mtu = 1420;

    let inbound_queue = Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new()));
    let poll_notify = Arc::new(Notify::new());
    let (outbound_tx, outbound_rx) = mpsc::unbounded_channel();

    let device = VirtualIpDevice::new(inbound_queue.clone(), outbound_tx, mtu);

    let wg = WireGuardTunnel::new(
        private,
        public,
        preshared,
        keepalive,
        0,
        endpoint,
        bind,
        inbound_queue,
        poll_notify.clone(),
    )
    .await?;

    {
        let wg = wg.clone();
        tokio::spawn(async move { wg.produce_task().await });
    }
    {
        let wg = wg.clone();
        tokio::spawn(async move { wg.consume_task(outbound_rx).await });
    }
    {
        let wg = wg.clone();
        tokio::spawn(async move { wg.routine_task().await });
    }

    wg.initiate_handshake().await;
    info!("WireGuard tunnel started to {endpoint} (peer ip {source_peer_ip}/{tunnel_prefix})");

    let status_interval = config.update_interval.unwrap_or(DEFAULT_STATUS_INTERVAL);
    spawn_status_task(wg.clone(), sender.clone(), keepalive, status_interval);

    let interface = TcpVirtualInterface::new(
        &config.forwards,
        source_peer_ip,
        tunnel_prefix,
        &config.peer_allowed_ips,
        poll_notify,
    )?;
    interface.spawn_outbound_listeners().await?;

    interface.poll_loop(device).await
}

fn spawn_status_task(
    wg: Arc<WireGuardTunnel>,
    sender: Sender<ChangedMessage>,
    keepalive: Option<u16>,
    interval_duration: Duration,
) {
    // Consider the tunnel connected while the last handshake is fresh. Rekeys
    // happen roughly every two minutes, so pad the threshold generously.
    let stale_after = Duration::from_secs((keepalive.unwrap_or(0) as u64 * 3).max(180));
    tokio::spawn(async move {
        let mut interval = time::interval(interval_duration);
        loop {
            interval.tick().await;
            let age = wg.handshake_age();
            let connected = age.map(|a| a < stale_after).unwrap_or(false);
            let age_secs = age.map(|a| a.as_secs() as f32).unwrap_or(f32::NAN);

            let _ = sender.send(ChangedMessage::BinarySensorValueChange {
                key: STATUS_CONNECTED_ID.to_string(),
                value: connected,
            });
            if age_secs.is_finite() {
                let _ = sender.send(ChangedMessage::SensorValueChange {
                    key: STATUS_HANDSHAKE_AGE_ID.to_string(),
                    value: age_secs,
                });
            }
        }
    });
}

#[cfg(test)]
mod e2e_tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD, Engine};
    use boringtun::x25519::{PublicKey as XPublic, StaticSecret as XSecret};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream, UdpSocket};

    async fn free_tcp_port() -> u16 {
        TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    async fn free_udp_port() -> u16 {
        UdpSocket::bind("127.0.0.1:0")
            .await
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    fn keys(seed: u8) -> (String, String) {
        let secret = XSecret::from([seed; 32]);
        let public = XPublic::from(&secret);
        (
            STANDARD.encode(secret.to_bytes()),
            STANDARD.encode(public.as_bytes()),
        )
    }

    /// End-to-end: a client dials the "home" peer's outbound listener, which
    /// tunnels through real WireGuard to the "device" peer's inbound listener,
    /// which bridges to a local echo server. Proves both the inbound and outbound
    /// data paths across a genuine handshake.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn forwards_tcp_through_the_tunnel() {
        let (device_priv, device_pub) = keys(1);
        let (home_priv, home_pub) = keys(2);

        let device_udp = free_udp_port().await;
        let home_udp = free_udp_port().await;
        let echo_port = free_tcp_port().await;
        let home_listen = free_tcp_port().await;

        // Local echo server behind the device's inbound forward.
        let echo = TcpListener::bind(("127.0.0.1", echo_port)).await.unwrap();
        tokio::spawn(async move {
            while let Ok((mut socket, _)) = echo.accept().await {
                tokio::spawn(async move {
                    let mut buf = [0u8; 1024];
                    loop {
                        match socket.read(&mut buf).await {
                            Ok(0) | Err(_) => break,
                            Ok(n) => {
                                if socket.write_all(&buf[..n]).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                });
            }
        });

        // Device peer (10.9.0.2/32): inbound forward to the echo server. Uses a /32
        // address + `0.0.0.0/0` allowed-ips (like a typical client config), so the
        // reply to the home peer is routed via the installed default route.
        let device_cfg = WireGuardConfig {
            address: "10.9.0.2".to_string(),
            netmask: Some("255.255.255.255".to_string()),
            private_key: device_priv,
            peer_endpoint: "127.0.0.1".to_string(),
            peer_port: Some(home_udp),
            peer_public_key: home_pub,
            peer_preshared_key: None,
            peer_persistent_keepalive: Some(Duration::from_secs(5)),
            peer_allowed_ips: vec!["0.0.0.0/0".to_string()],
            update_interval: None,
            forwards: vec![config::ForwardRule {
                direction: config::Direction::Inbound,
                listen: "10.9.0.2:9000".parse().unwrap(),
                remote: format!("127.0.0.1:{echo_port}").parse().unwrap(),
            }],
        };

        // Home peer (10.9.0.1/32): outbound forward to the device's inbound listener.
        // Reaches the device (10.9.0.2) via its allowed-ips host route.
        let home_cfg = WireGuardConfig {
            address: "10.9.0.1".to_string(),
            netmask: Some("255.255.255.255".to_string()),
            private_key: home_priv,
            peer_endpoint: "127.0.0.1".to_string(),
            peer_port: Some(device_udp),
            peer_public_key: device_pub,
            peer_preshared_key: None,
            peer_persistent_keepalive: Some(Duration::from_secs(5)),
            peer_allowed_ips: vec!["10.9.0.2/32".to_string()],
            update_interval: None,
            forwards: vec![config::ForwardRule {
                direction: config::Direction::Outbound,
                listen: format!("127.0.0.1:{home_listen}").parse().unwrap(),
                remote: "10.9.0.2:9000".parse().unwrap(),
            }],
        };

        let (tx, _rx) = tokio::sync::broadcast::channel(64);
        let tx2 = tx.clone();

        let device_bind: SocketAddr = format!("127.0.0.1:{device_udp}").parse().unwrap();
        let home_bind: SocketAddr = format!("127.0.0.1:{home_udp}").parse().unwrap();
        tokio::spawn(async move { run_tunnel(device_cfg, tx, Some(device_bind)).await });
        tokio::spawn(async move { run_tunnel(home_cfg, tx2, Some(home_bind)).await });

        // Give the handshake + listeners time to come up.
        let mut response = Vec::new();
        let mut ok = false;
        for _ in 0..40 {
            time::sleep(Duration::from_millis(250)).await;
            if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", home_listen)).await {
                if stream.write_all(b"ping").await.is_ok() {
                    let mut buf = [0u8; 4];
                    if let Ok(Ok(4)) =
                        time::timeout(Duration::from_secs(2), stream.read_exact(&mut buf)).await
                    {
                        response = buf.to_vec();
                        ok = true;
                        break;
                    }
                }
            }
        }
        assert!(ok, "no response received through the tunnel");
        assert_eq!(&response, b"ping");
    }
}
