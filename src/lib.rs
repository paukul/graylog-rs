// Needed to derive `Serialize` on ServiceProperties
#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate flate2;
extern crate chrono;

pub mod logger;
