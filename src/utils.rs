//! Various utilities
/*
use std::old_io::net::ip;


/// Convert socket address to bytes in network order.
pub fn netaddr_to_netbytes(addr: &ip::SocketAddr) -> Vec<u8> {
    match addr.ip {
        ip::Ipv4Addr(a, b, c, d) =>
            vec![a, b, c, d, (addr.port >> 8) as u8, (addr.port & 0xFF) as u8],
        // TODO(divius): implement
        ip::Ipv6Addr(..) => panic!("IPv6 not implemented")
    }
}

/// Get socket address from netbytes.
pub fn netaddr_from_netbytes(bytes: &[u8]) -> ip::SocketAddr {
    assert_eq!(6, bytes.len());
    ip::SocketAddr {
        ip: ip::Ipv4Addr(bytes[0], bytes[1], bytes[2], bytes[3]),
        port: ((bytes[4] as u16) << 8) + bytes[5] as u16
    }
}
*/

#[cfg(test)]
pub mod test {
    use std::net::SocketAddr;
    use std::net::SocketAddrV4;
    use std::net::Ipv4Addr;
    use num::traits::FromPrimitive;

    use num;

    use super::super::Node;


    pub static ADDR: &'static str = "127.0.0.1:8008";

    pub fn new_node(id: usize) -> Node {
        new_node_with_port(id, 8008)
    }

    pub fn new_node_with_port(id: usize, port: u16) -> Node {
        Node {
            id: FromPrimitive::from_usize(id).unwrap(),
            address: SocketAddr::V4( SocketAddrV4::new( 
                Ipv4Addr::new(127, 0, 0, 1),
                port
            ) )
        }
    }

    pub fn usize_to_id(id: usize) -> num::BigUint {
        FromPrimitive::from_usize(id).unwrap()
    }
}
