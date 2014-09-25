use std::from_str::FromStr;
use std::io::net::ip::SocketAddr;
use std::sync::Future;
use num::BigUint;
use serialize::{Decodable, Decoder};
use serialize::{Encodable, Encoder};


/// Trait representing table with known nodes.
///
/// Keeps some reasonable subset of known nodes passed to `update`.
#[unstable]
pub trait GenericNodeTable : Send + Sync {
    /// Store or update node in the table.
    fn update(&mut self, node: &Node) -> bool;
    /// Find given number of node, closest to given ID.
    fn find(&self, id: &BigUint, count: uint) -> Vec<Node>;
    /// Pop expired or the oldest nodes from table for inspection.
    fn pop_oldest(&mut self) -> Vec<Node>;
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
    /// Network address of the node.
    pub address: SocketAddr,
    /// ID of the node.
    pub id: BigUint
}


impl<E, S:Encoder<E>> Encodable<S, E> for Node {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_struct("Node", 2, |s| {
            try!(s.emit_struct_field("address", 0, |s2| {
                let addr = format!("{}", self.address);
                addr.encode(s2)
            }));

            try!(s.emit_struct_field("id", 1, |s2| {
                let id = format!("{}", self.id);
                id.encode(s2)
            }));

            Ok(())
        })
    }
}

impl<E, D:Decoder<E>> Decodable<D, E> for Node {
    fn decode(d: &mut D) -> Result<Node, E> {
        d.read_struct("Node", 2, |d| {
            let addr = try!(d.read_struct_field("address", 0, |d2| {
                let s = try!(d2.read_str());
                match FromStr::from_str(s.as_slice()) {
                    Some(addr) => Ok(addr),
                    None => {
                        let err = format!("Expected socket address, got {}", s);
                        Err(d2.error(err.as_slice()))
                    }
                }
            }));

            let id = try!(d.read_struct_field("id", 1, |d2| {
                let s = try!(d2.read_str());
                match FromStr::from_str(s.as_slice()) {
                    Some(id) => Ok(id),
                    None => {
                        let err = format!("Expected ID, got {}", s);
                        Err(d2.error(err.as_slice()))
                    }
                }
            }));

            Ok(Node { address: addr, id: id })
        })
    }
}


#[cfg(test)]
mod test {
    use std::from_str::FromStr;
    use std::num::FromPrimitive;
    use serialize::{json, Encodable};
    use super::Node;

    static ADDR: &'static str = "127.0.0.1:80";

    fn new_node(id: uint) -> Node {
        Node {
            id: FromPrimitive::from_uint(id).unwrap(),
            address: FromStr::from_str(ADDR).unwrap()
        }
    }

    #[deriving(Show, Clone, Encodable, Decodable)]
    struct SimplifiedNode {
        address: String,
        id: String
    }

    #[test]
    fn test_node_encode() {
        let n = new_node(42);
        let j = json::encode(&n);
        let m: SimplifiedNode = json::decode(j.as_slice()).unwrap();
        assert_eq!(ADDR, m.address.as_slice());
        assert_eq!("42", m.id.as_slice());
    }

    #[test]
    fn test_node_decode() {
        let sn = SimplifiedNode {
            address: "127.0.0.1:80".to_string(),
            id: "42".to_string()
        };
        let j = json::encode(&sn);
        let n: Node = json::decode(j.as_slice()).unwrap();
        assert_eq!(42, n.id.to_uint().unwrap());
    }

    #[test]
    fn test_node_decode_bad_address() {
        let sn = SimplifiedNode {
            address: "127.0.0.1".to_string(),
            id: "42".to_string()
        };
        let j = json::encode(&sn);
        assert!(json::decode::<Node>(j.as_slice()).is_err());
    }

    #[test]
    fn test_node_decode_bad_id() {
        let sn = SimplifiedNode {
            address: "127.0.0.1:80".to_string(),
            id: "x42".to_string()
        };
        let j = json::encode(&sn);
        assert!(json::decode::<Node>(j.as_slice()).is_err());
    }

    #[test]
    fn test_node_encode_decode() {
        let n = new_node(42);
        let j = json::encode(&n);
        let n2 = json::decode::<Node>(j.as_slice()).unwrap();
        assert_eq!(n.id, n2.id);
        assert_eq!(n.address, n2.address);
    }
}
