use std::net::IpAddr;

use getifs::{best_local_addrs, interfaces};
use log::debug;

pub fn get_ip_address() -> Result<IpAddr, String> {
    let network_interfaces = interfaces().map_err(|e| format!("Failed to list interfaces: {e}"))?;

    // Addresses that belong to an interface exposing a MAC address. Interfaces
    // without a MAC (loopback, VPN/tunnel interfaces such as `utun*`) can't act
    // as a real device on the network, so we exclude their addresses.
    let mut mac_bearing_addrs: Vec<IpAddr> = Vec::new();
    debug!("Detected Networks:");
    for interface in network_interfaces.iter() {
        let interface_name = interface.name().as_str().to_string();
        let interface_addrs = interface.addrs().map_err(|e| e.to_string())?;

        debug!(
            "{}[{:?}]:\t{:?}",
            interface_name,
            interface.mac_addr(),
            interface_addrs
        );

        if interface.mac_addr().is_some() {
            mac_bearing_addrs.extend(interface_addrs.iter().map(|addr| addr.addr()));
        }
    }

    let mut all_addrs: Vec<IpAddr> = best_local_addrs()
        .unwrap()
        .into_iter()
        .map(|addr| addr.addr())
        // Will only contain addresses from interfaces with best default routes
        .filter(|addr| mac_bearing_addrs.contains(addr))
        .collect();

    // The best default route may run over an interface without a MAC (e.g. a
    // VPN). In that case fall back to any interface that does expose a MAC so we
    // still advertise a real device instead of failing.
    if all_addrs.is_empty() {
        debug!(
            "No best-route interface exposes a MAC address; falling back to any MAC-bearing interface"
        );
        all_addrs = mac_bearing_addrs;
    }

    all_addrs.sort_by(|a, b| match (a, b) {
        (IpAddr::V4(_), IpAddr::V6(_)) => std::cmp::Ordering::Less,
        (IpAddr::V6(_), IpAddr::V4(_)) => std::cmp::Ordering::Greater,
        _ => std::cmp::Ordering::Equal,
    });

    debug!("Best local addresses: {:?}", all_addrs);
    all_addrs
        .first()
        .cloned()
        .ok_or("No valid IP address found".to_string())
}

// use mac_address::MacAddressIterator;
// use nix::ifaddrs::getifaddrs;

pub fn get_network_mac_address(ip: IpAddr) -> Result<String, String> {
    let network_interfaces = interfaces().map_err(|e| format!("Failed to list interfaces: {e}"))?;

    for interface in network_interfaces.iter() {
        let interface_addrs = interface.addrs().map_err(|e| e.to_string())?;
        let contains_ip = interface_addrs.iter().any(|addr| addr.addr() == ip);

        if contains_ip {
            if let Some(mac) = interface.mac_addr() {
                return Ok(mac.to_string());
            } else {
                return Err("No MAC address found".to_string());
            }
        }
    }

    Err("Ip Address not found".to_string())
}
