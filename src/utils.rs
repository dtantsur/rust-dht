//! Various utilities


#[cfg(test)]
pub mod test {
    use std::net;

    use num;
    use num::FromPrimitive;

    use super::super::Node;


    pub static ADDR: &'static str = "127.0.0.1:8008";

    pub fn new_node(id: usize) -> Node {
        new_node_with_port(id, 8008)
    }

    pub fn new_node_with_port(id: usize, port: u16) -> Node {
        Node {
            id: FromPrimitive::from_usize(id).unwrap(),
            address: net::SocketAddr::V4(net::SocketAddrV4::new(
                net::Ipv4Addr::new(127, 0, 0, 1),
                port
            ))
        }
    }

    pub fn usize_to_id(id: usize) -> num::BigUint {
        FromPrimitive::from_usize(id).unwrap()
    }
}
