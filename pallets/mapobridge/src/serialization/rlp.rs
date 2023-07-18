use crate::alloc::borrow::ToOwned;
use crate::traits::FromBytes;
use num_bigint::{BigInt as Integer, Sign};
use num_traits::Zero;
use rlp::{DecoderError, Rlp};
use sp_std::vec::Vec;
pub fn rlp_list_field_from_bytes<T>(rlp: &Rlp, index: usize) -> Result<T, DecoderError>
where
    T: FromBytes + Clone,
{
    rlp.at(index)?
        .decoder()
        .decode_value(|data| match T::from_bytes(data) {
            Ok(field) => Ok(field.to_owned()),
            Err(_) => Err(DecoderError::Custom("invalid length data")),
        })
}

pub fn rlp_field_from_bytes<T>(rlp: &Rlp) -> Result<T, DecoderError>
where
    T: FromBytes + Clone,
{
    rlp.decoder()
        .decode_value(|data| match T::from_bytes(data) {
            Ok(field) => Ok(field.to_owned()),
            Err(_) => Err(DecoderError::Custom("invalid length data")),
        })
}

pub fn rlp_to_big_int(rlp: &Rlp, index: usize) -> Result<Integer, DecoderError> {
    rlp.at(index)?
        .decoder()
        .decode_value(|bytes| Ok(Integer::from_bytes_be(Sign::Plus, bytes)))
}

pub fn big_int_to_rlp_compat_bytes(val: &Integer) -> Vec<u8> {
    if val.is_zero() {
        Vec::new()
    } else {
        val.to_bytes_be().1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::Num;

    #[test]
    fn parses_big_int() {
        let expected_bytes: Vec<u8> = vec![236, 160, 242, 246, 191, 251, 255, 120];
        let rlp_list = bytes_to_rlp_list(&expected_bytes);
        let r = Rlp::new(&rlp_list);

        assert_eq!(
            "17050895330821537656",
            rlp_to_big_int(&r, 0).unwrap().to_str_radix(10),
        );

        assert_eq!(
            big_int_to_rlp_compat_bytes(&Integer::from_str_radix("0", 10).unwrap()),
            Vec::<u8>::new(),
        );

        assert_eq!(
            big_int_to_rlp_compat_bytes(
                &Integer::from_str_radix("17050895330821537656", 10).unwrap()
            ),
            expected_bytes,
        );
    }

    fn bytes_to_rlp_list(bytes: &[u8]) -> Vec<u8> {
        let mut r = rlp::RlpStream::new();
        r.begin_list(1);
        r.append(&bytes);
        r.out()
    }
}
