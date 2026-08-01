use std::collections::{HashMap, VecDeque};
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration as StdDuration;

use log::{debug, error, info, warn};
use smoltcp::iface::{Config as IfaceConfig, Interface, SocketHandle, SocketSet};
use smoltcp::socket::tcp;
use smoltcp::time::Instant;
use smoltcp::wire::{HardwareAddress, IpAddress, IpCidr, IpListenEndpoint};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::sync::Notify;

use crate::config::{Direction, ForwardRule};
use crate::device::VirtualIpDevice;

const SOCKET_BUFFER: usize = 65536;
const READ_CHUNK: usize = 8192;

static CONN_ID: AtomicU64 = AtomicU64::new(1);

/// Application-side data destined for a virtual socket, or a request to close it.
enum AppMsg {
    Data(Vec<u8>),
    Close,
}

/// Control-plane command from an outbound TCP listener to the poll loop: open a
/// virtual client socket connecting out through the tunnel to `remote`.
struct OpenOutbound {
    conn_id: u64,
    remote: SocketAddr,
    net_to_app: UnboundedSender<Vec<u8>>,
}

struct Conn {
    handle: SocketHandle,
    net_to_app: UnboundedSender<Vec<u8>>,
    send_queue: VecDeque<Vec<u8>>,
    closing: bool,
}

/// A single smoltcp interface that serves every forward rule: outbound client
/// sockets (local listener → tunnel) and inbound listening sockets (tunnel →
/// local service) share one IP stack and one virtual device.
pub struct TcpVirtualInterface {
    source_peer_ip: IpAddr,
    tunnel_prefix: u8,
    allowed_cidrs: Vec<IpCidr>,
    default_v4: bool,
    default_v6: bool,
    outbound_rules: Vec<ForwardRule>,
    inbound_rules: Vec<ForwardRule>,
    poll_notify: Arc<Notify>,
    commands_rx: UnboundedReceiver<OpenOutbound>,
    commands_tx: UnboundedSender<OpenOutbound>,
    app_to_net_rx: UnboundedReceiver<(u64, AppMsg)>,
    app_to_net_tx: UnboundedSender<(u64, AppMsg)>,
    next_port: u16,
}

impl TcpVirtualInterface {
    pub fn new(
        rules: &[ForwardRule],
        source_peer_ip: IpAddr,
        tunnel_prefix: u8,
        allowed_ips: &[String],
        poll_notify: Arc<Notify>,
    ) -> Result<Self, String> {
        let (commands_tx, commands_rx) = mpsc::unbounded_channel();
        let (app_to_net_tx, app_to_net_rx) = mpsc::unbounded_channel();

        let mut allowed_cidrs = Vec::new();
        let mut default_v4 = false;
        let mut default_v6 = false;
        for entry in allowed_ips {
            match parse_allowed_ip(entry)? {
                AllowedIp::DefaultV4 => default_v4 = true,
                AllowedIp::DefaultV6 => default_v6 = true,
                AllowedIp::Cidr(cidr) => allowed_cidrs.push(cidr),
            }
        }

        Ok(Self {
            source_peer_ip,
            tunnel_prefix,
            allowed_cidrs,
            default_v4,
            default_v6,
            outbound_rules: rules
                .iter()
                .filter(|r| r.direction == Direction::Outbound)
                .cloned()
                .collect(),
            inbound_rules: rules
                .iter()
                .filter(|r| r.direction == Direction::Inbound)
                .cloned()
                .collect(),
            poll_notify,
            commands_rx,
            commands_tx,
            app_to_net_rx,
            app_to_net_tx,
            next_port: 49152,
        })
    }

    fn alloc_port(&mut self) -> u16 {
        let port = self.next_port;
        self.next_port = if self.next_port >= 60999 {
            49152
        } else {
            self.next_port + 1
        };
        port
    }

    fn interface_addresses(&self) -> Vec<IpCidr> {
        // The tunnel address with its subnet prefix, so replies to other peers in
        // the subnet (e.g. Home Assistant, for inbound) are treated as on-link.
        let mut cidrs = vec![IpCidr::new(
            IpAddress::from(self.source_peer_ip),
            self.tunnel_prefix,
        )];
        let mut add = |ip: IpAddress, prefix: u8| {
            if !cidrs.iter().any(|c| c.address() == ip) {
                cidrs.push(IpCidr::new(ip, prefix));
            }
        };
        // Networks reachable through the tunnel (peer_allowed_ips) are on-link.
        for cidr in &self.allowed_cidrs {
            add(cidr.address(), cidr.prefix_len());
        }
        // Each distinct outbound destination is reachable through the tunnel.
        for rule in &self.outbound_rules {
            let prefix = if rule.remote.is_ipv4() { 32 } else { 128 };
            add(IpAddress::from(rule.remote.ip()), prefix);
        }
        cidrs
    }

