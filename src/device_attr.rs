use crate::Device;
use log::debug;
use mdns::{Record, RecordKind};
use std::collections::HashMap;
use std::net::IpAddr;

type Attrs = (Option<Vec<String>>, Option<IpAddr>, Option<u16>);

#[derive(Debug)]
pub struct DeviceAttributes(HashMap<String, Attrs>);

impl DeviceAttributes {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add(&mut self, record: &Record) -> Option<Device> {
        let name = record
            .name
            .chars()
            .take_while(|c| c != &'.')
            .collect::<String>();

        let attrs = self.0.entry(name.clone()).or_insert((None, None, None));
        match &record.kind {
            RecordKind::TXT(txt) => attrs.0 = Some(txt.clone()),
            RecordKind::A(addr) => attrs.1 = Some(addr.clone().into()),
            RecordKind::AAAA(addr) => attrs.1 = Some(addr.clone().into()),
            RecordKind::SRV { port, .. } => attrs.2 = Some(*port),
            _ => (),
        }

        self.extract_device(&name)
    }

    fn extract_device(&mut self, name: &str) -> Option<Device> {
        if let Some((Some(desc), Some(ip), Some(port))) = self.0.get_mut(name) {
            if let Some(id) = DeviceAttributes::lookup_id(desc) {
                let device = Device::new(name, &id, desc, *ip, *port);

                self.0.remove(name);
                return Some(device);
            }
        }
        None
    }

    fn lookup_id(txt: &[String]) -> Option<String> {
        txt.iter().find(|s| s.starts_with("id=")).map(|s| {
            // id is ascii
            let id = s[3..].to_string();

            debug!("id field found: '{}' - extracted id: '{}' the id", s, id);
            id
        })
    }
}

impl Default for DeviceAttributes {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dns_parser::Class;
    use std::net::Ipv4Addr;

    #[test]
    fn add_should_shorten_the_name() {
        let mut attrs = DeviceAttributes::default();
        attrs.add(&mk_txt_record(
            "eWeLink_ababababab._ewelink._tcp.local",
            Vec::new(),
        ));
        if let Some(_) = attrs.0.get("eWeLink_ababababab") {
            // ok
        } else {
            panic!("attribute not found");
        }
    }

    #[test]
    fn add_should_return_the_device_when_all_attributes_are_present() {
        let mut attrs = DeviceAttributes::default();

        assert_eq!(
            attrs.add(&mk_a_record(
                "eWeLink_ababababab.local",
                Ipv4Addr::new(127, 0, 0, 1),
            )),
            None
        );

        assert_eq!(
            attrs.add(&mk_srv_record(
                "eWeLink_ababababab._ewelink._tcp.local",
                8081
            )),
            None
        );

        if let Some(device) = attrs.add(&mk_txt_record(
            "eWeLink_ababababab._ewelink._tcp.local",
            vec![
                "txtvers=1",
                "id=ababababab",
                "type=diy_plug",
                "apivers=1",
                "seq=1",
                concat!(
                    "data1={\"switch\":\"off\",",
                    "\"startup\":\"off\",\"pulse\":\"off\"",
                    ",\"pulseWidth\":500,\"rssi\":-28}"
                ),
            ],
        )) {
            assert_eq!(device.id, "ababababab");
            assert_eq!(device.ip, Ipv4Addr::new(127, 0, 0, 1));
            assert_eq!(device.port, 8081);
        } else {
            panic!("device not found");
        }
    }

    fn mk_txt_record(name: &str, v: Vec<&str>) -> Record {
        Record {
            name: name.to_string(),
            class: Class::IN,
            ttl: 300,
            kind: RecordKind::TXT(v.into_iter().map(String::from).collect()),
        }
    }

    fn mk_srv_record(name: &str, port: u16) -> Record {
        Record {
            name: name.to_string(),
            class: Class::IN,
            ttl: 300,
            kind: RecordKind::SRV {
                priority: 0,
                weight: 0,
                port,
                target: "target".to_string(),
            },
        }
    }

    fn mk_a_record(name: &str, ip: Ipv4Addr) -> Record {
        Record {
            name: name.to_string(),
            class: Class::IN,
            ttl: 300,
            kind: RecordKind::A(ip.into()),
        }
    }
}
