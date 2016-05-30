// Copyright 2016 Dmitry "Divius" Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

//! Protocol-agnostic service implementation

use num;

use super::{GenericNodeTable, Node};


static MAX_NODE_COUNT: usize = 16;


/// Service - generic implementation of DHT calls.
pub struct BaseService<TNodeTable> where TNodeTable: GenericNodeTable {
    table: TNodeTable,
    node_id: num::BigUint,
    clean_needed: bool
}


impl<TNodeTable> BaseService<TNodeTable> where TNodeTable: GenericNodeTable {
    /// Create a service with a random ID.
    pub fn new(node_table: TNodeTable) -> BaseService<TNodeTable> {
        let node_id = node_table.random_id();
        BaseService::new_with_id(node_table, node_id)
    }
    /// Create a service with a given ID.
    pub fn new_with_id(node_table: TNodeTable, node_id: num::BigUint)
            -> BaseService<TNodeTable> {
        BaseService {
            table: node_table,
            node_id: node_id,
            clean_needed: false
        }
    }

    /// Get an immutable reference to the node table.
    pub fn node_table(&self) -> &TNodeTable { &self.table }
    /// Get a mutable reference to the node table.
    pub fn node_table_mut(&mut self) -> &mut TNodeTable { &mut self.table }
    /// Get the current node ID.
    pub fn node_id(&self) -> &num::BigUint { &self.node_id }
    /// Check if some buckets are full already.
    pub fn clean_needed(&self) -> bool { self.clean_needed }

    /// Process the ping request.
    ///
    /// Essentially remembers the incoming node and returns true.
    pub fn ping(&mut self, sender: &Node) -> bool {
        self.update(sender);
        true
    }
    /// Process the find request.
    pub fn find_node(&mut self, sender: &Node, id: &num::BigUint) -> Vec<Node> {
        let res = self.table.find(&id, MAX_NODE_COUNT);
        self.update(sender);
        res
    }

    /// Try to clean up the table by checking the oldest records.
    ///
    /// Should be called periodically, especially when clean_needed is true.
    pub fn clean_up<TCheck>(&mut self, mut check: TCheck)
            where TCheck: FnMut(&Node) -> bool {
        let oldest = self.table.pop_oldest();
        for node in oldest {
            if check(&node) {
                self.table.update(&node);
            }
        }
        self.clean_needed = false;
    }

    fn update(&mut self, node: &Node) {
        if node.id == self.node_id {
            return
        }

        if ! self.table.update(&node) {
            self.clean_needed = true;
        }
    }
}


#[cfg(test)]
pub mod test {
    use num::{self, FromPrimitive, ToPrimitive};

    use super::super::{GenericNodeTable, Node};
    use super::super::utils::test;

    use super::BaseService;


    struct DummyNodeTable {
        pub node: Option<Node>
    }

    impl GenericNodeTable for DummyNodeTable {
        fn random_id(&self) -> num::BigUint {
            FromPrimitive::from_usize(42).unwrap()
        }

        fn update(&mut self, node: &Node) -> bool {
            match self.node {
                Some(..) => false,
                None => {
                    self.node = Some(node.clone());
                    true
                }
            }
        }

        fn find(&self, id: &num::BigUint, _count: usize) -> Vec<Node> {
            if let Some(ref node) = self.node {
                if node.id == *id {
                    vec![node.clone()]
                }
                else {
                    vec![]
                }
            }
            else {
                vec![]
            }
        }

        fn pop_oldest(&mut self) -> Vec<Node> {
            let result;
            if let Some(ref node) = self.node {
                result = vec![node.clone()];
            }
            else {
                result = vec![];
            }
            self.node = None;
            result
        }
    }

    #[test]
    fn test_new() {
        let node_table = DummyNodeTable { node: None };
        let mut svc = BaseService::new(node_table);

        assert_eq!(42, svc.node_id().to_i8().unwrap());
        assert!(svc.node_table().node.is_none());
        assert!(svc.node_table_mut().node.is_none());
        assert!(!svc.clean_needed());
    }

    #[test]
    fn test_find_saves_node() {
        let node_table = DummyNodeTable { node: None };
        let mut svc = BaseService::new(node_table);
        let node = test::new_node(43);

        assert!(svc.find_node(&node, &node.id).is_empty());
        let result = svc.find_node(&node, &node.id);
        assert_eq!(1, result.len());
        assert_eq!(43, result.get(0).unwrap().id.to_i8().unwrap())
    }

    #[test]
    fn test_ping_find_clean() {
        let node_table = DummyNodeTable { node: None };
        let mut svc = BaseService::new(node_table);
        let node = test::new_node(43);

        assert!(svc.ping(&node));
        assert_eq!(43, svc.table.node.as_ref().unwrap().id.to_i8().unwrap());
        assert!(!svc.clean_needed());

        assert!(svc.ping(&test::new_node(44)));
        assert_eq!(43, svc.table.node.as_ref().unwrap().id.to_i8().unwrap());
        assert!(svc.clean_needed());

        let mut result = svc.find_node(&node, &node.id);
        assert_eq!(1, result.len());
        assert_eq!(43, result.get(0).unwrap().id.to_i8().unwrap());

        let mut flag = false;
        svc.clean_up(|node| {
            assert_eq!(43, node.id.to_i8().unwrap());
            flag = true;
            true
        });
        assert!(flag);
        assert!(!svc.clean_needed());

        result = svc.find_node(&node, &node.id);
        assert_eq!(1, result.len());
        assert_eq!(43, result.get(0).unwrap().id.to_i8().unwrap());

        flag = false;
        svc.clean_up(|node| {
            assert_eq!(43, node.id.to_i8().unwrap());
            flag = true;
            false
        });
        assert!(flag);
        assert!(!svc.clean_needed());
        assert!(svc.find_node(&node, &node.id).is_empty());
    }
}
