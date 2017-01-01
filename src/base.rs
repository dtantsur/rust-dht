// Copyright 2014 Dmitry "Divius" Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

use rand;
use rand::Rng;

use std::hash::Hash;
use std::fmt::Debug;
use std::str::FromStr;
use std::net;

use rustc_serialize as serialize;
use rustc_serialize::hex::ToHex;
use rustc_serialize::hex::FromHex;

/// Generalization of num::BigUint, with hexadecimal encoding and decoding
pub trait GenericId : Hash + PartialEq + Eq + Ord + Clone + Send + Sync + Debug {
    fn bitxor(&self, other: &Self) -> Self;
    fn is_zero(&self) -> bool;
    fn bits(&self) -> usize;
    /// num::bigint::RandBigInt::gen_biguint
    fn gen(bit_size: usize) -> Self;

    fn encode<S:serialize::Encoder> (&self, s: &mut S) -> Result<(), S::Error>;
    fn decode<D:serialize::Decoder> (d : &mut D) -> Result<Self, D::Error>;
}

impl GenericId for u64 {
    fn bitxor(&self, other: &u64) -> u64 {
        self ^ other
    }
    fn is_zero(&self) -> bool {
        *self == 0
    }
    fn bits(&self) -> usize {
        (64 - self.leading_zeros()) as usize
    }
    fn gen(bit_size: usize) -> u64 {
        assert!(bit_size <= 64);
        if bit_size == 64 {
            rand::thread_rng().next_u64()
        }
        else {
            rand::thread_rng().gen_range(0, 1 << bit_size)
        }
    }

    fn encode<S:serialize::Encoder> (&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_str(&format!("{:x}", self))
    }
    fn decode<D:serialize::Decoder> (d : &mut D) -> Result<u64, D::Error> {
        let s: &str = &try!(d.read_str());
        match u64::from_str_radix(s, 16) {
            Ok(v) => Ok(v),
            Err(e) => {
                let err = format!("Expected hex-encoded ID, got {}, error {:?}", s, e);
                Err(d.error(&err))
            }
        }
    }
}

impl GenericId for Vec<u8> {
    fn bitxor(&self, other: &Vec<u8>) -> Vec<u8> {
        self.iter().zip(other.iter()).map(|(digit1, digit2)| digit1 ^ digit2).collect()
    }
    fn is_zero(&self) -> bool {
        self.iter().all(|digit| *digit == 0)
    }
    fn bits(&self) -> usize {
        let mut bits = self.len()*8;
        for digit in self {
            if *digit == 0 {
                bits -= 8;
            }
            else {
                return bits - digit.leading_zeros() as usize
            }
        }
        assert!(bits == 0);
        0
    }
    fn gen(bit_size: usize) -> Vec<u8> {
        let nb_full_digits = bit_size/8;
        let nb_bits_partial_digit = bit_size%8;
        let mut rng = rand::thread_rng();
        if nb_bits_partial_digit == 0 {
            let mut res = vec![0u8; nb_full_digits];
            rng.fill_bytes(&mut res);
            res
        }
        else {
            let mut res = vec![0u8; nb_full_digits+1];
            let first_digit = rng.gen_range(0, 1<<(nb_bits_partial_digit-1));
            res[0] = first_digit;
            rng.fill_bytes(&mut res[1..nb_full_digits+1]);
            res
        }
    }

    fn encode<S:serialize::Encoder> (&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_str(&self.to_hex())
    }
    fn decode<D:serialize::Decoder> (d : &mut D) -> Result<Vec<u8>, D::Error> {
        let s = try!(d.read_str());
        match s.from_hex() {
            Ok(v) => Ok(v),
            Err(e) => {
                let err = format!("Expected hex-encoded ID, got {}, error {:?}", s, e);
                Err(d.error(&err))
            }
        }
    }
}

/// Trait representing table with known nodes.
///
/// Keeps some reasonable subset of known nodes passed to `update`.
pub trait GenericNodeTable<TId, TAddr> : Send + Sync
        where TId: GenericId {
    /// Generate suitable random ID.
    fn random_id(&self) -> TId;
    /// Store or update node in the table.
    fn update(&mut self, node: &Node<TId, TAddr>) -> bool;
    /// Find given number of node, closest to given ID.
    fn find(&self, id: &TId, count: usize) -> Vec<Node<TId, TAddr>>;
    /// Pop expired or the oldest nodes from table for inspection.
    fn pop_oldest(&mut self) -> Vec<Node<TId, TAddr>>;
}

/// Structure representing a node in system.
///
/// Every node has an address (IP and port) and a numeric ID, which is
/// used to calculate metrics and look up data.
#[derive(Clone, Debug)]
pub struct Node<TId, TAddr> {
    /// Network address of the node.
    pub address: TAddr,
    /// ID of the node.
    pub id: TId
}

