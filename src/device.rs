use crate::Binary;
use log::{debug, info};
use serde_json::{json, Value};
use std::error::Error;
use std::net::IpAddr;

#[derive(Debug, PartialEq, Clone)]
pub struct Device {
    pub name: String,
    pub id: String,
    pub desc: Vec<String>,
    pub ip: IpAddr,
    pub port: u16,
}

impl Device {
    pub fn new(name: &str, id: &str, desc: &[String], ip: IpAddr, port: u16) -> Self {
        Device {
            name: name.to_string(),
            id: id.to_string(),
            desc: desc.to_vec(),
            ip,
            port,
        }
    }

    pub fn info(&self) -> Result<Value, Box<dyn Error>> {
        let payload = json!({
            "deviceid": &self.id,
            "data": {},
        });

        self.post("info", payload)?
            .get_mut("data")
            .map(|v| v.take())
            .ok_or("'data' in response not found".into())
    }

    pub fn unlock(&self) {
        let payload = json!({
            "deviceid": &self.id,
            "data": {},
        });

        info!("unlock");
        println!("{:?}", self.post("ota_unlock", payload));
    }

    pub fn wifi<S>(&self, ssid: S, pwd: S)
    where
        S: Into<String>,
    {
        let payload = json!({
            "deviceid": &self.id,
            "data": {
                "ssid": ssid.into(),
                "password": pwd.into(),
            },
        });

        info!("wifi");
        println!("{:?}", self.post("wifi", payload));
    }

    pub fn flash<S>(&self, endpoint: S, bin: &Binary)
        where S: Into<String>
    {
        let payload = json!({
            "deviceid": &self.id,
            "data": {
                "downloadUrl": endpoint.into(),
                "sha256sum": bin.sha256sum(),
            },
        });

        info!("flash");
        println!("{:?}", self.post("ota_flash", payload));
    }

    fn post<S>(&self, p: S, payload: Value) -> Result<Value, Box<dyn Error>>
    where
        S: Into<String>,
    {
        let client = reqwest::Client::new();
        let url = format!("http://{}:{}/zeroconf/{}", self.ip, self.port, p.into());
        debug!("post to: {}, payload: {:?}", url, payload);
        client
            .post(&url)
            .json(&payload)
            .send()?
            .json()
            .map_err(|e| e.into())
    }
}
