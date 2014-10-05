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

use std::sync;

use super::super::base;


/// Implementation of basic KRPC DHT on which BitTorrent DHT is based.
///
/// No peer retrival is supported: just finding and pinging nodes.
pub struct KRpcService<TNodeTable: base::GenericNodeTable> {
    this_node: base::Node,
    node_table: sync::Arc<sync::RWLock<TNodeTable>>,
}
