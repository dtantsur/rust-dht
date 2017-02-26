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
use std::fmt::Debug;
use std::collections::VecDeque;

use super::GenericId;
use super::GenericNodeTable;
use super::Node;


// TODO(divius): make public?
static BUCKET_SIZE: usize = 32;
static DEFAULT_HASH_SIZE: usize = 64;


/// Kademlia node table.
///
/// Keeps nodes in a number of k-buckets (equal to bit size of ID in a system,
/// usually 160), where N-th k-bucket contains nodes with distance
/// from 2^N to 2^(N+1) from our node.
///
/// methods may panic if distance between two ids is greater than the
/// `hash_size`.
pub struct KNodeTable<TId, TAddr> {
    this_id: TId,
    hash_size: usize,
    // TODO(divius): convert to more appropriate data structure
    buckets: Vec<KBucket<TId, TAddr>>,
}

/// K-bucket - structure for keeping last nodes in Kademlia.
pub struct KBucket<TId, TAddr> {
    data: VecDeque<Node<TId, TAddr>>,
    size: usize,
}


impl<TId, TAddr> KNodeTable<TId, TAddr>
        where TId: GenericId,
              TAddr: Clone + Debug {
    /// Create a new node table.
    ///
    /// `this_id` -- ID of the current node (used to calculate metrics).
    pub fn new(this_id: TId) -> KNodeTable<TId, TAddr> {
        KNodeTable::new_with_details(this_id, BUCKET_SIZE, DEFAULT_HASH_SIZE)
    }

    pub fn new_with_details(this_id: TId, bucket_size: usize,
                    hash_size: usize) -> KNodeTable<TId, TAddr> {
        KNodeTable {
            this_id: this_id,
            hash_size: hash_size,
            buckets: (0..hash_size).map(
                              |_| KBucket::new(bucket_size)).collect(),
        }
    }

    pub fn buckets(&self) -> &Vec<KBucket<TId, TAddr>> {
        &self.buckets
    }

    #[inline]
    fn distance(id1: &TId, id2: &TId) -> TId {
        id1.bitxor(id2)
    }

    fn bucket_number(&self, id: &TId) -> usize {
        let diff = KNodeTable::<TId, TAddr>::distance(&self.this_id, id);
        debug_assert!(!diff.is_zero());
        let res = diff.bits() - 1;
        if res >= self.hash_size {
            panic!(format!("Distance between IDs {:?} and {:?} is {:?}, which is \
                    greater than the hash size ({:?})",
                    id, self.this_id, res, self.hash_size));
        }
        debug!("ID {:?} relative to own ID {:?} falls into bucket {:?}",
               id, self.this_id, res);
        res
    }
}

impl<TId, TAddr> GenericNodeTable<TId, TAddr> for KNodeTable<TId, TAddr>
        where TId: GenericId,
              TAddr: Clone + Debug + Sync + Send {
    fn random_id(&self) -> TId {
        TId::gen(self.hash_size)
    }

    fn update(&mut self, node: &Node<TId, TAddr>) -> bool {
        assert!(node.id != self.this_id);
        let bucket = self.bucket_number(&node.id);
        self.buckets[bucket].update(node)
    }

    fn find(&self, id: &TId, count: usize) -> Vec<Node<TId, TAddr>> {
        debug_assert!(count > 0);
        assert!(*id != self.this_id);

        let mut data_copy: Vec<_> = self.buckets.iter().flat_map(|b| &b.data).map(|n| n.clone()).collect();
        data_copy.sort_by_key(|n| KNodeTable::<TId, TAddr>::distance(id, &n.id));
        data_copy[0..cmp::min(count, data_copy.len())].to_vec()
    }

    fn pop_oldest(&mut self) -> Vec<Node<TId, TAddr>> {
        // For every full k-bucket, pop the last.
        // TODO(divius): TTL expiration?
        self.buckets.iter_mut()
            .filter(|b| { !b.data.is_empty() && b.size == b.data.len() })
            .map(|b| b.data.pop_front().unwrap())
            .collect()
    }
}

