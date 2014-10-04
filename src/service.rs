// Copyright 2014 Dmitry "Divius" Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

//! DHT service

use std::sync;

use super::base::{mod, GenericNodeTable, GenericRpc};
use super::knodetable;


// TODO(divius): implement
/// Structure representing main DHT service.
#[experimental]
pub struct Service<TNodeTable:GenericNodeTable, TRpc:GenericRpc> {
    node_table: sync::Arc<sync::RWLock<TNodeTable>>,
    rpc: sync::Arc<TRpc>,
}


#[experimental]
impl<TRpc:GenericRpc> Service<knodetable::KNodeTable, TRpc> {
    /// Create new service with given RPC implementation.
    pub fn new(this_node: &base::Node, rpc: TRpc) -> Service<knodetable::KNodeTable, TRpc> {
        Service::with_node_table(
            knodetable::KNodeTable::new(this_node.id.clone()), rpc)
    }
}

#[experimental]
impl<TNodeTable:GenericNodeTable, TRpc:GenericRpc> Service<TNodeTable, TRpc> {
    /// Create new service with given node table and RPC implementations.
    pub fn with_node_table(node_table: TNodeTable, rpc: TRpc)
            -> Service<TNodeTable, TRpc> {
        Service {
            node_table: sync::Arc::new(sync::RWLock::new(node_table)),
            rpc: sync::Arc::new(rpc),
        }
    }

    /// Get lock object for a node_table.
    pub fn node_table(&self) -> &sync::RWLock<TNodeTable> {
        self.node_table.deref()
    }

    /// Get instanc of RPC object.
    pub fn rpc(&self) -> &TRpc {
        self.rpc.deref()
    }
}


#[cfg(test)]
mod test {
    use std::sync;

    use num;

    use super::super::base::{mod, GenericNodeTable, GenericRpc};

    use super::Service;

    use super::super::utils::test;


    struct DummyNodeTable {
        last_node: Option<base::Node>,
    }

    impl GenericNodeTable for DummyNodeTable {
        fn random_id(&self) -> num::BigUint {
            // This number is random, I promise :)
            test::uint_to_id(42)
        }

        fn update(&mut self, node: &base::Node) -> bool {
            self.last_node = Some(node.clone());
            true
        }
        #[allow(unused_variable)]
        fn find(&self, id: &num::BigUint, count: uint) -> Vec<base::Node> {
            match self.last_node {
                Some(ref n) if n.id == *id => vec![n.clone()],
                _ => vec![]
            }
        }
        fn pop_oldest(&mut self) -> Vec<base::Node> {
            vec![]
        }
    }

    struct DummyRpc;
    impl GenericRpc for DummyRpc {
        #[allow(unused_variable)]
        fn ping(&self, node: &base::Node) -> sync::Future<bool> {
            sync::Future::from_value(true)
        }
        #[allow(unused_variable)]
        fn find_node(&self, id: &num::BigUint) -> sync::Future<base::LookupResult> {
            sync::Future::from_value(base::NodeFound(test::new_node(100500)))
        }
    }


    #[test]
    fn test_new() {
        let s = Service::new(&test::new_node(1), DummyRpc);
        let g = s.node_table().write();
        assert_eq!(0, g.find(&test::uint_to_id(42), 1).len());
        assert!(s.rpc().ping(&test::new_node(42)).get());
    }

    #[test]
    fn test_with_node_table() {
        let s = Service::with_node_table(DummyNodeTable { last_node: None },
                                         DummyRpc);
        let g = s.node_table().write();
        assert_eq!(0, g.find(&test::uint_to_id(42), 1).len());
        assert!(s.rpc().ping(&test::new_node(42)).get());
    }
}
