//! Various utilities


#[cfg(test)]
pub mod test {
    use std::net;

    use super::super::Node;

    /*
    pub type IdType = u64;
    pub fn make_id(i: u8) -> IdType {
        i as IdType
    }*/
    pub type IdType = Vec<u8>;
    pub fn make_id(i: u8) -> IdType {
        vec![i]
    }

    pub static ADDR: &'static str = "127.0.0.1:8008";

    pub fn new_node(id: IdType) -> Node<IdType, net::SocketAddr> {
        new_node_with_port(id, 8008)
    }

    pub fn new_node_with_port(id: IdType, port: u16) -> Node<IdType, net::SocketAddr> {
        Node {
            id: id,
            address: net::SocketAddr::V4(net::SocketAddrV4::new(
                net::Ipv4Addr::new(127, 0, 0, 1),
                port
            ))
        }
    }
}
