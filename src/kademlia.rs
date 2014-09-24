//! DHT implementation based on Kademlia.
//!
//! See [original paper](http://pdos.csail.mit.edu/%7Epetar/papers/maymounkov-kademlia-lncs.pdf)
//! for details. The most essential difference is that when k-bucket is full,
//! no RPC call is done. It is up to upper-level code to ensure proper clean up
//! using `pop_oldest` call.

use std::num::Zero;
use num::BigUint;

use super::GenericNodeTable;
use super::Node;


// TODO(divius): make public?
static BUCKET_SIZE: uint = 1024;
static HASH_SIZE: uint = 160;


/// Kademlia node table.
///
/// Keeps nodes in a number of k-buckets (equal to bit size of ID in a system,
/// usually 160), where N-th k-bucket contains nodes with distance
/// from 2^N to 2^(N+1) from our node.
#[unstable]
pub struct NodeTable {
    own_id: BigUint,
    // TODO(divius): convert to more appropriate data structure
    buckets: Vec<KBucket>,
}

/// K-bucket - structure for keeping last nodes in Kademlia.
struct KBucket {
    data: Vec<Node>,
    size: uint
}


#[unstable]
impl NodeTable {
    /// Create a new node table.
    ///
    /// `own_id` -- ID of the current node (used to calculate metrics).
    pub fn new(own_id: BigUint) -> NodeTable {
        NodeTable::with_details(own_id, BUCKET_SIZE, HASH_SIZE)
    }

    // TODO(divius): make public
    fn with_details(own_id: BigUint, bucket_size: uint,
                    hash_size: uint) -> NodeTable {
        NodeTable {
            own_id: own_id,
            buckets: Vec::from_fn(hash_size,
                                  |_| KBucket::new(bucket_size)),
        }
    }

    #[inline]
    fn distance(id1: &BigUint, id2: &BigUint) -> BigUint {
        id1.bitxor(id2)
    }

    fn bucket_number(&self, id: &BigUint) -> uint {
        let diff = NodeTable::distance(&self.own_id, id);
        debug_assert!(!diff.is_zero());
        let res = diff.bits() - 1;
        debug!("ID {} relative to own ID {} falls into bucket {}",
               id, self.own_id, res);
        res
    }
}

#[unstable]
impl GenericNodeTable for NodeTable {
    fn update(&mut self, node: &Node) -> bool {
        assert!(node.id != self.own_id);
        let bucket = self.bucket_number(&node.id);
        self.buckets.get_mut(bucket).update(node)
    }

    fn find(&self, id: &BigUint, count: uint) -> Vec<Node> {
        debug_assert!(count > 0);
        assert!(*id != self.own_id)
        let bucket = self.bucket_number(id);
        self.buckets[bucket].find(id, count)
    }
}

impl KBucket {
    pub fn new(k: uint) -> KBucket {
        assert!(k > 0)
        KBucket {
            data: Vec::new(),
            size: k
        }
    }

    pub fn update(&mut self, node: &Node) -> bool {
        if self.data.iter().any(|x| x.id == node.id) {
            self.update_position(node);
            debug!("Promoted node {} to the top of kbucket", node);
            true
        }
        else if self.data.len() == self.size {
            debug!("Not adding new node {} to kbucket - no space left", node);
            false
        }
        else {
            self.data.push(node.clone());
            debug!("Added new node {} to kbucket", node);
            true
        }
    }

    pub fn find(&self, id: &BigUint, count: uint) -> Vec<Node> {
        let sort_fn = |a: &Node, b: &Node| {
            NodeTable::distance(id, &a.id).cmp(&NodeTable::distance(id, &b.id))
        };
        let mut data_copy = self.data.clone();
        data_copy.sort_by(sort_fn);
        Vec::from_slice(data_copy.slice(0, count))
    }

