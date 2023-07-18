use crate::types::errors::Kind;
use sp_std::marker::Sized;
use sp_std::vec::Vec;

// "Default" trait is implemented for a few selected fixed-array types. Taken we can't implement
// the trait outside of a crate, we created a new one that mimics the stdlib.
pub trait DefaultFrom {
    fn default() -> Self;
}

pub trait FromBytes {
    fn from_bytes(data: &[u8]) -> Result<&Self, Kind>;
}

pub trait FromVec {
    fn from_vec(data: &Vec<u8>) -> Result<Self, Kind>
    where
        Self: Sized;
}

pub trait ToRlp {
    fn to_rlp(&self) -> Vec<u8>;
}

pub trait FromRlp {
    fn from_rlp(bytes: &[u8]) -> Result<Self, Kind>
    where
        Self: Sized;
}
