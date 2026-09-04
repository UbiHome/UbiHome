use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use boringtun::noise::errors::WireGuardError;
use boringtun::noise::{Tunn, TunnResult};
use boringtun::x25519::{PublicKey, StaticSecret};
use log::{debug, error, warn};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Notify;
use tokio::time;

use crate::device::InboundQueue;

const MAX_PACKET: usize = 65536;

/// A userspace WireGuard tunnel: owns the boringtun state machine and the UDP
/// socket to the endpoint. The `Tunn` state is guarded by a std `Mutex` held only
/// across synchronous crypto — never across an `.await`.
pub struct WireGuardTunnel {
    peer: Mutex<Tunn>,
    udp: UdpSocket,
    inbound: InboundQueue,
    poll_notify: Arc<Notify>,
}

impl WireGuardTunnel {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        private: StaticSecret,
        public: PublicKey,
        preshared: Option<[u8; 32]>,
        keepalive: Option<u16>,
        index: u32,
        endpoint: SocketAddr,
        bind: SocketAddr,
        inbound: InboundQueue,
        poll_notify: Arc<Notify>,
    ) -> Result<Arc<Self>, String> {
        let peer = Tunn::new(private, public, preshared, keepalive, index, None);
        let udp = UdpSocket::bind(bind)
            .await
            .map_err(|e| format!("failed to bind WireGuard UDP socket: {e}"))?;
        udp.connect(endpoint)
            .await
            .map_err(|e| format!("failed to connect WireGuard UDP socket to {endpoint}: {e}"))?;
        Ok(Arc::new(Self {
            peer: Mutex::new(peer),
            udp,
            inbound,
            poll_notify,
        }))
    }

    /// Time since the last successful handshake, or `None` if never completed.
    pub fn handshake_age(&self) -> Option<Duration> {
        self.peer.lock().expect("tunn poisoned").stats().0
    }

    /// Proactively sends a handshake initiation so the tunnel comes up before any
    /// application traffic flows.
    pub async fn initiate_handshake(&self) {
        let packet = {
            let mut peer = self.peer.lock().expect("tunn poisoned");
            let mut buf = vec![0u8; MAX_PACKET];
            match peer.format_handshake_initiation(&mut buf, false) {
                TunnResult::WriteToNetwork(p) => Some(p.to_vec()),
                _ => None,
            }
        };
        if let Some(p) = packet {
            let _ = self.udp.send(&p).await;
        }
    }

    /// Receives encrypted datagrams from the endpoint, decapsulates them, and
    /// either replies on the wire (handshake) or enqueues the plaintext IP packet
    /// for the smoltcp stack.
    pub async fn produce_task(self: Arc<Self>) {
        let mut recv_buf = vec![0u8; MAX_PACKET];
        loop {
            let n = match self.udp.recv(&mut recv_buf).await {
                Ok(n) => n,
                Err(e) => {
                    error!("WireGuard UDP receive error: {e}");
                    time::sleep(Duration::from_millis(500)).await;
                    continue;
                }
            };

            let (to_network, to_stack) = {
                let mut peer = self.peer.lock().expect("tunn poisoned");
                let mut to_network: Vec<Vec<u8>> = Vec::new();
                let mut to_stack: Option<Vec<u8>> = None;
                let mut buf = vec![0u8; MAX_PACKET];
                match peer.decapsulate(None, &recv_buf[..n], &mut buf) {
                    TunnResult::Done => {}
                    TunnResult::Err(e) => debug!("WireGuard decapsulate error: {e:?}"),
                    TunnResult::WriteToNetwork(packet) => {
                        to_network.push(packet.to_vec());
                        // Flush any further queued packets (repeated-call pattern).
                        loop {
                            let mut flush = vec![0u8; MAX_PACKET];
                            match peer.decapsulate(None, &[], &mut flush) {
                                TunnResult::WriteToNetwork(packet) => {
                                    to_network.push(packet.to_vec())
                                }
                                _ => break,
                            }
                        }
                    }
                    TunnResult::WriteToTunnelV4(packet, _)
                    | TunnResult::WriteToTunnelV6(packet, _) => {
                        to_stack = Some(packet.to_vec());
                    }
                }
                (to_network, to_stack)
            };

            for packet in to_network {
                let _ = self.udp.send(&packet).await;
            }
            if let Some(packet) = to_stack {
                self.inbound
                    .lock()
                    .expect("inbound queue poisoned")
                    .push_back(packet);
                self.poll_notify.notify_one();
            }
        }
    }

    /// Encapsulates outbound IP packets produced by the smoltcp stack and sends
    /// them to the endpoint.
    pub async fn consume_task(self: Arc<Self>, mut outbound: UnboundedReceiver<Vec<u8>>) {
        while let Some(packet) = outbound.recv().await {
            let encapsulated = {
                let mut peer = self.peer.lock().expect("tunn poisoned");
                let mut buf = vec![0u8; MAX_PACKET];
                match peer.encapsulate(&packet, &mut buf) {
                    TunnResult::WriteToNetwork(p) => Some(p.to_vec()),
                    TunnResult::Err(e) => {
                        debug!("WireGuard encapsulate error: {e:?}");
                        None
                    }
                    _ => None,
                }
            };
            if let Some(p) = encapsulated {
                let _ = self.udp.send(&p).await;
            }
        }
    }

    /// Drives handshakes, keep-alives and timers.
    pub async fn routine_task(self: Arc<Self>) {
        loop {
            let outcome = {
                let mut peer = self.peer.lock().expect("tunn poisoned");
                let mut buf = vec![0u8; MAX_PACKET];
                match peer.update_timers(&mut buf) {
                    TunnResult::WriteToNetwork(p) => RoutineOutcome::Send(p.to_vec()),
                    TunnResult::Err(WireGuardError::ConnectionExpired) => {
                        warn!("WireGuard handshake expired; re-initiating");
                        let mut hs = vec![0u8; MAX_PACKET];
                        match peer.format_handshake_initiation(&mut hs, false) {
                            TunnResult::WriteToNetwork(p) => RoutineOutcome::Send(p.to_vec()),
                            _ => RoutineOutcome::Idle,
                        }
                    }
                    TunnResult::Err(e) => {
                        debug!("WireGuard routine error: {e:?}");
                        RoutineOutcome::Idle
                    }
                    _ => RoutineOutcome::Idle,
                }
            };

            match outcome {
                RoutineOutcome::Send(packet) => {
                    let _ = self.udp.send(&packet).await;
                }
                RoutineOutcome::Idle => {}
            }
            time::sleep(Duration::from_millis(250)).await;
        }
    }
}