impl<TId, TAddr> KBucket<TId, TAddr>
        where TId: GenericId,
              TAddr: Clone + Debug {
    pub fn new(k: usize) -> KBucket<TId, TAddr> {
        assert!(k > 0);
        KBucket {
            data: VecDeque::new(),
            size: k
        }
    }

    pub fn update(&mut self, node: &Node<TId, TAddr>) -> bool {
        if self.data.iter().any(|x| x.id == node.id) {
            self.update_position(node.clone());
            debug!("Promoted node {:?} to the top of kbucket", node);
            true
        }
        else if self.data.len() == self.size {
            debug!("Not adding new node {:?} to kbucket - no space left", node);
            false
        }
        else {
            self.data.push_back(node.clone());
            debug!("Added new node {:?} to kbucket", node);
            true
        }
    }

    pub fn find(&self, id: &TId, count: usize) -> Vec<Node<TId, TAddr>> {
        let mut data_copy: Vec<_> = self.data.iter().map(|n| n.clone()).collect();
        data_copy.sort_by_key(|n| KNodeTable::<TId, TAddr>::distance(id, &n.id));
        data_copy[0..cmp::min(count, data_copy.len())].to_vec()
    }

    pub fn data(&self) -> &VecDeque<Node<TId, TAddr>> {
        &self.data
    }
    pub fn size(&self) -> usize {
        self.size
    }

    fn update_position(&mut self, node: Node<TId, TAddr>) {
        // TODO(divius): 1. optimize, 2. make it less ugly
        let mut new_data = VecDeque::with_capacity(self.data.len());
        new_data.extend(self.data.iter()
                        .filter(|x| x.id != node.id)
                        .map(|x| x.clone()));
        new_data.push_back(node.clone());
        self.data = new_data;
    }
}


#[cfg(test)]
mod test {
    use std::net;

    use super::super::GenericNodeTable;
    use super::super::Node;

    use super::DEFAULT_HASH_SIZE;
    use super::KBucket;
    use super::KNodeTable;

    use super::super::utils::test;
    type TestsIdType = test::IdType;
    use super::super::base::GenericId;


    fn prepare(count: u8) -> KBucket<TestsIdType, net::SocketAddr> {
        KBucket {
            data: (0..count).map(|i| test::new_node(test::make_id(i))).collect(),
            size: 3,
        }
    }

    fn assert_node_list_eq(expected: &[&Node<TestsIdType, net::SocketAddr>], actual: &Vec<Node<TestsIdType, net::SocketAddr>>) {
        let act: Vec<TestsIdType> = actual.iter()
            .map(|n| n.id.clone()).collect();
        let exp: Vec<TestsIdType> = expected.iter()
            .map(|n| n.id.clone()).collect();
        assert_eq!(exp, act);
    }

    #[test]
    fn test_nodetable_new() {
        let n = KNodeTable::<u64, ()>::new(42);
        assert_eq!(DEFAULT_HASH_SIZE, n.buckets.len());
    }

    #[test]
    fn test_nodetable_bucket_number() {
        let n = KNodeTable::<u64, ()>::new(42);
        let id = 41;
        // 42 xor 41 == 3
        assert_eq!(1, n.bucket_number(&id));
    }

    #[test]
    fn test_nodetable_pop_oldest() {
        let mut n = KNodeTable::<TestsIdType, net::SocketAddr>::new_with_details(
            test::make_id(42), 2, DEFAULT_HASH_SIZE);
        let mut lengths = vec![0; n.hash_size];

        n.update(&test::new_node(test::make_id(41)));
        n.update(&test::new_node(test::make_id(43)));
        n.update(&test::new_node(test::make_id(40)));
        lengths[0] = 1;
        lengths[1] = 2;
        assert_eq!(n.buckets().iter().map(|b| b.data.len()).collect::<Vec<_>>(), lengths);

        let nodes = n.pop_oldest();
        assert_eq!(1, nodes.len());
        assert_eq!(test::make_id(41), nodes[0].id);
        lengths[1] = 1;
        assert_eq!(n.buckets().iter().map(|b| b.data.len()).collect::<Vec<_>>(), lengths);
        assert_eq!(test::make_id(40), n.buckets[1].data[0].id);
    }

