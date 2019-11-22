use std::error::Error;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Binary(PathBuf);

impl Binary {
    pub fn new<P>(path: P) -> Result<Binary, Box<dyn Error>>
    where
        P: Into<PathBuf>,
    {
        let path = path.into();

        if !path.exists() {
            return Err("binary not found!".into());
        }

        if !path.is_file() {
            return Err("binary must be a file".into());
        }

        Ok(Binary(path))
    }

    pub fn basename(&self) -> String {
        self.0.file_name().unwrap().to_string_lossy().to_string()
    }

    pub fn read(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        std::fs::read(&self.0).map_err(|e| e.into())
    }

    pub fn sha256sum(&self) -> String {
        let content = self.read().unwrap();
        String::from_utf8_lossy(&hmac_sha256::Hash::hash(&content).to_vec()).to_string()
    }
}

impl fmt::Display for Binary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.display())
    }
}
