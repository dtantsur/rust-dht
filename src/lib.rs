//! Distributed Hash Table.
//!
//! The goal of this project is to provide flexible implementation of DHT
//! for different kind of Rust applications. There will be 3 loosely coupled
//! parts:
//!
//! 1. DHT neighborhood table implementation, will be represented by
//!    `GenericNodeTable` trait and `kademlia::NodeTable` implementation.
//! 2. RPC implementation, will be represented by `GenericRpc` trait,
//!    exact implementation to by defined, likely some JSON-over-UDP.
//! 3. Generic struct `Service<TNodeTable: GenericNodeTable, TRpc: GenericRpc>`
//!    that will connect previous two.

#![crate_name = "dht"]
#![crate_type = "lib"]

#![feature(struct_variant)]
#![feature(phase)]

extern crate num;
extern crate sync;
#[phase(plugin, link)]
extern crate log;

pub use base::GenericNodeTable;
pub use base::GenericRpc;
pub use base::Node;
pub use service::Service;

#[unstable]
pub mod base;
#[unstable]
pub mod kademlia;
#[experimental]
pub mod service;
