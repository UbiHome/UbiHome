use base64::{engine::general_purpose::STANDARD, Engine};
use duration_str::deserialize_option_duration;
use garde::Validate;
use serde::Deserialize;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::time::Duration;

/// Default WireGuard endpoint port when `peer_port` is omitted.
const DEFAULT_PEER_PORT: u16 = 51820;

/// Direction of a forward rule relative to this device.
///
/// * `outbound` — this device dials a service on the home network (e.g. MQTT).
///   A local `TcpListener` is opened on `listen` and traffic is tunneled to `remote`.
/// * `inbound` — a peer on the home network dials this device (e.g. the ESPHome
///   API). A virtual listener is opened on the tunnel IP `listen` and each
///   accepted connection is bridged to the local service at `remote`.
#[derive(Clone, Copy, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Outbound,
    Inbound,
}

#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct ForwardRule {
    pub direction: Direction,
    /// For `outbound`: the local address to bind the TCP listener on.
    /// For `inbound`: the tunnel IP + port to accept connections on (the IP
    /// must equal the tunnel `address`).
    pub listen: SocketAddr,
    /// For `outbound`: the address on the home network to forward to.
    /// For `inbound`: the local service to bridge accepted connections to.
    pub remote: SocketAddr,
}

/// Configuration for the userspace WireGuard interface. Mirrors the ESPHome
/// [`wireguard`](https://esphome.io/components/wireguard/) component's schema,
/// with an additional `forwards` list that selects which connections are
/// tunneled (ESPHome routes the whole interface instead).
#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct WireGuardConfig {
    /// This device's IPv4 address inside the tunnel, e.g. `10.9.0.2`.
    pub address: String,
    /// Network mask for `address`. Defaults to `255.255.255.255`.
    pub netmask: Option<String>,
    /// Base64 private key of this device.
    pub private_key: String,
    /// Hostname or IP of the remote WireGuard peer (the home server).
    pub peer_endpoint: String,
    /// UDP port of the remote peer. Defaults to `51820`.
    pub peer_port: Option<u16>,
    /// Base64 public key of the remote peer.
    pub peer_public_key: String,
    /// Optional base64 pre-shared key.
    pub peer_preshared_key: Option<String>,
    /// Persistent keep-alive interval. Disabled by default; set e.g. `25s` when
    /// this device is behind NAT so inbound connections can reach it.
    #[serde(default, deserialize_with = "deserialize_option_duration")]
    pub peer_persistent_keepalive: Option<Duration>,
    /// Networks reachable through the tunnel, in CIDR notation (e.g. `10.9.0.0/24`
    /// or `0.0.0.0/0`). Used to route replies and outbound traffic.
    #[serde(default)]
    pub peer_allowed_ips: Vec<String>,
    /// Interval at which the connection status entities are refreshed.
    #[serde(default, deserialize_with = "deserialize_option_duration")]
    pub update_interval: Option<Duration>,
    /// Connections to tunnel. Each rule is an `inbound` or `outbound` forward.
    #[garde(dive)]
    #[serde(default)]
    pub forwards: Vec<ForwardRule>,
}

/// Decodes a base64 WireGuard key into 32 raw bytes.
pub fn decode_key(value: &str, what: &str) -> Result<[u8; 32], String> {
    let bytes = STANDARD
        .decode(value.trim())
        .map_err(|e| format!("invalid base64 {what}: {e}"))?;
    bytes
        .as_slice()
        .try_into()
        .map_err(|_| format!("{what} must decode to 32 bytes, got {}", bytes.len()))
}

/// Parses an IPv4 dotted netmask (e.g. `255.255.255.0`) into a prefix length.
pub fn netmask_to_prefix(netmask: &str) -> Result<u8, String> {
    let addr: std::net::Ipv4Addr = netmask
        .trim()
        .parse()
        .map_err(|e| format!("invalid netmask '{netmask}': {e}"))?;
    let bits = u32::from(addr);
    // A valid mask is a run of set high bits followed by zeros.
    if bits.leading_ones() + bits.trailing_zeros() != 32 {
        return Err(format!("netmask '{netmask}' is not contiguous"));
    }
    Ok(bits.leading_ones() as u8)
}

