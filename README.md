# graylog-rs

## WIP status

This is not really usable yet. It sends gelf via udp but doesnt do chunking yet. 
Also it's barely tested, documented and lacking all transports other than UDP. 

# Usage

```
#[macro_use] extern crate log;
extern crate graylog;

fn main() {
  graylog::logger::init();
  info!("The logger macros now use the graylog logging backend");
}
