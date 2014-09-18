use std::io::net::ip::SocketAddr;
use super::hashid::HashId;


#[deriving(Clone, Show)]
pub struct Node {
    pub address: SocketAddr,
    pub id: HashId
}