    #[test]
    fn test_nodetable_find() {
        let n = KNodeTable {
            buckets: vec![prepare(1), prepare(3), prepare(1)],
            this_id: test::make_id(0),
            hash_size: DEFAULT_HASH_SIZE,
        };
        // 0 xor 3 = 3, 1 xor 3 = 2, 2 xor 3 = 1
        let id = test::make_id(3);
        assert_node_list_eq(&[&n.buckets[1].data[2]],
                            &n.find(&id, 1));
    }

    #[test]
    #[should_panic(expected = "greater than the hash size")]
    fn test_nodetable_find_overflow() {
        let mut id1 = Vec::with_capacity(DEFAULT_HASH_SIZE/8);
        let mut id2 = Vec::with_capacity(DEFAULT_HASH_SIZE/8);
        id1.push(0);
        id2.push(255);
        for _ in 0..(DEFAULT_HASH_SIZE/8) {
            id1.push(0);
            id2.push(0);
        }
        let mut n = KNodeTable::new(id1);
        n.update(&test::new_node(id2));
    }

    #[test]
    fn test_nodetable_find_closest() {
        let mut n = KNodeTable::new(test::make_id(0b0000));
        let node1 = test::new_node(test::make_id(0b0101));
        let node2 = test::new_node(test::make_id(0b1010));
        let node3 = test::new_node(test::make_id(0b1110));
        assert!(n.update(&node1));
        assert!(n.update(&node2));
        assert!(n.update(&node3));
        assert_node_list_eq(&vec![&node3], &n.find(&test::make_id(0b1111), 1));
        assert_node_list_eq(&vec![&node2], &n.find(&test::make_id(0b1011), 1));
    }

    #[test]
    fn test_nodetable_update() {
        let mut n = KNodeTable::new_with_details(
            test::make_id(42), 1, DEFAULT_HASH_SIZE);
        let node = test::new_node(test::make_id(41));
        n.update(&node);
        assert_eq!(1, n.buckets[1].data.len());
        n.update(&node);
        assert_eq!(1, n.buckets[1].data.len());
    }

    #[test]
    fn test_nodetable_random_id() {
        let n = KNodeTable::<u64, ()>::new_with_details(
            42, 1, DEFAULT_HASH_SIZE);
        for _ in 0..100 {
            assert!(n.random_id().bits() <= DEFAULT_HASH_SIZE);
        }
        assert!(n.random_id() != n.random_id());
    }

    #[test]
    fn test_kbucket_new() {
        let b = KBucket::<TestsIdType, net::SocketAddr>::new(3);
        assert_eq!(0, b.data.len());
        assert_eq!(3, b.size);
    }

    #[test]
    fn test_kbucket_update_unknown() {
        let mut b = prepare(1);
        let node = test::new_node(test::make_id(42));
        assert!(b.update(&node));
        assert_eq!(2, b.data.len());
        assert_eq!(node.id, b.data[1].id);
    }

    #[test]
    fn test_kbucket_update_known() {
        let mut b = prepare(2);
        let node = test::new_node(test::make_id(0));
        assert!(b.update(&node));
        assert_eq!(2, b.data.len());
        assert_eq!(node.id, b.data[1].id);
    }

    #[test]
    fn test_kbucket_update_conflict() {
        let mut b = prepare(3);  // 3 is size
        let node = test::new_node(test::make_id(42));
        assert!(!b.update(&node))
    }

    #[test]
    fn test_kbucket_find() {
        let b = prepare(3);
        // Nodes with ID's 0, 1, 2; assume our ID is also 2 (impossible IRL)
        let id = test::make_id(2);
        // 0 xor 2 = 2, 1 xor 2 = 3, 2 xor 2 = 0
        assert_node_list_eq(&[&b.data[2]], &b.find(&id, 1));
        assert_node_list_eq(&[&b.data[2], &b.data[0]], &b.find(&id, 2));
    }

    #[test]
    fn test_kbucket_find_too_much() {
        let b = prepare(3);
        // Nodes with ID's 0, 1, 2; assume our ID is also 2 (impossible IRL)
        let id = test::make_id(2);
        // 0 xor 2 = 2, 1 xor 2 = 3, 2 xor 2 = 0
        assert_node_list_eq(&[&b.data[2], &b.data[0], &b.data[1]],
                            &b.find(&id, 100));
    }
}
