// Copyright 2016 Dmitry "Divius" Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

//! Protocol-agnostic service implementation

use std::marker;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::{GenericId, GenericNodeTable, Node};


static MAX_NODE_COUNT: usize = 16;


/// Result of the find operations - either data or nodes closest to it.
#[derive(Debug)]
pub enum FindResult<TId, TAddr, TData> {
    Value(TData),
    ClosestNodes(Vec<Node<TId, TAddr>>),
    Nothing
}

/// Handler - implementation of DHT requests.
pub struct Handler<TId, TAddr, TNodeTable, TData>
        where TId: GenericId,
              TNodeTable: GenericNodeTable<TId, TAddr>,
              TData: Send + Sync + Clone {
    _phantom: marker::PhantomData<TAddr>,
    node_id: TId,
    table: Arc<RwLock<TNodeTable>>,
    data: Arc<RwLock<HashMap<TId, TData>>>,
    clean_needed: bool,
}

/// Protocol agnostic DHT service.
///
/// Its type parameters are `TNodeTable` - the node table implementation
/// (see e.g. `KNodeTable`) and `TData` - stored data type.
///
/// The service starts a network listening loop in a separate thread.
pub struct Service<TId, TAddr, TNodeTable, TData>
        where TId: GenericId,
              TNodeTable: GenericNodeTable<TId, TAddr>,
              TData: Send + Sync + Clone {
    handler: Handler<TId, TAddr, TNodeTable, TData>,
    node_id: TId,
    table: Arc<RwLock<TNodeTable>>,
    data: Arc<RwLock<HashMap<TId, TData>>>
}


impl<TId, TAddr, TNodeTable, TData> Service<TId, TAddr, TNodeTable, TData>
        where TId: GenericId,
              TAddr: Send + Sync,
              TNodeTable: GenericNodeTable<TId, TAddr>,
              TData: Send + Sync + Clone {
    /// Create a service with a random ID.
    pub fn new(node_table: TNodeTable) -> Service<TId, TAddr, TNodeTable, TData> {
        let node_id = node_table.random_id();
        Service::new_with_id(node_table, node_id)
    }
    /// Create a service with a given ID.
    pub fn new_with_id(node_table: TNodeTable, node_id: TId)
            -> Service<TId, TAddr, TNodeTable, TData> {
        let table = Arc::new(RwLock::new(node_table));
        let data = Arc::new(RwLock::new(HashMap::new()));
        let handler = Handler {
            _phantom: marker::PhantomData,
            node_id: node_id.clone(),
            table: table.clone(),
            data: data.clone(),
            clean_needed: false
        };
        Service {
            handler: handler,
            node_id: node_id,
            table: table,
            data: data
        }
    }

    /// Get an immutable reference to the node table.
    pub fn node_table(&self) -> RwLockReadGuard<TNodeTable> {
        self.table.read().unwrap()
    }
    /// Get a mutable reference to the node table.
    pub fn node_table_mut(&mut self) -> RwLockWriteGuard<TNodeTable> {
        self.table.write().unwrap()
    }
    /// Get the current node ID.
    pub fn node_id(&self) -> &TId {
        &self.node_id
    }
    /// Get an immutable reference to the data.
    pub fn stored_data(&self)
            -> RwLockReadGuard<HashMap<TId, TData>> {
        self.data.read().unwrap()
    }
    /// Get an immutable reference to the data.
    pub fn stored_data_mut(&mut self)
            -> RwLockWriteGuard<HashMap<TId, TData>> {
        self.data.write().unwrap()
    }
    /// Check if some buckets are full already.
    pub fn clean_needed(&self) -> bool {
        self.handler.clean_needed
    }

    /// Try to clean up the table by checking the oldest records.
    ///
    /// Should be called periodically, especially when clean_needed is true.
    pub fn clean_up<TCheck>(&mut self, mut check: TCheck)
            where TCheck: FnMut(&Node<TId, TAddr>) -> bool {
        {
            let mut node_table = self.node_table_mut();
            let oldest = node_table.pop_oldest();
            for node in oldest {
                if check(&node) {
                    node_table.update(&node);
                }
            }
        }
        self.handler.clean_needed = false;
    }
}

impl<TId, TAddr, TNodeTable, TData> Handler<TId, TAddr, TNodeTable, TData>
        where TId: GenericId,
              TNodeTable: GenericNodeTable<TId, TAddr>,
              TData: Send + Sync + Clone {
    /// Process the ping request.
    ///
    /// Essentially remembers the incoming node and returns true.
    pub fn on_ping(&mut self, sender: &Node<TId, TAddr>) -> bool {
        self.update(sender);
        true
    }
    /// Process the find request.
    pub fn on_find_node(&mut self, sender: &Node<TId, TAddr>, id: &TId) -> Vec<Node<TId, TAddr>> {
        let res = self.table.read().unwrap().find(&id, MAX_NODE_COUNT);
        self.update(sender);
        res
    }
    /// Find a value or the closes nodes.
    pub fn on_find_value(&mut self, sender: &Node<TId, TAddr>, id: &TId)
            -> FindResult<TId, TAddr, TData> {
        self.update(sender);
        let data = self.data.read().unwrap();
        let table = self.table.read().unwrap();
        let res = match data.get(&id) {
            Some(value) => FindResult::Value(value.clone()),
            None => FindResult::ClosestNodes(table.find(&id, MAX_NODE_COUNT))
        };
        res
    }

    fn update(&mut self, node: &Node<TId, TAddr>) {
        if node.id == self.node_id {
            return
        }

        if ! self.table.write().unwrap().update(&node) {
            self.clean_needed = true;
        }
    }
}


