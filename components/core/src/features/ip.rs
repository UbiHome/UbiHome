use std::net::IpAddr;

use getifs::{best_local_addrs, interfaces};
use log::debug;

pub fn get_ip_address() -> Result<IpAddr, String> {
    let network_interfaces = interfaces().map_err(|e| format!("Failed to list interfaces: {e}"))?;

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
    }

    let mut all_addrs: Vec<IpAddr> = best_local_addrs()
        .unwrap()
        .into_iter()
        .map(|addr| addr.addr())
        .collect();
    // Will only contain addresses from interfaces with best default routes

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
