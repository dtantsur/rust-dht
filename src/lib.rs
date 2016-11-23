// Copyright 2014 Dmitry "Divius" Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

//! Distributed Hash Table.
//!
//! The goal of this project is to provide flexible implementation of DHT
//! for different kind of Rust applications. There will be loosely coupled parts:
//!
//! 1. DHT neighborhood table implementation, will be represented by
//!    `GenericNodeTable` trait and `KNodeTable` implementation.
//! 2. Generic DHT logic implementation in `Service` and `service::Handler`
//!    structures.
//! 3. Generic bits for implementing protocols in `service::Handler` structure
//!    and `protocol` module.
//! 4. (In the future) simple implementations for testing purposes.

#![crate_name = "dht"]
#![crate_type = "lib"]

#[macro_use]
extern crate log;
extern crate rand;
extern crate rustc_serialize;

pub use base::GenericId;
pub use base::GenericNodeTable;
pub use base::Node;
pub use knodetable::KNodeTable;
pub use service::Service;

mod base;
mod knodetable;
pub mod protocol;
pub mod service;
mod utils;
