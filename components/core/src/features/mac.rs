// use mac_address::MacAddressIterator;
// use nix::ifaddrs::getifaddrs;

pub fn get_mac_address() -> Result<String, String> {
    return Ok("00:00:00:00:00:00".to_string());

    let test = mac_address::get_mac_address().unwrap();

    // let ifiter = getifaddrs()?;

    // for interface in ifiter {
    //     println!("Interface: {:?} {:?}", interface.interface_name, interface.address);
    // }


    // for addr in MacAddressIterator::new().unwrap() {
    //     println!("{}", addr);
    // }

    // match test {
    //     Some(mac) => {
    //         let mac = format!("{:?}", mac);
    //         return Ok(Some(mac));
    //     }
    //     None => {
    //         return Ok(None);
    //     }
    // }
}