/// Trait representing the API.
pub trait GenericAPI<TId, TAddr>
        where TId: GenericId {
    /// Value type.
    type TValue: Send + Sync + Clone;
    /// Ping a node.
    fn ping<F>(&mut self, node: &Node<TId, TAddr>, callback: F)
        where F: FnOnce(&Node<TId, TAddr>, bool);
    /// Return nodes clothest to the given id.
    fn find_node<F>(&mut self, id: &TId, callback: F)
        where F: FnOnce(Vec<Node<TId, TAddr>>);
    /// Find a value in the network.
    ///
    /// Either returns a value or several clothest nodes.
    fn find_value<F>(&mut self, id: &TId, callback: F)
        where F: FnOnce(Option<Self::TValue>, Vec<Node<TId, TAddr>>);
    /// Store a value on a node.
    fn store(&mut self, node: &Node<TId, TAddr>, id: &TId, value: Self::TValue);
}

impl<TId> serialize::Encodable for Node<TId, net::SocketAddr>
        where TId: GenericId {
    fn encode<S:serialize::Encoder> (&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("Node", 2, |s| {
            try!(s.emit_struct_field("address", 0, |s2| {
                let addr = format!("{}", self.address);
                addr.encode(s2)
            }));

            try!(s.emit_struct_field("id", 1, |s2| self.id.encode(s2)));

            Ok(())
        })
    }
}

impl<TId> serialize::Decodable for Node<TId, net::SocketAddr>
        where TId: GenericId {
    fn decode<D:serialize::Decoder> (d : &mut D) -> Result<Node<TId, net::SocketAddr>, D::Error> {
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

            let id = try!(d.read_struct_field("id", 1, TId::decode));

            Ok(Node { address: addr, id: id })
        })
    }
}

#[cfg(test)]
mod test {
    use std::net;
    use rustc_serialize::json;

    use super::{GenericAPI, Node};

    use super::super::utils::test;
    type TestsIdType = test::IdType;

    #[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
    struct SimplifiedNode {
        address: String,
        id: String
    }

    struct DummyAPI {
        value: Option<i32>
    }

    impl GenericAPI<TestsIdType, net::SocketAddr> for DummyAPI {
        type TValue = i32;
        fn ping<F>(&mut self, node: &Node<TestsIdType, net::SocketAddr>, callback: F)
                where F: FnOnce(&Node<TestsIdType, net::SocketAddr>, bool) {
            callback(node, true);
        }
        fn find_node<F>(&mut self, _id: &TestsIdType, callback: F)
                where F: FnOnce(Vec<Node<TestsIdType, net::SocketAddr>>) {
            callback(vec![]);
        }
        fn find_value<F>(&mut self, _id: &TestsIdType, callback: F)
                where F: FnOnce(Option<Self::TValue>, Vec<Node<TestsIdType, net::SocketAddr>>) {
            callback(self.value, vec![]);
        }
        fn store(&mut self, _node: &Node<TestsIdType, net::SocketAddr>, _id: &TestsIdType, value: Self::TValue) {
            self.value = Some(value);
        }
    }

    #[test]
    fn test_node_encode() {
        let n = test::new_node(test::make_id(42));
        let j = json::encode(&n);
        let m: SimplifiedNode = json::decode(&j.unwrap()).unwrap();
        assert_eq!(test::ADDR, &m.address);
        assert_eq!("2a", &m.id);
    }

    #[test]
    fn test_node_decode() {
        let sn = SimplifiedNode {
            address: "127.0.0.1:80".to_string(),
            id: "2a".to_string()
        };
        let j = json::encode(&sn);
        let n: Node<TestsIdType, net::SocketAddr> = json::decode(&j.unwrap()).unwrap();
        assert_eq!(test::make_id(42), n.id);
    }

    #[test]
    fn test_node_decode_bad_address() {
        let sn = SimplifiedNode {
            address: "127.0.0.1".to_string(),
            id: "2a".to_string()
        };
        let j = json::encode(&sn);
        assert!(json::decode::<Node<TestsIdType, net::SocketAddr>>(&j.unwrap()).is_err());
    }

    #[test]
    fn test_node_decode_bad_id() {
        let sn = SimplifiedNode {
            address: "127.0.0.1:80".to_string(),
            id: "x42".to_string()
        };
        let j = json::encode(&sn);
        assert!(json::decode::<Node<TestsIdType, net::SocketAddr>>(&j.unwrap()).is_err());
    }

    #[test]
    fn test_node_encode_decode() {
        let n = test::new_node(test::make_id(42));
        let j = json::encode(&n);
        let n2 = json::decode::<Node<TestsIdType, net::SocketAddr>>(&j.unwrap()).unwrap();
        assert_eq!(n.id, n2.id);
        assert_eq!(n.address, n2.address);
    }

    #[test]
    fn test_generic_api() {
        let mut api = DummyAPI { value: None };
        let n = test::new_node(test::make_id(42));
        api.ping(&n, |_node, res| {
            assert!(res);
        });
    }
}