enum RoutineOutcome {
    Send(Vec<u8>),
    Idle,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    fn keypair(seed: u8) -> (StaticSecret, PublicKey) {
        let secret = StaticSecret::from([seed; 32]);
        let public = PublicKey::from(&secret);
        (secret, public)
    }

    async fn free_port() -> u16 {
        tokio::net::UdpSocket::bind("127.0.0.1:0")
            .await
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    /// Drives two real userspace WireGuard peers against each other over UDP
    /// loopback and asserts the handshake completes on both sides.
    #[tokio::test]
    async fn completes_handshake_between_two_peers() {
        let (client_secret, client_public) = keypair(1);
        let (server_secret, server_public) = keypair(2);

        let a_addr: SocketAddr = format!("127.0.0.1:{}", free_port().await).parse().unwrap();
        let b_addr: SocketAddr = format!("127.0.0.1:{}", free_port().await).parse().unwrap();

        let make = |secret, peer_public, endpoint, bind, index| {
            let inbound = Arc::new(Mutex::new(VecDeque::new()));
            let notify = Arc::new(Notify::new());
            WireGuardTunnel::new(
                secret,
                peer_public,
                None,
                Some(25),
                index,
                endpoint,
                bind,
                inbound,
                notify,
            )
        };

        let a = make(client_secret, server_public, b_addr, a_addr, 1)
            .await
            .unwrap();
        let b = make(server_secret, client_public, a_addr, b_addr, 2)
            .await
            .unwrap();

        tokio::spawn(a.clone().produce_task());
        tokio::spawn(a.clone().routine_task());
        tokio::spawn(b.clone().produce_task());
        tokio::spawn(b.clone().routine_task());

        a.initiate_handshake().await;

        let mut completed = false;
        for _ in 0..50 {
            time::sleep(Duration::from_millis(100)).await;
            if a.handshake_age().is_some() && b.handshake_age().is_some() {
                completed = true;
                break;
            }
        }
        assert!(completed, "WireGuard handshake did not complete within 5s");
    }
}