impl WireGuardConfig {
    pub fn tunnel_ip(&self) -> Result<IpAddr, String> {
        // Tolerate an optional `/prefix` suffix on the address.
        let ip_part = self
            .address
            .split('/')
            .next()
            .unwrap_or(&self.address)
            .trim();
        ip_part
            .parse::<IpAddr>()
            .map_err(|e| format!("invalid address '{}': {e}", self.address))
    }

    /// On-link prefix length for the interface address, derived from `netmask`
    /// (or an inline `address` suffix), defaulting to `/32`.
    pub fn tunnel_prefix(&self) -> Result<u8, String> {
        if let Some(netmask) = &self.netmask {
            return netmask_to_prefix(netmask);
        }
        if let Some(suffix) = self.address.split('/').nth(1) {
            return suffix
                .trim()
                .parse::<u8>()
                .map_err(|e| format!("invalid address prefix in '{}': {e}", self.address));
        }
        Ok(if self.tunnel_ip()?.is_ipv4() { 32 } else { 128 })
    }

    pub fn keepalive_seconds(&self) -> Option<u16> {
        self.peer_persistent_keepalive
            .map(|d| d.as_secs().min(u16::MAX as u64) as u16)
            .filter(|s| *s > 0)
    }

    /// Resolves `peer_endpoint` + `peer_port` to a concrete socket address.
    pub fn resolve_endpoint(&self) -> Result<SocketAddr, String> {
        let port = self.peer_port.unwrap_or(DEFAULT_PEER_PORT);
        let host = self.peer_endpoint.trim();
        // Accept `host` or `host:port`; an explicit port in the string wins.
        let target = if host.parse::<SocketAddr>().is_ok() {
            host.to_string()
        } else {
            format!("{host}:{port}")
        };
        target
            .to_socket_addrs()
            .map_err(|e| format!("failed to resolve peer_endpoint '{target}': {e}"))?
            .next()
            .ok_or_else(|| format!("peer_endpoint '{target}' resolved to no addresses"))
    }

    pub fn outbound_rules(&self) -> impl Iterator<Item = &ForwardRule> {
        self.forwards
            .iter()
            .filter(|f| f.direction == Direction::Outbound)
    }

    pub fn inbound_rules(&self) -> impl Iterator<Item = &ForwardRule> {
        self.forwards
            .iter()
            .filter(|f| f.direction == Direction::Inbound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn netmask_parses_to_prefix() {
        assert_eq!(netmask_to_prefix("255.255.255.255").unwrap(), 32);
        assert_eq!(netmask_to_prefix("255.255.255.0").unwrap(), 24);
        assert_eq!(netmask_to_prefix("255.255.0.0").unwrap(), 16);
        assert_eq!(netmask_to_prefix("0.0.0.0").unwrap(), 0);
        assert!(netmask_to_prefix("255.0.255.0").is_err());
        assert!(netmask_to_prefix("not-a-mask").is_err());
    }

    #[test]
    fn decode_key_requires_32_bytes() {
        let valid = STANDARD.encode([7u8; 32]);
        assert_eq!(decode_key(&valid, "private_key").unwrap(), [7u8; 32]);
        let short = STANDARD.encode([1u8; 16]);
        assert!(decode_key(&short, "private_key").is_err());
        assert!(decode_key("!!!not-base64!!!", "private_key").is_err());
    }

    #[test]
    fn prefix_defaults_and_suffix() {
        let base = WireGuardConfig {
            address: "10.9.0.2".to_string(),
            netmask: None,
            private_key: String::new(),
            peer_endpoint: "h".to_string(),
            peer_port: None,
            peer_public_key: String::new(),
            peer_preshared_key: None,
            peer_persistent_keepalive: None,
            peer_allowed_ips: vec![],
            update_interval: None,
            forwards: vec![],
        };
        assert_eq!(base.tunnel_prefix().unwrap(), 32);

        let with_suffix = WireGuardConfig {
            address: "10.9.0.2/24".to_string(),
            ..base.clone()
        };
        assert_eq!(with_suffix.tunnel_prefix().unwrap(), 24);

        let with_netmask = WireGuardConfig {
            netmask: Some("255.255.0.0".to_string()),
            ..base
        };
        assert_eq!(with_netmask.tunnel_prefix().unwrap(), 16);
    }
}
