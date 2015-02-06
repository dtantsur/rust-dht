// Copyright 2014 Dmitry "Divius" Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

//! BitTorrent DHT implementation.

pub use self::krpc::DefaultKRpcService;
pub use self::krpc::KRpcService;

mod krpc;
pub mod protocol;
pub mod udpwrapper;
