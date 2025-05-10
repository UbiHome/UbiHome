use std::{f32::consts::E, net::IpAddr};

use log::debug;
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};


pub(crate) fn get_primary_interface() -> Result<NetworkInterface, String> {
    let network_interfaces = NetworkInterface::show().unwrap();

    debug!("Detected Networks:");
    for interface in network_interfaces.iter() {
        debug!("{}[{:?}]:\t{:?}", &interface.name, &interface.mac_addr, &interface.addr);

        // Windows uses "Wi-Fi"
        if interface.name == "Wi-Fi" {
           return Ok(interface.clone());
        }

        // Linux uses "wlan0" or "eth0"
        if interface.name == "wlan0" || interface.name == "eth0" {
           return Ok(interface.clone());
        }
    }
    Err("No valid network interface found".to_string())
}

pub fn get_ip_address() -> Result<IpAddr, String> {
    let network_interface = get_primary_interface().unwrap();
    let mut addresses: Vec<IpAddr> = network_interface
    .addr
    .iter()
    .filter_map(|addr| match addr {
        Addr::V4(v4) => Some(IpAddr::V4(v4.ip)),
        Addr::V6(v6) => Some(IpAddr::V6(v6.ip)),
    })
    .collect();

    addresses.sort_by(|a, b| match (a, b) {
        (IpAddr::V4(_), IpAddr::V6(_)) => std::cmp::Ordering::Less,
        (IpAddr::V6(_), IpAddr::V4(_)) => std::cmp::Ordering::Greater,
        _ => std::cmp::Ordering::Equal,
    });

    addresses
        .into_iter()
        .next()
        .ok_or_else(|| "No valid IP address found".to_string())
    // let default_ip: IpAddr = "0.0.0.0".parse().unwrap();
    // Ok(network_interfac.last().map(|(_,ip)| *ip).unwrap_or(default_ip))
}

// use mac_address::MacAddressIterator;
// use nix::ifaddrs::getifaddrs;

pub fn get_network_mac_address(ip: IpAddr) -> Result<String, String> {
    let network_interfaces = NetworkInterface::show().unwrap();
    for interface in network_interfaces.iter() {
        let contains_ip = interface
            .addr
            .iter()
            .any(|addr| match addr {
                Addr::V4(v4) => v4.ip == ip,
                Addr::V6(v6) => v6.ip == ip,
            });
        if contains_ip {
            if let Some(mac) = &interface.mac_addr {
                return Ok(mac.clone());
            } else {
                return Err("No MAC address found".to_string());
            }
        }
    }
    return Err("Ip Address not found".to_string());
}
