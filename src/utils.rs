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
