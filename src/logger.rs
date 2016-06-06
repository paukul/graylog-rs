use std;
use std::io::prelude::*;
use std::net::{UdpSocket, ToSocketAddrs};
use std::fmt;

use log;
use log::{Log, LogRecord, LogLevel, LogMetadata, LogLevelFilter};
use flate2::Compression;
use flate2::write::GzEncoder;
use chrono::{DateTime, UTC};
use serde_json;

pub struct GraylogLogger<A: ToSocketAddrs> {
    server: A,
    sock: UdpSocket,
    level: LogLevel,
    hostname: String,
}

#[derive(Serialize, Deserialize)]
struct Gelf {
    version: String,
    host: String,
    short_message: String,
    full_message: Option<String>,
    timestamp: Option<i64>,
    level: Option<u8>,
}

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

impl std::error::Error for GraylogError {
    fn description(&self) -> &str {
        match *self {
            GraylogError::SetLoggerError(ref err) => err.description(),
            GraylogError::Io(ref err) => err.description(),
            GraylogError::JsonError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
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

pub fn init<A: ToSocketAddrs + 'static>(addr: A) -> Result<(), GraylogError>
    where A: std::marker::Send + std::marker::Sync
{
    use log;
    use std::process::Command;
    let sock = try!(UdpSocket::bind("0.0.0.0:0"));

    try!(log::set_logger(|max_log_level| {
        max_log_level.set(LogLevelFilter::Debug);
        let hostname = Command::new("hostname")
            .arg("-f")
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).into_owned())
            .unwrap_or("unknown".to_string());

        Box::new(GraylogLogger {
            level: LogLevel::Debug,
            server: addr,
            sock: sock,
            hostname: hostname,
        })
    }));
    Ok(())
}

impl<A: ToSocketAddrs> GraylogLogger<A> {
    fn send_as_gelf(&self, record: &LogRecord) -> Result<usize, GraylogError> {
        let mut e = GzEncoder::new(Vec::new(), Compression::Default);
        let utc: DateTime<UTC> = UTC::now();

        let gelf = Gelf {
            version: "1.1".to_string(),
            short_message: format!("{}", record.args()),
            full_message: None,
            timestamp: Some(utc.timestamp()),
            level: None,
            host: self.hostname.clone(),
        };

        let json = try!(serde_json::to_string(&gelf));
        // println!("gelf json:\n {}", json);
        try!(e.write(json.as_bytes()));

        let compressed_bytes = try!(e.finish());
        self.send(&compressed_bytes)
    }

    fn send(&self, buffer: &[u8]) -> Result<usize, GraylogError> {
        let s = try!(self.sock.send_to(buffer, &self.server).or_else(|e| {
            println!("Error writing to Graylog: {}", e);
            Err(e)
        }));
        Ok(s)
    }
}

impl<A: ToSocketAddrs> Log for GraylogLogger<A>
    where A: std::marker::Send + std::marker::Sync
{
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            if self.send_as_gelf(record).is_err() {
                println!("{} {} - {}", UTC::now(), record.level(), record.args());
            };
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_it_works() {
        assert!(true);
    }
}
