// Copyright 2014 Dmitry "Divius" Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

//! DHT node table implementation based on Kademlia.
//!
//! See [original paper](http://pdos.csail.mit.edu/%7Epetar/papers/maymounkov-kademlia-lncs.pdf)
//! for details. The most essential difference is that when k-bucket is full,
//! no RPC call is done. It is up to upper-level code to ensure proper clean up
//! using `pop_oldest` call.

use std::cmp;
use num::Zero;

use num;
use std::num::ToPrimitive;
use std::ops::BitXor;

use super::GenericNodeTable;
use super::base::{Peer};
use std::old_io::{Timer};
use std::time::Duration;


// TODO(divius): make public?
static BUCKET_SIZE: usize = 64;
static HASH_SIZE: usize = 160;


/// Kademlia node table.
///
/// Keeps nodes in a number of k-buckets (equal to bit size of ID in a system,
/// usually 160), where N-th k-bucket contains nodes with distance
/// from 2^N to 2^(N+1) from our node.
#[unstable]
pub struct KNodeTable<P : Peer> {
    this_id: P::ID,
    hash_size: usize,
    // TODO(divius): convert to more appropriate data structure
    buckets: Vec<KBucket<P>>,
}

/// K-bucket - structure for keeping last nodes in Kademlia.
struct KBucket<P : Peer> {
    data: Vec<P>,
    size: usize,
}


#[unstable]
impl<P : Peer> KNodeTable<P> {
    /// Create a new node table.
    ///
    /// `this_id` -- ID of the current node (used to calculate metrics).
    pub fn new(this_id: P::ID) -> KNodeTable<P> {
        KNodeTable::with_details(this_id, BUCKET_SIZE, HASH_SIZE)
    }

    // TODO(divius): make public?
    fn with_details(this_id: P::ID, bucket_size: usize,
                    hash_size: usize) -> KNodeTable<P> {
        KNodeTable {
            this_id: this_id,
            hash_size: hash_size,
            buckets: (0..hash_size).map(
                              |_| KBucket::new(bucket_size)).collect(),
        }
    }

    #[inline]
    fn distance (id1: &P::ID, id2: &P::ID) -> num::BigUint {
       <P as Peer>::key_as_buint(id1).bitxor(<P as Peer>::key_as_buint(id2))
    }


    fn bucket_number(&self, id: &P::ID) -> usize {
        let diff = KNodeTable::<P>::distance(&self.this_id, id);
        debug_assert!(!diff.is_zero());
        let res = diff.bits() - 1;
        debug!("ID {:?} relative to own ID {:?} falls into bucket {:?}",
               id, self.this_id, res);
        res
    }


    #[inline]
    pub fn update(&mut self, node: &P) -> bool {
        assert!(*node.get_id() != self.this_id);
        let bucket = self.bucket_number(node.get_id());
        self.buckets[bucket].update(node)
    }

    pub fn find(&self, id: &P::ID, count: usize) -> Vec<P> {
        debug_assert!(count > 0);
        assert!(*id != self.this_id);
        let bucket = self.bucket_number(id);
        self.buckets[bucket].find(id, count)
    }

    fn pop_oldest(&mut self) -> Vec<P> {
        // For every full k-bucket, pop the last.
        // TODO(divius): TTL expiration?
        self.buckets.iter_mut()
            .filter(|b| { !b.data.is_empty() && b.size == b.data.len() })
            .map(|b| b.data.remove(0))
            .collect()
    }

    pub fn remove(&mut self, id: &P::ID) -> bool {
        let bucket = self.bucket_number(id);
        self.buckets[bucket].remove(id)
    }


}

#[unstable]
impl<P : Peer> GenericNodeTable for KNodeTable<P> {
    type P = P;
    
    fn random_id(&self) -> P::ID {
        <P as Peer>::random_id(self.hash_size)
    }

    fn update(&mut self, node: &P) -> bool {
        self.update(node)
    }

    fn find(&self, id: &P::ID, count: usize) -> Vec<P> {
        self.find(id, count)
    }

    fn pop_oldest(&mut self) -> Vec<P> {
        self.pop_oldest()
    }
}

impl<P : Peer> KBucket<P> {


    pub fn new(k: usize) -> KBucket<P> {
        assert!(k > 0);
        KBucket {
            data: Vec::new(),
            size: k
        }
    }

    pub fn update(&mut self, node: &P) -> bool {
        if self.data.iter().any(|x| x.get_id() == node.get_id()) {
            self.update_position(node.clone());
            debug!("Promoted node {:?} to the top of kbucket", node);
            true
        }
        else if self.data.len() == self.size {
            debug!("Not adding new node {:?} to kbucket - no space left", node);
            false
        }
        else {
            self.data.push(node.clone());
            debug!("Added new node {:?} to kbucket", node);
            true
        }
    }

    pub fn find(&self, id: &P::ID, count: usize) -> Vec<P> {
/*        let sort_fn = |&: a: &P, b: &P| {
            KNodeTable::<P>::distance(id, a.get_id())
                .cmp(
            &KNodeTable::<P>::distance(id, b.get_id())
                )
        };
        let mut data_copy = self.data.clone();
        data_copy.sort_by(sort_fn);
        data_copy.slice(0, cmp::min(count, data_copy.len())).to_vec()*/
        self.data.slice(0, cmp::min(count, self.data.len())).to_vec()
    }

    fn update_position(&mut self, node: P) {
        // TODO(divius): 1. optimize, 2. make it less ugly
/*        let mut new_data = Vec::with_capacity(self.data.len());
        new_data.extend(self.data.iter()
                        .filter(|x| x.get_id() != node.get_id())
                        .map(|x| x.clone()));
        new_data.push(node.clone());
        self.data = new_data;
        */
        self.remove(node.get_id());
        self.data.push(node.clone());
    }

