// Copyright 2014 Dmitry "Divius" Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

//! UDP socket wrapper.

use std::old_io::{self, IoResult};
use std::old_io::net::ip;
use std::old_io::net::udp;

use bencode::{self, FromBencode, ToBencode};

use super::super::base;
use super::protocol;


/// Helper trait for any socket wrapper.
pub trait GenericSocketWrapper : Send + Clone {
    /// Send package to the node.
    fn send(&mut self, package: &protocol::Package, node: &base::Node)
        -> IoResult<()>;
    /// Receive package.
    fn receive(&mut self) -> IoResult<(protocol::Package, ip::SocketAddr)>;
}

/// Wrapper around UDP socket with converting to/from Package.
#[derive(Clone)]
pub struct UdpSocketWrapper {
    socket: udp::UdpSocket,
}


impl UdpSocketWrapper {
    /// New wrapper listening on the current node's address.
    pub fn new(this_node: &base::Node) -> IoResult<UdpSocketWrapper> {
        let socket = try!(udp::UdpSocket::bind(this_node.address.clone()));
        Ok(UdpSocketWrapper {
            socket: socket
        })
    }
}

impl GenericSocketWrapper for UdpSocketWrapper {
    /// Send package to the node.
    fn send(&mut self, package: &protocol::Package, node: &base::Node)
            -> IoResult<()> {
        let pkg = try!(package.to_bencode().to_bytes());
        try!(self.socket.send_to(pkg.as_slice(), node.address.clone()));
        Ok(())
    }

    /// Receive package.
    fn receive(&mut self) -> IoResult<(protocol::Package, ip::SocketAddr)> {
        let mut buf = [0u8; 1600];  // TODO(dtantsur): better number?

        let (amt, src) = try!(self.socket.recv_from(&mut buf));
        let benc = try!(bencode::from_buffer(&buf[0..amt]).map_err(|e| {
            old_io::IoError {
                kind: old_io::InvalidInput,
                desc: "Cannot read bencoded buffer",
                detail: Some(format!("Cannot read bencoded buffer: {}", e.msg))
            }
        }));

        let pkg = try!(FromBencode::from_bencode(&benc).ok_or_else(|| {
            old_io::IoError {
                kind: old_io::InvalidInput,
                desc: "Cannot decode bencoded buffer",
                detail: None
            }
        }));
        Ok((pkg, src))
    }
}


#[cfg(test)]
mod test {
    use super::super::protocol;

    use super::super::super::utils::test;

    use super::GenericSocketWrapper;

    // NOTE(dtantsur): it's hard to test receive w/o relying on network.

    fn new_package(payload: protocol::Payload) -> protocol::Package {
        protocol::Package {
            transaction_id: vec![1, 2, 3],
            sender: Some(test::new_node(42)),
            payload: payload
        }
    }

    #[test]
    fn test_new_send() {
        let n = test::new_node(42);
        let p = new_package(protocol::Payload::Error(1, "".to_string()));
        let mut sock = super::UdpSocketWrapper::new(&n).unwrap();
        sock.send(&p, &test::new_node(1)).unwrap();
    }
}