    fn new_tcp_socket() -> tcp::Socket<'static> {
        tcp::Socket::new(
            tcp::SocketBuffer::new(vec![0u8; SOCKET_BUFFER]),
            tcp::SocketBuffer::new(vec![0u8; SOCKET_BUFFER]),
        )
    }

    /// Starts the outbound TCP listeners. Each accepts local connections and asks
    /// the poll loop (over the command channel) to open a matching virtual socket.
    pub async fn spawn_outbound_listeners(&self) -> Result<(), String> {
        for rule in &self.outbound_rules {
            let listener = TcpListener::bind(rule.listen)
                .await
                .map_err(|e| format!("failed to bind outbound listener on {}: {e}", rule.listen))?;
            info!(
                "Tunnel outbound forward listening on {} -> {}",
                rule.listen, rule.remote
            );
            let remote = rule.remote;
            let commands = self.commands_tx.clone();
            let app_to_net = self.app_to_net_tx.clone();
            tokio::spawn(async move {
                loop {
                    match listener.accept().await {
                        Ok((stream, peer)) => {
                            let conn_id = CONN_ID.fetch_add(1, Ordering::Relaxed);
                            let (net_to_app_tx, net_to_app_rx) = mpsc::unbounded_channel();
                            debug!("[{conn_id}] outbound connection from {peer} -> {remote}");
                            if commands
                                .send(OpenOutbound {
                                    conn_id,
                                    remote,
                                    net_to_app: net_to_app_tx,
                                })
                                .is_err()
                            {
                                return;
                            }
                            let app_to_net = app_to_net.clone();
                            tokio::spawn(bridge(conn_id, stream, app_to_net, net_to_app_rx));
                        }
                        Err(e) => {
                            error!("outbound listener accept error: {e}");
                            return;
                        }
                    }
                }
            });
        }
        Ok(())
    }

    /// Runs the smoltcp poll loop until an unrecoverable error occurs.
    pub async fn poll_loop(mut self, mut device: VirtualIpDevice) -> Result<(), String> {
        let iface_config = IfaceConfig::new(HardwareAddress::Ip);
        let mut iface = Interface::new(iface_config, &mut device, Instant::now());
        let addresses = self.interface_addresses();
        iface.update_ip_addrs(|addrs| {
            for cidr in addresses {
                let _ = addrs.push(cidr);
            }
        });
        // A `0.0.0.0/0` / `::/0` in peer_allowed_ips means "route everything through
        // the tunnel": install a default route via this peer's own tunnel address.
        if self.default_v4 {
            if let IpAddress::Ipv4(gw) = IpAddress::from(self.source_peer_ip) {
                let _ = iface.routes_mut().add_default_ipv4_route(gw);
            }
        }
        if self.default_v6 {
            if let IpAddress::Ipv6(gw) = IpAddress::from(self.source_peer_ip) {
                let _ = iface.routes_mut().add_default_ipv6_route(gw);
            }
        }

        let mut sockets = SocketSet::new(Vec::new());
        let mut conns: HashMap<u64, Conn> = HashMap::new();
        // Listening sockets for inbound rules: handle -> rule index.
        let mut listeners: HashMap<SocketHandle, usize> = HashMap::new();

        for (idx, rule) in self.inbound_rules.iter().enumerate() {
            let handle = add_listen_socket(&mut sockets, self.source_peer_ip, rule.listen.port())?;
            listeners.insert(handle, idx);
            info!(
                "Tunnel inbound forward listening on {} -> {}",
                rule.listen, rule.remote
            );
        }

        loop {
            let timestamp = Instant::now();
            iface.poll(timestamp, &mut device, &mut sockets);

            // Promote any listening socket that received a connection into a bridged
            // connection, and re-arm a fresh listener in its place.
            let promoted: Vec<(SocketHandle, usize)> = listeners
                .iter()
                .filter(|(handle, _)| {
                    let socket = sockets.get::<tcp::Socket>(**handle);
                    !matches!(socket.state(), tcp::State::Listen | tcp::State::Closed)
                })
                .map(|(h, i)| (*h, *i))
                .collect();
            for (handle, idx) in promoted {
                listeners.remove(&handle);
                let rule = self.inbound_rules[idx].clone();
                let conn_id = CONN_ID.fetch_add(1, Ordering::Relaxed);
                let (net_to_app_tx, net_to_app_rx) = mpsc::unbounded_channel();
                conns.insert(
                    conn_id,
                    Conn {
                        handle,
                        net_to_app: net_to_app_tx,
                        send_queue: VecDeque::new(),
                        closing: false,
                    },
                );
                debug!(
                    "[{conn_id}] inbound connection accepted on {} -> dialing {}",
                    rule.listen, rule.remote
                );
                let app_to_net = self.app_to_net_tx.clone();
                tokio::spawn(dial_and_bridge(
                    conn_id,
                    rule.remote,
                    app_to_net,
                    net_to_app_rx,
                ));

                match add_listen_socket(&mut sockets, self.source_peer_ip, rule.listen.port()) {
                    Ok(new_handle) => {
                        listeners.insert(new_handle, idx);
                    }
                    Err(e) => warn!("failed to re-arm inbound listener: {e}"),
                }
            }

            // Pump each active connection between its smoltcp socket and its bridge.
            let mut finished: Vec<u64> = Vec::new();
            for (conn_id, conn) in conns.iter_mut() {
                let socket = sockets.get_mut::<tcp::Socket>(conn.handle);

                if socket.can_recv() {
                    let received = socket.recv(|buffer| (buffer.len(), buffer.to_vec())).ok();
                    if let Some(data) = received {
                        if !data.is_empty() && conn.net_to_app.send(data).is_err() {
                            // Bridge gone; tear down the socket.
                            socket.close();
                        }
                    }
                }

                while socket.can_send() {
                    let Some(front) = conn.send_queue.front() else {
                        break;
                    };
                    match socket.send_slice(front) {
                        Ok(0) => break,
                        Ok(sent) if sent < front.len() => {
                            let rest = front[sent..].to_vec();
                            conn.send_queue.pop_front();
                            conn.send_queue.push_front(rest);
                        }
                        Ok(_) => {
                            conn.send_queue.pop_front();
                        }
                        Err(e) => {
                            debug!("[{conn_id}] send error: {e:?}");
                            break;
                        }
                    }
                }

                if conn.closing && conn.send_queue.is_empty() && socket.may_send() {
                    socket.close();
                }

                if socket.state() == tcp::State::Closed {
                    finished.push(*conn_id);
                }
            }
            for conn_id in finished {
                if let Some(conn) = conns.remove(&conn_id) {
                    sockets.remove(conn.handle);
                    debug!("[{conn_id}] connection closed");
                }
            }

            let delay = compute_delay(&mut iface, &sockets, timestamp);

            tokio::select! {
                _ = sleep_opt(delay) => {}
                _ = self.poll_notify.notified() => {}
                Some(cmd) = self.commands_rx.recv() => {
                    self.handle_open_outbound(cmd, &mut iface, &mut sockets, &mut conns);
                }
                Some((conn_id, msg)) = self.app_to_net_rx.recv() => {
                    if let Some(conn) = conns.get_mut(&conn_id) {
                        match msg {
                            AppMsg::Data(data) => conn.send_queue.push_back(data),
                            AppMsg::Close => conn.closing = true,
                        }
                    }
                }
            }
        }
    }

    fn handle_open_outbound(
        &mut self,
        cmd: OpenOutbound,
        iface: &mut Interface,
        sockets: &mut SocketSet<'static>,
        conns: &mut HashMap<u64, Conn>,
    ) {
        let local_port = self.alloc_port();
        let handle = sockets.add(Self::new_tcp_socket());
        let socket = sockets.get_mut::<tcp::Socket>(handle);
        let remote = (IpAddress::from(cmd.remote.ip()), cmd.remote.port());
        let local = (IpAddress::from(self.source_peer_ip), local_port);
        match socket.connect(iface.context(), remote, local) {
            Ok(()) => {
                conns.insert(
                    cmd.conn_id,
                    Conn {
                        handle,
                        net_to_app: cmd.net_to_app,
                        send_queue: VecDeque::new(),
                        closing: false,
                    },
                );
            }
            Err(e) => {
                error!("[{}] failed to open virtual socket: {e:?}", cmd.conn_id);
                sockets.remove(handle);
            }
        }
    }
}