    fn update_position(&mut self, node: &Node) {
        // TODO(divius): 1. optimize, 2. make it less ugly
        let mut new_data = Vec::with_capacity(self.data.len());
        new_data.extend(self.data.iter()
                        .filter(|x| x.id != node.id)
                        .map(|x| x.clone()));
        new_data.push(node.clone());
        self.data = new_data;
    }
}



#[cfg(test)]
mod test {
    use std::from_str::FromStr;
    use std::num::FromPrimitive;
    use num::BigUint;
    use super::super::GenericNodeTable;
    use super::super::Node;
    use super::HASH_SIZE;
    use super::KBucket;
    use super::NodeTable;

    static ADDR: &'static str = "127.0.0.1:80";

    fn new_node(id: uint) -> Node {
        Node {
            id: FromPrimitive::from_uint(id).unwrap(),
            address: FromStr::from_str(ADDR).unwrap()
        }
    }

    fn prepare(count: uint) -> KBucket {
        KBucket {
            data: Vec::from_fn(count, |i| new_node(i)),
            size: 3,
        }
    }

    fn assert_node_list_eq(expected: &[&Node], actual: &Vec<Node>) {
        let act: Vec<BigUint> = actual.iter().map(|x| x.id.clone()).collect();
        let exp: Vec<BigUint> = expected.iter().map(|x| x.id.clone()).collect();
        assert_eq!(exp, act);
    }

    #[test]
    fn test_nodetable_new() {
        let n = NodeTable::new(FromPrimitive::from_uint(42).unwrap());
        assert_eq!(HASH_SIZE, n.buckets.len());
    }

    #[test]
    fn test_nodetable_bucket_number() {
        let n = NodeTable::new(FromPrimitive::from_uint(42).unwrap());
        let id = FromPrimitive::from_uint(41).unwrap();
        // 42 xor 41 == 3
        assert_eq!(1, n.bucket_number(&id));
    }

    #[test]
    fn test_nodetable_update() {
        let mut n = NodeTable::with_details(
            FromPrimitive::from_uint(42).unwrap(), 1, HASH_SIZE);
        let node = new_node(41);
        n.update(&node);
        assert_eq!(1, n.buckets[1].data.len());
        n.update(&node);
        assert_eq!(1, n.buckets[1].data.len());
    }

    #[test]
    fn test_nodetable_find() {
        let n = NodeTable {
            buckets: vec![prepare(1), prepare(3), prepare(1)],
            own_id: FromPrimitive::from_uint(0).unwrap()
        };
        // 0 xor 3 = 3, 1 xor 3 = 2, 2 xor 3 = 1
        let id = FromPrimitive::from_uint(3).unwrap();
        assert_node_list_eq([&n.buckets[1].data[2]],
                            &n.find(&id, 1));
    }

    #[test]
    fn test_kbucket_new() {
        let b = KBucket::new(3);
        assert_eq!(0, b.data.len());
        assert_eq!(3, b.size);
    }

    #[test]
    fn test_kbucket_update_unknown() {
        let mut b = prepare(1);
        let node = new_node(42);
        assert!(b.update(&node))
        assert_eq!(2, b.data.len());
        assert_eq!(node.id, b.data.get(1).id);
    }

    #[test]
    fn test_kbucket_update_known() {
        let mut b = prepare(2);
        let node = new_node(0);
        assert!(b.update(&node))
        assert_eq!(2, b.data.len());
        assert_eq!(node.id, b.data.get(1).id);
    }

    #[test]
    fn test_kbucket_update_conflict() {
        let mut b = prepare(3);  // 3 is size
        let node = new_node(42);
        assert!(!b.update(&node))
    }

    #[test]
    fn test_kbucket_find() {
        let b = prepare(3);
        // Nodes with ID's 0, 1, 2; assume our ID is also 2 (impossible IRL)
        let id = FromPrimitive::from_uint(2).unwrap();
        // 0 xor 2 = 2, 1 xor 2 = 3, 2 xor 2 = 0
        assert_node_list_eq([&b.data[2]], &b.find(&id, 1));
        assert_node_list_eq([&b.data[2], &b.data[0]], &b.find(&id, 2));
    }
}
