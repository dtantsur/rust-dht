use std::io::net::ip::SocketAddr;
use num::BigUint;


static HASH_SIZE: uint = 160;


pub trait GenericNodeTable {
    fn update(&mut self, node: &Node) -> bool;
}


#[deriving(PartialEq, Eq, Clone, Show)]
pub struct HashId {
    value: BigUint,
}


#[deriving(Clone, Show)]
pub struct Node {
    pub address: SocketAddr,
    pub id: HashId
}


impl HashId {
    pub fn new(value: BigUint) -> Option<HashId> {
        if value.bits() <= HASH_SIZE {
            Some(HashId {value: value})
        } else {
            None
        }
    }

    #[inline]
    pub fn hash_size() -> uint { HASH_SIZE }

    pub fn from_uint(value: uint) -> HashId {
        let number: BigUint = FromPrimitive::from_uint(value).unwrap();
        let maybe_id = HashId::new(number);
        maybe_id.unwrap()
    }

    pub fn distance_to(&self, second: &HashId) -> BigUint {
        self.value.bitxor(&second.value)
    }
}


#[cfg(test)]
mod test {
    mod hashid {
        use num::BigUint;
        use std::uint;

        use super::super::HashId;

        fn prepare(value: int) -> (BigUint, HashId) {
            let number: BigUint = FromPrimitive::from_int(value).unwrap();
            let maybe_id = HashId::new(number.clone());
            (number, maybe_id.unwrap())
        }

        #[test]
        fn test_new() {
            let (num, id) = prepare(42);
            assert_eq!(num, id.value);
        }

        #[test]
        fn test_new_fail_too_large() {
            let big: BigUint = FromPrimitive::from_uint(uint::MAX).unwrap();
            let very_big = range(0i, 6i).fold(FromPrimitive::from_int(1).unwrap(),
                                              |acc, _| big * acc);
            assert!(HashId::new(very_big).is_none())
        }

        #[test]
        fn test_distance_to() {
            let id = HashId::from_uint(42);
            let id2 = HashId::from_uint(41);
            // 42 xor 41 == 3
            assert_eq!(3, id.distance_to(&id2).to_int().unwrap());
            assert_eq!(3, id2.distance_to(&id).to_int().unwrap());
            // 42 xor 42 == 0
            assert_eq!(0, id.distance_to(&id).to_int().unwrap());
        }
    }
}
