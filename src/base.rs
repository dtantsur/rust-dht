// Copyright 2014 Dmitry "Divius" Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

use std::str::FromStr;
use std::net;

use num;
use rustc_serialize as serialize;


/// Trait representing table with known nodes.
///
/// Keeps some reasonable subset of known nodes passed to `update`.
pub trait GenericNodeTable : Send + Sync {
    /// Generate suitable random ID.
    fn random_id(&self) -> num::BigUint;
    /// Store or update node in the table.
    fn update(&mut self, node: &Node) -> bool;
    /// Find given number of node, closest to given ID.
    fn find(&self, id: &num::BigUint, count: usize) -> Vec<Node>;
    /// Pop expired or the oldest nodes from table for inspection.
    fn pop_oldest(&mut self) -> Vec<Node>;
}

/// Structure representing a node in system.
///
/// Every node has an address (IP and port) and a numeric ID, which is
/// used to calculate metrics and look up data.
#[derive(Clone, Debug)]
pub struct Node {
    /// Network address of the node.
    pub address: net::SocketAddr,
    /// ID of the node.
    pub id: num::BigUint
}

/// Trait representing the API.
pub trait GenericAPI {
    /// Value type.
    type TValue: Send + Sync + Clone;
    /// Ping a node.
    fn ping<F>(&mut self, node: &Node, callback: F)
        where F: FnOnce(&Node, bool);
    /// Return nodes clothest to the given id.
    fn find_node<F>(&mut self, id: &num::BigUint, callback: F)
        where F: FnOnce(Vec<Node>);
    /// Find a value in the network.
    ///
    /// Either returns a value or several clothest nodes.
    fn find_value<F>(&mut self, id: &num::BigUint, callback: F)
        where F: FnOnce(Option<Self::TValue>, Vec<Node>);
    /// Store a value on a node.
    fn store(&mut self, node: &Node, id: &num::BigUint, value: Self::TValue);
}


impl serialize::Encodable for Node {
    fn encode<S:serialize::Encoder> (&self, s: &mut S) -> Result<(), S::Error> {
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

impl serialize::Decodable for Node {
    fn decode<D:serialize::Decoder> (d : &mut D) -> Result<Node, D::Error> {
        d.read_struct("Node", 2, |d| {
            let addr = try!(d.read_struct_field("address", 0, |d2| {
                let s = try!(d2.read_str());
                match FromStr::from_str(&s) {
                    Ok(addr) => Ok(addr),
                    Err(e) => {
                        let err = format!("Expected socket address, got {}, error {:?}", s, e);
                        Err(d2.error(&err))
                    }
                }
            }));

            let id = try!(d.read_struct_field("id", 1, |d2| {
                let s = try!(d2.read_str());
                match FromStr::from_str(&s) {
                    Ok(id) => Ok(id),
                    Err(e) => {
                        let err = format!("Expected ID, got {}, error {:?}", s, e);
                        Err(d2.error(&err))
                    }
                }
            }));

            Ok(Node { address: addr, id: id })
        })
    }
}

#[cfg(test)]
mod test {
    use num::{self, ToPrimitive};
    use rustc_serialize::json;

    use super::{GenericAPI, Node};

    use super::super::utils::test;


    #[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
    struct SimplifiedNode {
        address: String,
        id: String
    }

    struct DummyAPI {
        value: Option<i32>
    }

    impl GenericAPI for DummyAPI {
        type TValue = i32;
        fn ping<F>(&mut self, node: &Node, callback: F)
                where F: FnOnce(&Node, bool) {
            callback(node, true);
        }
        fn find_node<F>(&mut self, id: &num::BigUint, callback: F)
                where F: FnOnce(Vec<Node>) {
            callback(vec![]);
        }
        fn find_value<F>(&mut self, id: &num::BigUint, callback: F)
                where F: FnOnce(Option<Self::TValue>, Vec<Node>) {
            callback(self.value, vec![]);
        }
        fn store(&mut self, node: &Node, id: &num::BigUint, value: Self::TValue) {
            self.value = Some(value);
        }
    }

    #[test]
    fn test_node_encode() {
        let n = test::new_node(42);
        let j = json::encode(&n);
        let m: SimplifiedNode = json::decode(&j.unwrap()).unwrap();
        assert_eq!(test::ADDR, &m.address);
        assert_eq!("42", &m.id);
    }

    #[test]
    fn test_node_decode() {
        let sn = SimplifiedNode {
            address: "127.0.0.1:80".to_string(),
            id: "42".to_string()
        };
        let j = json::encode(&sn);
        let n: Node = json::decode(&j.unwrap()).unwrap();
        assert_eq!(42, n.id.to_u8().unwrap());
    }

    #[test]
    fn test_node_decode_bad_address() {
        let sn = SimplifiedNode {
            address: "127.0.0.1".to_string(),
            id: "42".to_string()
        };
        let j = json::encode(&sn);
        assert!(json::decode::<Node>(&j.unwrap()).is_err());
    }

    #[test]
    fn test_node_decode_bad_id() {
        let sn = SimplifiedNode {
            address: "127.0.0.1:80".to_string(),
            id: "x42".to_string()
        };
        let j = json::encode(&sn);
        assert!(json::decode::<Node>(&j.unwrap()).is_err());
    }

    #[test]
    fn test_node_encode_decode() {
        let n = test::new_node(42);
        let j = json::encode(&n);
        let n2 = json::decode::<Node>(&j.unwrap()).unwrap();
        assert_eq!(n.id, n2.id);
        assert_eq!(n.address, n2.address);
    }

    #[test]
    fn test_generic_api() {
        let mut api = DummyAPI { value: None };
        let n = test::new_node(42);
        api.ping(&n, |node, res| {
            assert!(res);
        });
    }
}
