// Copyright 2014 Dmitry "Divius" Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

use std::str::FromStr;
use std::old_io::net::ip;
use std::old_io::IoResult;
use std::ops::BitXor;
use std::fmt;
use rand;
use rand::{ThreadRng,Rng};
use num::BigUint;
use num::bigint::RandBigInt;

use num;
use rustc_serialize as serialize;


/// Trait representing table with known nodes.
///
/// Keeps some reasonable subset of known nodes passed to `update`.
pub trait GenericNodeTable : Send + Sync + 'static {
    type P : Peer + 'static;
    /// Generate suitable random ID.
//    #[experimental]
    fn random_id(&self) -> <Self::P as Peer>::Id;
    /// Store or update node in the table.
    fn update(&mut self, node: &Self::P) -> bool;
    /// Find given number of node, closest to given ID.
    fn find(&self, id: &<Self::P as Peer>::Id, count: usize) -> Vec<Self::P>;
    /// Pop expired or the oldest nodes from table for inspection.
    fn pop_oldest(&mut self) -> Vec<Self::P>;
    /// Remove a given node from table.
    fn remove(&mut self, id: &<Self::P as Peer>::Id) -> bool;


}

/// Structure representing a node in system.
///
/// Every node has an address (IP and port) and a numeric ID, which is
/// used to calculate metrics and look up data.
#[derive(Clone, Debug)]
pub struct Node {
    /// Network address of the node.
    pub address: ip::SocketAddr,
    /// ID of the node.
    pub id: num::BigUint,
}

pub trait Peer :  Send + Sync + fmt::Debug + Clone + 'static {
    type Id : Eq + Send + Sync + fmt::Debug + 'static;
    fn id<'a> (&self) -> &Self::Id; // TODO change it to return &Key (lot of useless clone in impls
    fn key_as_buint<'a>(&'a Self::Id) -> &'a num::BigUint;
    fn random_id(usize) -> Self::Id; // TODO usize in peer trait
}

impl ip::ToSocketAddr for Node {
    #[inline]
    fn to_socket_addr(&self) -> IoResult<ip::SocketAddr>{
        Ok(self.address)
    }
}

impl Peer for Node {
    type Id = num::BigUint;
    #[inline]
    fn id<'a> (&'a self) -> &'a num::BigUint {
        &self.id
    }
    #[inline]
    fn key_as_buint<'a>(k : &'a num::BigUint) -> &'a num::BigUint {
        k
    }
    #[inline]
    fn random_id(hash_size : usize) -> num::BigUint {
        let mut rng = rand::thread_rng();
        rng.gen_biguint(hash_size)
    }
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
                match FromStr::from_str(s.as_slice()) {
                    Ok(addr) => Ok(addr),
                    Err(e) => {
                        let err = format!("Expected socket address, got {}, error {:?}", s, e);
                        Err(d2.error(err.as_slice()))
                    }
                }
            }));

            let id = try!(d.read_struct_field("id", 1, |d2| {
                let s = try!(d2.read_str());
                match FromStr::from_str(s.as_slice()) {
                    Ok(id) => Ok(id),
                    Err(e) => {
                        let err = format!("Expected ID, got {}, error {:?}", s, e);
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
    use rustc_serialize::json;
    use std::num::ToPrimitive;

    use super::Node;

    use super::super::utils::test;


    #[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
    struct SimplifiedNode {
        address: String,
        id: String
    }

    #[test]
    fn test_node_encode() {
        let n = test::new_node(42);
        let j = json::encode(&n);
        let m: SimplifiedNode = json::decode(j.unwrap().as_slice()).unwrap();
        assert_eq!(test::ADDR, m.address.as_slice());
        assert_eq!("42", m.id.as_slice());
    }

    #[test]
    fn test_node_decode() {
        let sn = SimplifiedNode {
            address: "127.0.0.1:80".to_string(),
            id: "42".to_string()
        };
        let j = json::encode(&sn);
        let n: Node = json::decode(j.unwrap().as_slice()).unwrap();
        assert_eq!(42, n.id.to_uint().unwrap());
    }

    #[test]
    fn test_node_decode_bad_address() {
        let sn = SimplifiedNode {
            address: "127.0.0.1".to_string(),
            id: "42".to_string()
        };
        let j = json::encode(&sn);
        assert!(json::decode::<Node>(j.unwrap().as_slice()).is_err());
    }

    #[test]
    fn test_node_decode_bad_id() {
        let sn = SimplifiedNode {
            address: "127.0.0.1:80".to_string(),
            id: "x42".to_string()
        };
        let j = json::encode(&sn);
        assert!(json::decode::<Node>(j.unwrap().as_slice()).is_err());
    }

    #[test]
    fn test_node_encode_decode() {
        let n = test::new_node(42);
        let j = json::encode(&n);
        let n2 = json::decode::<Node>(j.unwrap().as_slice()).unwrap();
        assert_eq!(n.id, n2.id);
        assert_eq!(n.address, n2.address);
    }
}
