//! The graylog crate provides graylog (https://www.graylog.org/) compatible logging based on
//! the log crate and its logging facade (https://doc.rust-lang.org/log/log/index.html).
//!
//! # Examples
//!
//! ```
//! #[macro_use] extern crate log;
//! extern crate graylog;
//!
//! fn main() {
//!   graylog::logger::init();
//!   info!("The logger macros now use the graylog logging backend");
//! }
//! ```


// Needed to derive `Serialize` on ServiceProperties
#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate flate2;
extern crate chrono;

pub mod logger;
