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

use super::GenericNodeTable;
use super::GenericRpc;
use super::Node;


// TODO(divius): implement
/// Structure representing main DHT service.
#[experimental]
pub struct Service<TNodeTable:GenericNodeTable, TRpc:GenericRpc> {
    node_table: sync::Arc<sync::RWLock<TNodeTable>>,
    rpc: sync::Arc<sync::RWLock<TRpc>>,
}


#[experimental]
impl<TNodeTable:GenericNodeTable, TRpc:GenericRpc> Service<TNodeTable, TRpc> {
    pub fn new(own_node: &Node, node_table: TNodeTable) -> Service<TNodeTable, TRpc> {
        info!("Starting RPC for node {}", own_node);
        let rpc = GenericRpc::start_on(own_node);
        Service::new_with_rpc(node_table, rpc)
    }

    pub fn node_table_lock(&self) -> &sync::RWLock<TNodeTable> {
        self.node_table.deref()
    }

    pub fn rpc_lock(&self) -> &sync::RWLock<TRpc> {
        self.rpc.deref()
    }

    fn new_with_rpc(node_table: TNodeTable, rpc: TRpc) -> Service<TNodeTable, TRpc> {
        Service {
            node_table: sync::Arc::new(sync::RWLock::new(node_table)),
            rpc: sync::Arc::new(sync::RWLock::new(rpc)),
        }
    }
}


#[cfg(test)]
mod test {
    use std::sync;

    use num;

    use super::super::GenericNodeTable;
    use super::super::GenericRpc;
    use super::super::Node;

    use super::Service;

    use super::super::utils::test;


    struct DummyNodeTable {
        last_node: Option<Node>,
    }

    impl GenericNodeTable for DummyNodeTable {
        fn update(&mut self, node: &Node) -> bool {
            self.last_node = Some(node.clone());
            true
        }
        #[allow(unused_variable)]
        fn find(&self, id: &num::BigUint, count: uint) -> Vec<Node> {
            match self.last_node {
                Some(ref n) if n.id == *id => vec![n.clone()],
                _ => vec![]
            }
        }
        fn pop_oldest(&mut self) -> Vec<Node> {
            vec![]
        }
    }

    struct DummyRpc;
    impl GenericRpc for DummyRpc {
        #[allow(unused_variable)]
        fn ping(&mut self, node: &Node) -> sync::Future<bool> {
            sync::Future::from_value(true)
        }
        #[allow(unused_variable)]
        fn find_node(&mut self, id: &num::BigUint) -> sync::Future<Node> {
            sync::Future::from_value(test::new_node(100500))
        }
        #[allow(unused_variable)]
        fn start_on(own_node: &Node) -> DummyRpc {
            DummyRpc
        }
    }


    #[test]
    fn test_new() {
        let s: Service<DummyNodeTable, DummyRpc>;
        s = Service::new(&test::new_node(1),
                         DummyNodeTable { last_node: None });
        let mut g = s.node_table_lock().write();
        assert_eq!(0, g.find(&test::uint_to_id(42), 1).len());
        let mut r = s.rpc_lock().write();
        assert!(r.ping(&test::new_node(42)).get());
    }
}
