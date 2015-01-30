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

use std::old_io::{IoErrorKind, IoResult};
use std::sync;
use std::thread;
use std::num::ToPrimitive;

use super::super::base;
use super::super::knodetable;

use super::udpwrapper;


/// Generic implementation of basic KRPC DHT on which BitTorrent DHT is based.
///
/// No peer retrival is supported: just finding and pinging nodes.
///
/// Usually you will be creating it as:
/// ```
/// let service = dht::bt::KRpcService::new_default(this_node);
/// ```
pub struct KRpcService<TNodeTable: base::GenericNodeTable,
                       TSocket: udpwrapper::GenericSocketWrapper> {
    this_node: base::Node,
    node_table: sync::Arc<sync::RwLock<TNodeTable>>,
    socket: TSocket,
    // this is for tests only, proper cancelling will follow later
    active: sync::Arc<sync::RwLock<bool>>,
}

/// Default kind of KRpc service.
pub type DefaultKRpcService = KRpcService<knodetable::KNodeTable<base::Node>, udpwrapper::UdpSocketWrapper>;


// This can't be derived, compiler is confused because of Arc.
impl<TNodeTable: base::GenericNodeTable,
     TSocket: udpwrapper::GenericSocketWrapper>
Clone for KRpcService<TNodeTable, TSocket> {
    fn clone(&self) -> KRpcService<TNodeTable, TSocket> {
        KRpcService {
            this_node: self.this_node.clone(),
            node_table: self.node_table.clone(),
            socket: self.socket.clone(),
            active: self.active.clone(),
        }
    }
}

fn handle_incoming<TNodeTable: base::GenericNodeTable,
                   TSocket: udpwrapper::GenericSocketWrapper>
                   (mut service: KRpcService<TNodeTable, TSocket>) {
    while *service.active.read().unwrap() {
        match service.socket.receive() {
            Ok((package, addr)) =>
                // TODO(divius): implement
                debug!("Received {:?} from {:?}", package, addr),
            Err(e) =>
                if e.kind != IoErrorKind::TimedOut {
                    debug!("Error during receiving {}", e);
                }
        }
    }
}

impl KRpcService<knodetable::KNodeTable<base::Node>, udpwrapper::UdpSocketWrapper> {
    /// New service with default node table.
    pub fn new_default(this_node: base::Node) -> IoResult<DefaultKRpcService> {
        let node_table = knodetable::KNodeTable::new(this_node.id.clone());
        let socket = try!(udpwrapper::UdpSocketWrapper::new(&this_node));
        KRpcService::new(this_node, node_table, socket)
    }
}

impl<TNodeTable: base::GenericNodeTable,
     TSocket: udpwrapper::GenericSocketWrapper>
KRpcService<TNodeTable, TSocket> {
    /// New service with given node table and socket.
    pub fn new(this_node: base::Node, node_table: TNodeTable, socket: TSocket)
            -> IoResult<KRpcService<TNodeTable, TSocket>> {
        let self_ = KRpcService {
            this_node: this_node,
            node_table: sync::Arc::new(sync::RwLock::new(node_table)),
            socket: socket,
            active: sync::Arc::new(sync::RwLock::new(true)),
        };

        let self_clone = self_.clone();
        thread::Thread::spawn(move || handle_incoming(self_clone));

        Ok(self_)
    }

    /// Get lock guarding access to a node table.
    pub fn node_table_lock(&self) -> &sync::RwLock<TNodeTable> {
        &(*self.node_table)
    }

    /// Get reference to a socket wrapper.
    ///
    /// Clone it if you want to get mutable copy.
    pub fn socket_ref(&self) -> &TSocket {
        &self.socket
    }
}

#[unsafe_destructor]
impl<TNodeTable: base::GenericNodeTable,
     TSocket: udpwrapper::GenericSocketWrapper>
Drop for KRpcService<TNodeTable, TSocket> {
    fn drop(&mut self) {
        *self.active.write().unwrap() = false;
    }
}

#[cfg(test)]
mod test {
    use std::old_io::IoResult;
    use std::old_io::net::ip;

    use std::num::ToPrimitive;
    use num;

    use super::super::super::base::{self, GenericNodeTable};
    use super::super::super::utils::test;

    use super::super::protocol;
    use super::super::udpwrapper::GenericSocketWrapper;

    use super::KRpcService;


    struct DummyNodeTable {
        last_node: Option<base::Node>,
    }

    impl GenericNodeTable for DummyNodeTable {

        type P = base::Node;

        fn random_id(&self) -> num::BigUint {
            // This number is random, I promise :)
            test::usize_to_id(42)
        }

        fn update(&mut self, node: &base::Node) -> bool {
            self.last_node = Some(node.clone());
            true
        }
        #[allow(unused_variables)]
        fn find(&self, id: &num::BigUint, count: usize) -> Vec<base::Node> {
            match self.last_node {
                Some(ref n) if n.id == *id => vec![n.clone()],
                _ => vec![]
            }
        }
        fn pop_oldest(&mut self) -> Vec<base::Node> {
            vec![]
        }
    }

    #[derive(Clone)]
    struct DummySocketWrapper {
        last_package: Option<protocol::Package>,
        last_node: Option<base::Node>
    }

    impl GenericSocketWrapper for DummySocketWrapper {
        fn send(&mut self, package: &protocol::Package, node: &base::Node)
            -> IoResult<()> {
            self.last_package = Some(package.clone());
            self.last_node = Some(node.clone());
            Ok(())
        }

        fn receive(&mut self) -> IoResult<(protocol::Package, ip::SocketAddr)> {
            Ok((self.last_package.as_ref().unwrap().clone(),
                self.last_node.as_ref().unwrap().address.clone()))
        }
    }


    #[test]
    fn test_new() {
        let n = test::new_node_with_port(42, 8007);
        let sock = DummySocketWrapper { last_package: None, last_node: None };
        let s = KRpcService::new(n.clone(),
                                 DummyNodeTable { last_node: None },
                                 sock).unwrap();
        assert_eq!(n.id, s.this_node.id);
        let mut nt = s.node_table_lock().write().unwrap();
        nt.update(&test::new_node(1));
        assert_eq!(nt.random_id().to_u8().unwrap(), 42);
    }

    #[test]
    fn test_new_default() {
        let n = test::new_node_with_port(42, 8009);
        let s = KRpcService::new_default(n.clone()).unwrap();
        assert_eq!(n.id, s.this_node.id);
        let nt = s.node_table_lock().read().unwrap();
        assert_eq!(0, nt.find(&test::usize_to_id(1), 1).len());
    }
}
