use crate::*;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value, to_string_pretty, from_str};
use std::{fmt, net::IpAddr};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

    pub fn info(&self) -> Result<String> {
        let payload = json!({
            "deviceid": &self.id,
            "data": {},
        });

        match self
            .post("info", payload)?
            .get("data")
        {
            Some(Value::String(ref data)) => {
                Ok(to_string_pretty(&from_str::<Value>(data)?)?)
            },
            _ => Err(Error::JSONLookupError {
                msg: "'data' in response not found".to_string(),
            }),
        }
    }

    pub fn switch(&self, state: SwitchState) -> Result<String> {
        let state = match state {
            SwitchState::On => "on",
            SwitchState::Off => "off",
        };

        let payload = json!({
            "deviceid": &self.id,
            "data": {
                "switch": state,
            },
        });
        self.post_("switch", payload)
    }

    pub fn unlock(&self) -> Result<String> {
        let payload = json!({
            "deviceid": &self.id,
            "data": {},
        });

        self.post_("ota_unlock", payload)
    }

    pub fn wifi<S>(&self, ssid: S, pwd: S) -> Result<String>
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

        self.post_("wifi", payload)
    }

    pub fn flash<S>(&self, endpoint: S, bin: &Binary) -> Result<String>
    where
        S: Into<String>,
    {
        let payload = json!({
            "deviceid": &self.id,
            "data": {
                "downloadUrl": endpoint.into(),
                "sha256sum": bin.sha256sum(),
            },
        });

        self.post_("ota_flash", payload)
    }

    fn post<S>(&self, p: S, payload: Value) -> Result<Value>
    where
        S: Into<String>,
    {
        let client = reqwest::Client::new();
        let url = format!("http://{}:{}/zeroconf/{}", self.ip, self.port, p.into());
        debug!("post to: {} with payload: {:?}", url, to_string_pretty(&payload)?);
        let res = client
            .post(&url)
            .json(&payload)
            .send()?
            .json()
            .map_err(|e| e.into());
        debug!("response: {:#?}", res);
        res
    }

    fn post_<S>(&self, p: S, payload: Value) -> Result<String>
    where
        S: Into<String>
    {
        let value = self.post(p, payload)?;
        Ok(to_string_pretty(&value)?)
    }

}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "device-name: {}, id: {}, ip: {}",
            self.name, self.id, self.ip
        )
    }
}
