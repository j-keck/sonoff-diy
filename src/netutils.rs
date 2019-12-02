use crate::*;
use ipnet::*;
use std::net::{IpAddr, SocketAddr};

pub fn matching_host_ip_for(other: &IpAddr) -> Result<IpAddr> {
    let other: IpAddr = other.to_string().parse().unwrap();

    if let Some(ip) = host_ips()?
        .into_iter()
        .find(|host_ip| host_ip.contains(&other))
    {
        Ok(ip.addr())
    } else {
        Err(Error::GenericError { msg: "no usable interface found".into() })
    }
}

pub fn host_ips() -> Result<Vec<IpNet>> {
    fn prefix_len(addr: SocketAddr) -> u8 {
        match addr.ip() {
            IpAddr::V4(addr) => addr.octets().to_vec(),
            IpAddr::V6(addr) => addr.octets().to_vec(),
        }
        .iter()
        .map(|n| n.count_ones() as u8)
        .sum()
    }

    ifaces::ifaces()?
        .into_iter()
        .filter_map(|iface| match (iface.addr, iface.mask) {
            (Some(SocketAddr::V4(addr)), Some(mask)) => {
                Some(Ok(Ipv4Net::new(*addr.ip(), prefix_len(mask)).unwrap().into()))
            }
            (Some(SocketAddr::V6(addr)), Some(mask)) => {
                Some(Ok(Ipv6Net::new(*addr.ip(), prefix_len(mask)).unwrap().into()))
            }
            _ => None,
        })
        .collect()
}
