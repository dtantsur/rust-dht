// Copyright 2014 Dmitry "Divius" Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

//! DHT service

use std::sync::Arc;
use std::sync::RWLock;

use super::GenericNodeTable;
use super::GenericRpc;


// TODO(divius): implement
/// Structure representing main DHT service.
#[experimental]
pub struct Service<TNodeTable: GenericNodeTable, TRpc: GenericRpc> {
    node_table: Arc<RWLock<TNodeTable>>,
    rpc: Arc<TRpc>,
}


impl<TNodeTable: GenericNodeTable, TRpc: GenericRpc> Service<TNodeTable, TRpc> {
    #[experimental]
    pub fn new(node_table: TNodeTable, rpc: TRpc) -> Service<TNodeTable, TRpc> {
        Service {
            node_table: Arc::new(RWLock::new(node_table)),
            rpc: Arc::new(rpc),
        }
    }
}


#[cfg(test)]
mod test {
    use std::sync::Future;
    use num::BigUint;
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
        fn find(&self, id: &BigUint, count: uint) -> Vec<Node> {
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
        fn ping(&self, node: &Node) -> Future<bool> {
            Future::from_value(true)
        }
        #[allow(unused_variable)]
        fn find_node(&self, id: &BigUint) -> Future<Node> {
            Future::from_value(test::new_node(100500))
        }
    }

    #[test]
    fn test_new() {
        let s = Service::new(DummyNodeTable { last_node: None }, DummyRpc);
        let mut g = s.node_table.write();
        assert_eq!(0, g.find(&test::uint_to_id(42), 1).len());
        assert!(s.rpc.ping(&test::new_node(42)).get());
    }
}