enum AllowedIp {
    DefaultV4,
    DefaultV6,
    Cidr(IpCidr),
}

/// Parses a `peer_allowed_ips` entry in CIDR notation. A `/0` prefix is treated
/// as a default route rather than an interface address.
fn parse_allowed_ip(entry: &str) -> Result<AllowedIp, String> {
    let entry = entry.trim();
    let (addr_str, prefix_str) = entry
        .split_once('/')
        .ok_or_else(|| format!("peer_allowed_ips entry '{entry}' must be CIDR (ip/prefix)"))?;
    let ip: IpAddr = addr_str
        .trim()
        .parse()
        .map_err(|e| format!("invalid peer_allowed_ips entry '{entry}': {e}"))?;
    let prefix: u8 = prefix_str
        .trim()
        .parse()
        .map_err(|e| format!("invalid prefix in peer_allowed_ips entry '{entry}': {e}"))?;
    if prefix == 0 {
        return Ok(if ip.is_ipv4() {
            AllowedIp::DefaultV4
        } else {
            AllowedIp::DefaultV6
        });
    }
    Ok(AllowedIp::Cidr(IpCidr::new(IpAddress::from(ip), prefix)))
}

fn add_listen_socket(
    sockets: &mut SocketSet<'static>,
    ip: IpAddr,
    port: u16,
) -> Result<SocketHandle, String> {
    let mut socket = TcpVirtualInterface::new_tcp_socket();
    let endpoint = IpListenEndpoint {
        addr: Some(IpAddress::from(ip)),
        port,
    };
    socket
        .listen(endpoint)
        .map_err(|e| format!("failed to listen on {ip}:{port}: {e:?}"))?;
    Ok(sockets.add(socket))
}

