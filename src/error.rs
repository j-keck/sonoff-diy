use snafu::{ErrorCompat, Snafu};
use std::io;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error while scanning: {}", source))]
    ScanError { source: mdns::Error },

    #[snafu(display("Device with id: {} not found", device_id))]
    DeviceNotFound { device_id: String },

    #[snafu(display("IO error: {}", source))]
    IOError { source: io::Error },

    #[snafu(display("JSON parser error: {}", source))]
    JSONError { source: serde_json::Error },

    #[snafu(display("JSON lookup error: {}", msg))]
    JSONLookupError { msg: String },

    #[snafu(display("Request error: {}", source))]
    ReqwestError { source: reqwest::Error },

    #[snafu(display("Invalid binary: {}", msg))]
    InvalidBinary { msg: String },

    #[snafu(display("Parser error: {}", msg))]
    ParserError { msg: String },

    #[snafu(display("Invalid request: {}", msg))]
    InvalidRequest { msg: String },

    #[snafu(display("{}", msg))]
    GenericError { msg: String },
}

impl Error {
    pub fn print_backtrace(&self) {
        if let Some(backtrace) = ErrorCompat::backtrace(self) {
            eprintln!("{}", backtrace);
        }
    }
}

impl From<mdns::Error> for Error {
    fn from(source: mdns::Error) -> Self {
        Error::ScanError { source }
    }
}

impl From<io::Error> for Error {
    fn from(source: io::Error) -> Self {
        Error::IOError { source }
    }
}

impl From<serde_json::Error> for Error {
    fn from(source: serde_json::Error) -> Self {
        Error::JSONError { source }
    }
}

impl From<reqwest::Error> for Error {
    fn from(source: reqwest::Error) -> Self {
        Error::ReqwestError { source }
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Error::ParserError {
            msg: err.to_string(),
        }
    }
}