#[cfg(test)]
pub mod test {
    use std::net;
    use super::super::{GenericNodeTable, Node};
    use super::super::utils::test;
    type TestsIdType = test::IdType;

    use super::{FindResult, Service};


    struct DummyNodeTable {
        pub node: Option<Node<TestsIdType, net::SocketAddr>>
    }

    impl GenericNodeTable<TestsIdType, net::SocketAddr> for DummyNodeTable {
        fn random_id(&self) -> TestsIdType {
            test::make_id(42)
        }

        fn update(&mut self, node: &Node<TestsIdType, net::SocketAddr>) -> bool {
            match self.node {
                Some(..) => false,
                None => {
                    self.node = Some(node.clone());
                    true
                }
            }
        }

        fn find(&self, id: &TestsIdType, _count: usize) -> Vec<Node<TestsIdType, net::SocketAddr>> {
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

        fn pop_oldest(&mut self) -> Vec<Node<TestsIdType, net::SocketAddr>> {
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
        let mut svc: Service<TestsIdType, net::SocketAddr, DummyNodeTable, String> =
            Service::new(node_table);

        assert_eq!(test::make_id(42), *svc.node_id());
        assert!(svc.node_table().node.is_none());
        assert!(svc.node_table_mut().node.is_none());
        assert!(!svc.clean_needed());
    }

    #[test]
    fn test_find_saves_node() {
        let node_table = DummyNodeTable { node: None };
        let mut svc: Service<TestsIdType, net::SocketAddr, DummyNodeTable, String> =
            Service::new(node_table);
        let node = test::new_node(test::make_id(43));

        assert!(svc.handler.on_find_node(&node, &node.id).is_empty());
        let result = svc.handler.on_find_node(&node, &node.id);
        assert_eq!(1, result.len());
        assert_eq!(test::make_id(43), result.get(0).unwrap().id)
    }

    #[test]
    fn test_ping_find_clean() {
        let node_table = DummyNodeTable { node: None };
        let mut svc: Service<TestsIdType, net::SocketAddr, DummyNodeTable, String> =
            Service::new(node_table);
        let node = test::new_node(test::make_id(43));

        assert!(svc.handler.on_ping(&node));
        assert_eq!(test::make_id(43), svc.node_table().node.as_ref().unwrap().id);
        assert!(!svc.clean_needed());

        assert!(svc.handler.on_ping(&test::new_node(test::make_id(44))));
        assert_eq!(test::make_id(43), svc.node_table().node.as_ref().unwrap().id);
        assert!(svc.clean_needed());

        let mut result = svc.handler.on_find_node(&node, &node.id);
        assert_eq!(1, result.len());
        assert_eq!(test::make_id(43), result.get(0).unwrap().id);

        let mut flag = false;
        svc.clean_up(|node| {
            assert_eq!(test::make_id(43), node.id);
            flag = true;
            true
        });
        assert!(flag);
        assert!(!svc.clean_needed());

        result = svc.handler.on_find_node(&node, &node.id);
        assert_eq!(1, result.len());
        assert_eq!(test::make_id(43), result.get(0).unwrap().id);

        flag = false;
        svc.clean_up(|node| {
            assert_eq!(test::make_id(43), node.id);
            flag = true;
            false
        });
        assert!(flag);
        assert!(!svc.clean_needed());
        assert!(svc.handler.on_find_node(&node, &node.id).is_empty());
    }

    #[test]
    fn test_ping_find_value() {
        let node_table = DummyNodeTable { node: None };
        let mut svc: Service<TestsIdType, net::SocketAddr, DummyNodeTable, String> =
            Service::new(node_table);
        let node = test::new_node(test::make_id(43));
        let id1: TestsIdType = test::make_id(44);
        let id2: TestsIdType = test::make_id(43);

        svc.handler.on_ping(&node);
        svc.stored_data_mut().insert(id1.clone(), "foobar".to_string());

        {
            let res1 = svc.handler.on_find_value(&node, &id1);
            match res1 {
                FindResult::Value(value) => assert_eq!("foobar", value),
                _ => panic!("wrong result {:?}", res1)
            }
        }

        {
            let res2 = svc.handler.on_find_value(&node, &id2);
            match res2 {
                FindResult::ClosestNodes(nodes) => assert_eq!(1, nodes.len()),
                _ => panic!("wrong result {:?}", res2)
            }
        }
    }
}
