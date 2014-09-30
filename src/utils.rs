//! Various utilities

use std::io::net::ip;


/// Convert socket address to bytes in network order.
pub fn netaddr_to_netbytes(addr: &ip::SocketAddr) -> Vec<u8> {
    match addr.ip {
        ip::Ipv4Addr(a, b, c, d) =>
            vec![a, b, c, d, (addr.port >> 8) as u8, (addr.port & 0xFF) as u8],
        // TODO(divius): implement
        ip::Ipv6Addr(..) => fail!("IPv6 not implemented")
    }
}


#[cfg(test)]
pub mod test {
    use std::from_str::FromStr;
    use std::num::FromPrimitive;

    use num;

    use super::super::Node;


    pub static ADDR: &'static str = "127.0.0.1:80";

    pub fn new_node(id: uint) -> Node {
        Node {
            id: FromPrimitive::from_uint(id).unwrap(),
            address: FromStr::from_str(ADDR).unwrap()
        }
    }

    pub fn uint_to_id(id: uint) -> num::BigUint {
        FromPrimitive::from_uint(id).unwrap()
    }
}
