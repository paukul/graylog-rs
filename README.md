# graylog-rs

## WIP status

This is an early WIP. It sends gziped Gelf via UDP and does support chunking for large messages.
However, it's barely tested, documented and lacking all transports other than UDP for now.

The API is likely to change when it supports different transports.

It also doesn't support structured logging yet. I'll have a look if I will leverage projects like
[emit](https://github.com/emit-rs/emit) for structured logging.

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
On OSX one has to enable NFS for the docker-machine via [docker-machine-nfs](https://github.com/adlogix/docker-machine-nfs) in order to make it work with the docker volumes with
the custom log4j config (required to see debug logs). This should no longer be necessary as soon as docker for OSX is released.

The `GRAYLOG_REST_TRANSPORT_URI` in the docker-compose image probably also has to be modified based on the IP of your docker-machine ip.