fn compute_delay(
    iface: &mut Interface,
    sockets: &SocketSet,
    timestamp: Instant,
) -> Option<StdDuration> {
    // Newly queued app data wakes the loop via the app_to_net select branch, and
    // inbound ACKs wake it via poll_notify, so honoring smoltcp's own poll delay
    // (capped) is enough and avoids busy-spinning under TCP backpressure.
    match iface.poll_delay(timestamp, sockets) {
        Some(d) => {
            Some(StdDuration::from_micros(d.total_micros()).min(StdDuration::from_millis(1000)))
        }
        None => Some(StdDuration::from_millis(1000)),
    }
}

async fn sleep_opt(delay: Option<StdDuration>) {
    match delay {
        Some(d) => tokio::time::sleep(d).await,
        None => std::future::pending::<()>().await,
    }
}

/// For inbound: dials the local service, then bridges it to the virtual socket.
async fn dial_and_bridge(
    conn_id: u64,
    remote: SocketAddr,
    app_to_net: UnboundedSender<(u64, AppMsg)>,
    net_to_app_rx: UnboundedReceiver<Vec<u8>>,
) {
    match TcpStream::connect(remote).await {
        Ok(stream) => bridge(conn_id, stream, app_to_net, net_to_app_rx).await,
        Err(e) => {
            error!("[{conn_id}] failed to dial local service {remote}: {e}");
            let _ = app_to_net.send((conn_id, AppMsg::Close));
        }
    }
}

/// Copies bytes between a real tokio TCP stream and its virtual socket via channels.
async fn bridge(
    conn_id: u64,
    mut stream: TcpStream,
    app_to_net: UnboundedSender<(u64, AppMsg)>,
    mut net_to_app_rx: UnboundedReceiver<Vec<u8>>,
) {
    let mut buf = vec![0u8; READ_CHUNK];
    loop {
        tokio::select! {
            read = stream.read(&mut buf) => {
                match read {
                    Ok(0) => break,
                    Ok(n) => {
                        if app_to_net.send((conn_id, AppMsg::Data(buf[..n].to_vec()))).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        debug!("[{conn_id}] local read error: {e}");
                        break;
                    }
                }
            }
            msg = net_to_app_rx.recv() => {
                match msg {
                    Some(data) => {
                        if stream.write_all(&data).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    }
    let _ = app_to_net.send((conn_id, AppMsg::Close));
    let _ = stream.shutdown().await;
}
