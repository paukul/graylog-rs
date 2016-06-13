use std;
use serde_json;
use std::io::prelude::*;
use std::net::{ToSocketAddrs, UdpSocket};
use std::hash::{Hash, Hasher, SipHasher};
use log::{Log, LogLevel, LogMetadata, LogRecord};
use flate2::Compression;
use flate2::write::GzEncoder;
use chrono::{DateTime, UTC};
use byteorder::{BigEndian, WriteBytesExt};

use GraylogError;

const MAX_PACKET_SIZE: usize = 8192;
const MAGIC_BYTES: u16 = 0x1E0F;
const HEADER_SIZE: usize = 12; // TODO: figure out how to do this with mem::size_of<ChunkHeader>()

struct GraylogLogger<A: ToSocketAddrs> {
    server: A,
    sock: UdpSocket,
    level: LogLevel,
    hostname: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Gelf {
    version: String,
    host: String,
    short_message: String,
    full_message: Option<String>,
    timestamp: Option<i64>,
    level: Option<u8>,
    #[serde(rename="_file")]
    file: String,
    #[serde(rename="_line")]
    line: u32,
}

struct GelfChunks<'a> {
    count: usize,
    total: usize,
    #[allow(dead_code)]
    data: &'a [u8],
    chunks: Vec<&'a [u8]>,
    message_id: u64,
}

struct Chunk<'a> {
    header: ChunkHeader,
    data: &'a [u8],
}

impl<'a> Chunk<'a> {
    fn to_binary(&self) -> Result<Vec<u8>, GraylogError> {
        let mut wrt = vec![];
        try!(wrt.write_u16::<BigEndian>(self.header.magic_bytes));
        try!(wrt.write_u64::<BigEndian>(self.header.message_id));
        try!(wrt.write_u8(self.header.seq_number));
        try!(wrt.write_u8(self.header.seq_count));
        wrt.extend(self.data.iter());
        Ok(wrt)
    }
}

#[derive(Debug)]
struct ChunkHeader {
    magic_bytes: u16,
    message_id: u64,
    seq_number: u8,
    seq_count: u8,
}

impl<'a> GelfChunks<'a> {
    fn new(data: &'a [u8], message_id: u64) -> GelfChunks<'a> {
        let chunk_size = MAX_PACKET_SIZE - HEADER_SIZE;
        let mut chunks = data.chunks(chunk_size).collect::<Vec<&[u8]>>();
        chunks.reverse();

        GelfChunks {
            count: 0,
            total: chunks.len(),
            chunks: chunks,
            data: data,
            message_id: message_id,
        }
    }
}

impl<'a> Iterator for GelfChunks<'a> {
    type Item = Chunk<'a>;

    fn next(&mut self) -> Option<Chunk<'a>> {
        self.chunks.pop().map(|data| {
            let header = ChunkHeader {
                magic_bytes: MAGIC_BYTES,
                message_id: self.message_id,
                seq_number: self.count as u8,
                seq_count: self.total as u8,
            };
            println!("{:?}", header);

            self.count += 1;

            Chunk {
                header: header,
                data: data,
            }
        })
    }
}

pub fn init<A: ToSocketAddrs + 'static>(addr: A, level: LogLevel) -> Result<(), GraylogError>
    where A: std::marker::Send + std::marker::Sync
{
    use log;
    use std::process::Command;
    let sock = try!(UdpSocket::bind("0.0.0.0:0"));

    try!(log::set_logger(|max_log_level| {
        max_log_level.set(level.to_log_level_filter());
        let hostname = Command::new("hostname")
            .arg("-f")
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).into_owned())
            .unwrap_or("unknown".to_string());

        Box::new(GraylogLogger {
            level: level,
            server: addr,
            sock: sock,
            hostname: hostname.trim().to_string(),
        })
    }));
    Ok(())
}

impl Gelf {
    fn message_id(&self) -> u64 {
        use chrono::Timelike;
        let mut s = SipHasher::new();
        self.host.hash(&mut s);
        self.timestamp.hash(&mut s);
        UTC::now().nanosecond().hash(&mut s);
        s.finish()
    }
}

impl<A: ToSocketAddrs> GraylogLogger<A> {
    fn send_as_gelf(&self, record: &LogRecord) -> Result<usize, GraylogError> {
        let utc: DateTime<UTC> = UTC::now();

        let location = record.location();
        let gelf = Gelf {
            version: "1.1".to_string(),
            short_message: format!("{}", record.args()),
            full_message: None,
            timestamp: Some(utc.timestamp()),
            level: None,
            host: self.hostname.clone(),
            file: location.file().to_string(),
            line: location.line(),
        };

        let json = try!(serde_json::to_string(&gelf));
        let mut e = GzEncoder::new(Vec::new(), Compression::Default);
        try!(e.write_all(json.as_bytes()));

        let compressed_bytes = try!(e.finish());
        if compressed_bytes.len() > MAX_PACKET_SIZE {
            self.send_chunked_gelf(&compressed_bytes, gelf.message_id())
        } else {
            self.send(&compressed_bytes)
        }
    }

    fn send_chunked_gelf(&self, buffer: &[u8], message_id: u64) -> Result<usize, GraylogError> {
        let chunks = GelfChunks::new(buffer, message_id);
        println!("Graylog message_id: {:x}", message_id);
        for chunk in chunks {
            let data = try!(chunk.to_binary());
            try!(self.send(&data));
        }
        Ok(1)
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

#[test]
fn test_gelf_serialization() {
    use serde_json::Value;
    let file = "asdfasdfadsf.rs".to_string();
    let line = 123;
    let gelf = Gelf {
        version: "1.1".to_string(),
        host: "somehost".to_string(),
        short_message: "some short message".to_string(),
        full_message: Some("Full message".to_string()),
        timestamp: Some(UTC::now().timestamp()),
        level: None,
        file: file.clone(),
        line: line,
    };
    let json = serde_json::to_string(&gelf).unwrap();
    println!("Json\n{}", json);

    let data: Value = serde_json::from_str(&json).unwrap();
    let obj = data.as_object().unwrap();
    assert_eq!(obj.get("_file").unwrap().as_string().unwrap(), file);
    assert_eq!(obj.get("_line").unwrap().as_u64().unwrap(), line as u64);

    let deserialized_gelf: Gelf = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized_gelf, gelf);
    println!("{:?}", obj);
}

#[cfg(test)]
mod tests {
    use log;

    use super::*;

    #[test]
    fn test_it_works() {
        assert!(init("192.168.1.1", log::LogLevel::Debug).is_ok());
    }

}
