// Copyright 2014 Dmitry "Divius" Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

//! KRPC DHT service as described in
//! [BEP 0005](http://www.bittorrent.org/beps/bep_0005.html).
//!
//! Create as:
//! ```
//! let service = dht::bt::KRpcService::new(this_node);
//! ```

use std::io::net::udp;
use std::sync;

use super::super::base;
use super::super::knodetable;


/// Implementation of basic KRPC DHT on which BitTorrent DHT is based.
///
/// No peer retrival is supported: just finding and pinging nodes.
pub struct KRpcService<TNodeTable: base::GenericNodeTable> {
    this_node: base::Node,
    node_table: sync::Arc<sync::RWLock<TNodeTable>>,
}


fn handle_incoming(socket: udp::UdpSocket) {
    // TODO(divius): implement
    drop(socket);
}

impl KRpcService<knodetable::KNodeTable> {
    /// New service with default node table.
    pub fn new_default(this_node: base::Node)
            -> KRpcService<knodetable::KNodeTable> {
        let node_table = knodetable::KNodeTable::new(this_node.id.clone());
        KRpcService::new(this_node, node_table)
    }
}

impl<TNodeTable: base::GenericNodeTable> KRpcService<TNodeTable> {
    /// New service with given node table.
    pub fn new(this_node: base::Node, node_table: TNodeTable)
            -> KRpcService<TNodeTable> {
        let socket = udp::UdpSocket::bind(this_node.address.clone()).ok()
            .expect(format!("Cannot bind to {}", this_node.address).as_slice());
        spawn(proc() handle_incoming(socket));

        KRpcService {
            this_node: this_node,
            node_table: sync::Arc::new(sync::RWLock::new(node_table)),
        }
    }

    /// Get lock guarding access to a node table.
    pub fn node_table_lock(&self) -> &sync::RWLock<TNodeTable> {
        self.node_table.deref()
    }
}


#[cfg(test)]
mod test {
    use num;

    use super::super::super::base::{mod, GenericNodeTable};
    use super::super::super::utils::test;

    use super::KRpcService;


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


    #[test]
    fn test_new() {
        let n = test::new_node(42);
        let s = KRpcService::new(n.clone(),
                                 DummyNodeTable { last_node: None });
        assert_eq!(n.id, s.this_node.id);
        let mut nt = s.node_table_lock().write();
        nt.update(&test::new_node(1));
        let nt2 = nt.downgrade();
        assert_eq!(nt2.random_id().to_u8().unwrap(), 42);
    }

    #[test]
    fn test_new_default() {
        let n = test::new_node(42);
        let s = KRpcService::new_default(n.clone());
        assert_eq!(n.id, s.this_node.id);
        let nt = s.node_table_lock().read();
        assert_eq!(0, nt.find(&test::uint_to_id(1), 1).len());
    }
}
