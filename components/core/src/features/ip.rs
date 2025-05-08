use std::net::IpAddr;

use local_ip_address::list_afinet_netifas;
use log::debug;


pub fn get_ip_address() -> Result<IpAddr, String> {

    let network_interfaces = list_afinet_netifas().unwrap();
    debug!("Detected Networks:");
    for (name, ip) in network_interfaces.iter() {
        debug!("{}:\t{:?}", name, ip);

        // Windows uses "Wi-Fi"
        if name == "Wi-Fi" {
            if ip.is_ipv4() {
                return Ok(*ip);
            }
        }

        // Linux uses "wlan0" or "eth0"
        if name == "wlan0" || name == "eth0" {
            if ip.is_ipv4() {
                return Ok(*ip);
            }
        }

    }
    let default_ip: IpAddr = "0.0.0.0".parse().unwrap();
    Ok(network_interfaces.last().map(|(_,ip)| *ip).unwrap_or(default_ip))
}