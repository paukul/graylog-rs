//! The graylog crate provides graylog (https://www.graylog.org/) compatible logging based on
//! the log crate and its logging facade (https://doc.rust-lang.org/log/log/index.html).
//!
//! # Examples
//!
//! ```rust
//! #[macro_use]
//! extern crate log;
//! extern crate graylog;
//!
//! fn main() {
//!   graylog::logger::init("192.168.99.100:5555", log::LogLevel::Debug).unwrap();
//!   info!("The logger macros now use the graylog logging backend");
//! }
//! ```


// Needed to derive `Serialize` on ServiceProperties
#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
// #![warn(missing_docs)]
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate flate2;
extern crate chrono;
extern crate byteorder;

use std::fmt;
use std::error::Error;

pub mod logger;

#[derive(Debug)]
pub enum GraylogError {
    SetLoggerError(log::SetLoggerError),
    Io(std::io::Error),
    JsonError(serde_json::Error),
}

impl fmt::Display for GraylogError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GraylogError::SetLoggerError(ref err) => err.fmt(f),
            GraylogError::Io(ref err) => err.fmt(f),
            GraylogError::JsonError(ref err) => err.fmt(f),
        }
    }
}

impl Error for GraylogError {
    fn description(&self) -> &str {
        match *self {
            GraylogError::SetLoggerError(ref err) => err.description(),
            GraylogError::Io(ref err) => err.description(),
            GraylogError::JsonError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            GraylogError::SetLoggerError(ref err) => Some(err),
            GraylogError::Io(ref err) => Some(err),
            GraylogError::JsonError(ref err) => Some(err),
        }
    }
}

impl From<log::SetLoggerError> for GraylogError {
    fn from(err: log::SetLoggerError) -> GraylogError {
        GraylogError::SetLoggerError(err)
    }
}

impl From<std::io::Error> for GraylogError {
    fn from(err: std::io::Error) -> GraylogError {
        GraylogError::Io(err)
    }
}

impl From<serde_json::Error> for GraylogError {
    fn from(err: serde_json::Error) -> GraylogError {
        GraylogError::JsonError(err)
    }
}
