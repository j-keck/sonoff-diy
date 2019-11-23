use crate::*;
use log::debug;

use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

pub struct DeviceCache(Vec<Device>, PathBuf);

impl DeviceCache {
    pub fn new(devices: Vec<Device>) -> Self {
        Self(devices, ".sonoff-diy-cache.json".into())
    }

    pub fn add(&mut self, device: &Device) -> Result<()> {
        if self.0.iter().find(|d| d.id == device.id).is_none() {
            debug!("add device to cache: {}", device);
            self.0.push(device.clone());
            self.save()
        } else {
            debug!("device already in the cache - ignore it");
            Ok(())
        }
    }

    pub fn devices(&self) -> Vec<Device> {
        self.0.clone()
    }

    pub fn lookup<S>(&self, device_id: S) -> Result<Device>
    where
        S: Into<String>,
    {
        let device_id = device_id.into();
        self.0
            .iter()
            .find(|d| d.id == device_id)
            .cloned()
            .ok_or(Error::DeviceNotFound { device_id })
    }

    pub fn load() -> Result<Self>
    {
        let mut cache = DeviceCache::new(Vec::new());
        debug!("load cache from {}", cache.1.display());
        let file = File::open(&cache.1)?;
        let reader = BufReader::new(file);
        cache.0 = serde_json::from_reader(reader)?;
        debug!("cache content: {:?}", cache.0);
        Ok(cache)
    }

    pub fn save(&self) -> Result<()> {
        debug!("save cache to: {}", self.1.display());
        let file = File::create(&self.1)?;
        let writer = BufWriter::new(file);
        Ok(serde_json::to_writer(writer, &self.0)?)
    }
}

impl Default for DeviceCache {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
