use crate::{Device, DeviceAttributes, Result};
use log::{debug, info};
use std::{thread, time::Duration};

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

    pub fn scan(&mut self) -> Result<Device> {
        info!("scan for sonoff devices in the current network");
        let mut attrs = DeviceAttributes::default();
        loop {
            for response in
                mdns::discover::all(&self.service_name)?.timeout(Duration::from_millis(500))
            {
                for record in response?.records() {
                    if let Some(device) = attrs.add(record) {
                        if !self.devices.contains(&device) {
                            self.devices.push(device.clone());
                            return Ok(device);
                        } else {
                            debug!("ignore already seen device: {}", device.name);
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(1000));
        }
    }

    pub fn scan_loop<F, T>(&mut self, mut cb: F) -> Result<()>
    where
        F: FnMut(Result<Device>) -> Result<T>,
    {
        loop {
            cb(self.scan())?;
        }
    }
}
