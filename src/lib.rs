#![feature(struct_variant)]
#![feature(phase)]

extern crate num;
extern crate sync;
#[phase(plugin, link)]
extern crate log;

pub use hashid::HashId;
pub use node::Node;

pub mod kademlia;

mod hashid;
mod node;
