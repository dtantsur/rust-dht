use std::num::Zero;

use super::GenericNodeTable;
use super::HashId;
use super::Node;


pub struct NodeTable {
    own_id: HashId,
    buckets: Vec<KBucket>,
}

struct KBucket {
    data: Vec<Node>,
    size: uint
}


impl NodeTable {
    pub fn new(own_id: HashId, bucket_size: uint) -> NodeTable {
        NodeTable {
            own_id: own_id,
            buckets: Vec::from_fn(HashId::hash_size(),
                                  |_| KBucket::new(bucket_size)),
        }
    }

    fn bucket_number(&self, id: &HashId) -> uint {
        let diff = id.distance_to(&self.own_id);
        assert!(!diff.is_zero());
        diff.bits() - 1
    }
}

impl GenericNodeTable for NodeTable {
    fn update(&mut self, node: &Node) -> bool {
        assert!(node.id != self.own_id);
        let bucket = self.bucket_number(&node.id);
        self.buckets.get_mut(bucket).update(node)
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
            self._update_position(node);
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

    fn _update_position(&mut self, node: &Node) {
        // TODO(dtantsur): 1. optimize, 2. make it less ugly
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
    use super::super::GenericNodeTable;
    use super::super::HashId;
    use super::super::Node;
    use super::KBucket;
    use super::NodeTable;

    static ADDR: &'static str = "127.0.0.1:80";

    fn new_node(id: uint) -> Node {
        Node {
            id: HashId::from_uint(id),
            address: FromStr::from_str(ADDR).unwrap()
        }
    }

    #[test]
    fn test_nodetable_new() {
        let n = NodeTable::new(HashId::from_uint(42), 5);
        assert_eq!(HashId::hash_size(), n.buckets.len());
    }

    #[test]
    fn test_nodetable_bucket_number() {
        let n = NodeTable::new(HashId::from_uint(42), 5);
        let id = HashId::from_uint(41);
        // 42 xor 41 == 3
        assert_eq!(1, n.bucket_number(&id));
    }

    #[test]
    fn test_nodetable_update() {
        let mut n = NodeTable::new(HashId::from_uint(42), 1);
        let node = new_node(41);
        n.update(&node);
        assert_eq!(1, n.buckets[1].data.len());
        n.update(&node);
        assert_eq!(1, n.buckets[1].data.len());
    }

    fn prepare(count: uint) -> KBucket {
        KBucket {
            data: Vec::from_fn(count, |i| new_node(i)),
            size: 3,
        }
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
}