    pub fn remove(&mut self, id: &P::ID) -> bool {
        let oix = self.data.iter().position(|x| x.get_id() == id);
        match oix {
            Some(ix) => {
                // slow consider swap remove even if not kad compatible
                self.data.remove(ix);
                true
            },
            None => false,
        }
    }

}


#[cfg(test)]
mod test {
    use num;
    use std::num::ToPrimitive;

    use super::super::GenericNodeTable;
    use super::super::Node;

    use super::HASH_SIZE;
    use super::KBucket;
    use super::KNodeTable;

    use super::super::utils::test;

    fn prepare(count: usize) -> KBucket<Node> {
        KBucket {
            data: (0..count).map(|i| test::new_node(i)).collect(),
            size: 3,
        }
    }

    fn assert_node_list_eq(expected: &[&Node], actual: &Vec<Node>) {
        let act: Vec<num::BigUint> = actual.iter()
            .map(|n| n.id.clone()).collect();
        let exp: Vec<num::BigUint> = expected.iter()
            .map(|n| n.id.clone()).collect();
        assert_eq!(exp, act);
    }

    #[test]
    fn test_nodetable_new() {
        let n = KNodeTable::<Node>::new(test::usize_to_id(42));
        assert_eq!(HASH_SIZE, n.buckets.len());
    }

    #[test]
    fn test_nodetable_bucket_number() {
        let n = KNodeTable::<Node>::new(test::usize_to_id(42));
        let id = test::usize_to_id(41);
        // 42 xor 41 == 3
        assert_eq!(1, n.bucket_number(&id));
    }

    #[test]
    fn test_nodetable_pop_oldest() {
        let mut n = KNodeTable::with_details(
            test::usize_to_id(42), 2, HASH_SIZE);
        n.update(&test::new_node(41));
        n.update(&test::new_node(43));
        n.update(&test::new_node(40));
        assert_eq!(0, n.buckets[2].data.len());
        assert_eq!(2, n.buckets[1].data.len());
        assert_eq!(1, n.buckets[0].data.len());
        let nodes = n.pop_oldest();
        assert_eq!(1, nodes.len());
        assert_eq!(41, nodes[0].id.to_int().unwrap());
        assert_eq!(0, n.buckets[2].data.len());
        assert_eq!(1, n.buckets[1].data.len());
        assert_eq!(1, n.buckets[0].data.len());
        assert_eq!(40, n.buckets[1].data[0].id.to_int().unwrap());
    }

    #[test]
    fn test_nodetable_find() {
        let n = KNodeTable {
            buckets: vec![prepare(1), prepare(3), prepare(1)],
            this_id: test::usize_to_id(0),
            hash_size: HASH_SIZE,
        };
        // 0 xor 3 = 3, 1 xor 3 = 2, 2 xor 3 = 1
        let id = test::usize_to_id(3);
        assert_node_list_eq(&[&n.buckets[1].data[2]],
                            &n.find(&id, 1));
    }

    #[test]
    fn test_nodetable_update() {
        let mut n = KNodeTable::with_details(
            test::usize_to_id(42), 1, HASH_SIZE);
        let node = test::new_node(41);
        n.update(&node);
        assert_eq!(1, n.buckets[1].data.len());
        n.update(&node);
        assert_eq!(1, n.buckets[1].data.len());
    }

    #[test]
    fn test_nodetable_random_id() {
        let n = KNodeTable::<Node>::with_details(
            test::usize_to_id(42), 1, HASH_SIZE);
        for _ in range(0us, 100us) {
            assert!(n.random_id().bits() <= HASH_SIZE);
        }
        assert!(n.random_id() != n.random_id());
    }

    #[test]
    fn test_kbucket_new() {
        let b = KBucket::<Node>::new(3);
        assert_eq!(0, b.data.len());
        assert_eq!(3, b.size);
    }

    #[test]
    fn test_kbucket_update_unknown() {
        let mut b = prepare(1);
        let node = test::new_node(42);
        assert!(b.update(&node));
        assert_eq!(2, b.data.len());
        assert_eq!(node.id, b.data[1].id);
    }

    #[test]
    fn test_kbucket_update_known() {
        let mut b = prepare(2);
        let node = test::new_node(0);
        assert!(b.update(&node));
        assert_eq!(2, b.data.len());
        assert_eq!(node.id, b.data[1].id);
    }

    #[test]
    fn test_kbucket_update_conflict() {
        let mut b = prepare(3);  // 3 is size
        let node = test::new_node(42);
        assert!(!b.update(&node))
    }

    #[test]
    fn test_kbucket_find() {
        let b = prepare(3);
        // Nodes with ID's 0, 1, 2; assume our ID is also 2 (impossible IRL)
        let id = test::usize_to_id(2);
        // 0 xor 2 = 2, 1 xor 2 = 3, 2 xor 2 = 0
        assert_node_list_eq(&[&b.data[2]], &b.find(&id, 1));
        assert_node_list_eq(&[&b.data[2], &b.data[0]], &b.find(&id, 2));
    }

    #[test]
    fn test_kbucket_find_too_much() {
        let b = prepare(3);
        // Nodes with ID's 0, 1, 2; assume our ID is also 2 (impossible IRL)
        let id = test::usize_to_id(2);
        // 0 xor 2 = 2, 1 xor 2 = 3, 2 xor 2 = 0
        assert_node_list_eq(&[&b.data[2], &b.data[0], &b.data[1]],
                            &b.find(&id, 100));
    }
}
