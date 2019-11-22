use crate::{Device, DeviceAttributes};
use log::{debug, info};
use std::{error::Error, thread, time::Duration};

pub struct Scanner {
    service_name: String,
    devices: Vec<Device>,
}

impl Scanner {
    pub fn new(service_name: String) -> Self {
        Self {
            service_name,
            devices: Vec::new(),
        }
    }

    pub fn scan(&mut self) -> Result<Device, Box<dyn Error>> {
        info!("scan for sonoff devices in the current network");
        let mut attrs = DeviceAttributes::default();
        loop {
            for response in
                mdns::discover::all(&self.service_name)?.timeout(Duration::from_millis(50))
            {
                for record in response?.records() {
                    if let Some(device) = attrs.add(record) {
                        if self.not_contains(&device) {
                            self.devices.push(device.clone());
                            return Ok(device);
                        } else {
                            debug!("ignore already seen device: {}", device.name);
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(500));
        }
    }

    fn not_contains(&self, device: &Device) -> bool {
        !self.devices.contains(device)
    }
}
