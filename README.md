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

# Development

Nightly rust is required for development for now (mostly for clippy).

The project contains a docker-compose.yml that allows to setup a graylog cluster for testing.
On OSX one has to enable NFS for the docker-machine (https://github.com/adlogix/docker-machine-nfs) in order to make it work with the docker volumes with
the custom log4j config (required to see debug logs). This should no longer be necessary as soon as docker for OSX is released.

The `GRAYLOG_REST_TRANSPORT_URI` in the docker-compose image probably also has to be modified based on the IP of your docker-machine ip.
