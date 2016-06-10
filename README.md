# graylog-rs

## WIP status

This is not really usable yet. It sends gelf via udp maybe it even chunks large gelf messages, who knows.
Also it's barely tested, documented and lacking all transports other than UDP. 

# Usage

```rust
#[macro_use]
extern crate log;
extern crate graylog;

fn main() {
  graylog::logger::init("192.168.99.100:5555", log::LogLevel::Debug);
  info!("The logger macros now use the graylog logging backend");
}
```
