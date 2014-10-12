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
//!    `GenericNodeTable` trait and `knodetable::KNodeTable` implementation.
//! 2. Particular implementations, first one is `bt::KRpcService`.

#![crate_name = "dht"]
#![crate_type = "lib"]
#![unstable]

#![feature(if_let)]
#![feature(macro_rules)]
#![feature(phase)]
#![feature(struct_variant)]

extern crate bencode;
extern crate num;
extern crate serialize;
extern crate sync;
#[phase(plugin, link)]
extern crate log;

pub use base::GenericNodeTable;
pub use base::Node;

#[unstable]
pub mod base;
#[experimental]
pub mod bt;
#[unstable]
pub mod knodetable;

mod utils;
