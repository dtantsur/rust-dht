use std::io::net::ip::SocketAddr;
use num::BigUint;


/// Trait representing table with known nodes.
///
/// Keeps some reasonable subset of known nodes passed to `update`.
pub trait GenericNodeTable {
    /// Store or update node in the table.
    fn update(&mut self, node: &Node) -> bool;
    /// Find given number of node, closest to given ID.
    fn find(&self, id: &BigUint, count: uint) -> Vec<Node>;
}


/// Structure representing a node in system.
///
/// Every node has an address (IP and port) and a numeric ID, which is
/// used to calculate metrics and look up data.
#[deriving(Clone, Show)]
pub struct Node {
    pub address: SocketAddr,
    pub id: BigUint
}
