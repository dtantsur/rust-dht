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
    /*
    #[cfg(feature="num")]
    use num;
    #[cfg(feature="num")]
    use num::FromPrimitive;

    pub type IdType = num::BigUint;
    pub fn make_id(i: u8) -> IdType {
        FromPrimitive::from_usize(i as usize).unwrap()
    }*/

    pub static ADDR: &'static str = "127.0.0.1:8008";

    pub fn new_node(id: IdType) -> Node<IdType> {
        new_node_with_port(id, 8008)
    }

    pub fn new_node_with_port(id: IdType, port: u16) -> Node<IdType> {
        Node {
            id: id,
            address: net::SocketAddr::V4(net::SocketAddrV4::new(
                net::Ipv4Addr::new(127, 0, 0, 1),
                port
            ))
        }
    }
}
