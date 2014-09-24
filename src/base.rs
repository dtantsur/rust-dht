use std::io::net::ip::SocketAddr;
use std::sync::Future;
use num::BigUint;


/// Trait representing table with known nodes.
///
/// Keeps some reasonable subset of known nodes passed to `update`.
#[unstable]
pub trait GenericNodeTable : Send + Sync {
    /// Store or update node in the table.
    fn update(&mut self, node: &Node) -> bool;
    /// Find given number of node, closest to given ID.
    fn find(&self, id: &BigUint, count: uint) -> Vec<Node>;
}


/// Trait representing RPC implementation.
#[experimental]
pub trait GenericRpc : Send + Sync {
    /// Ping a node, returning true if node seems reachable.
    fn ping(&self, node: &Node) -> Future<bool>;
    /// Find a node with given ID.
    fn find_node(&self, id: &BigUint) -> Future<Node>;
}


/// Structure representing a node in system.
///
/// Every node has an address (IP and port) and a numeric ID, which is
/// used to calculate metrics and look up data.
#[deriving(Clone, Show)]
#[unstable]
pub struct Node {
    pub address: SocketAddr,
    pub id: BigUint
}
