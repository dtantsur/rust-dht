#![feature(struct_variant)]
#![feature(phase)]

extern crate num;
extern crate sync;
#[phase(plugin, link)]
extern crate log;

pub use base::GenericNodeTable;
pub use base::HashId;
pub use base::Node;

pub mod base;
pub mod kademlia;
