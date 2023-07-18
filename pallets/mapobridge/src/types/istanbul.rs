use crate::serialization::rlp::{
    big_int_to_rlp_compat_bytes, rlp_field_from_bytes, rlp_to_big_int,
};
use crate::traits::{DefaultFrom, FromBytes};
use crate::{
    slice_as_array_ref,
    types::{common::Address, errors::Kind, header::Header},
};
use num_bigint::BigInt as Integer;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use serde::{Deserialize, Serialize};
use sp_std::vec::Vec;

/// PUBLIC_KEY_LENGTH represents the number of bytes used to represent BLS public key
pub const PUBLIC_KEY_LENGTH: usize = 128;

/// SerializedPublicKey is a public key of a validator that is used to i.e sign the validator set
/// in the header
pub type SerializedPublicKey = [u8; PUBLIC_KEY_LENGTH];

/// G1_PUBLIC_KEY_LENGTH represents the number of bytes used to represent G1 BLS public key
pub const G1_PUBLIC_KEY_LENGTH: usize = 64;

/// SerializedPublicKey is a G1 public key of a validator that is used to i.e sign the validator set
/// in the header
pub type SerializedG1PublicKey = [u8; G1_PUBLIC_KEY_LENGTH];

/// ISTANBUL_EXTRA_VANITY_LENGTH represents the number of bytes used to represent validator vanity
pub const ISTANBUL_EXTRA_VANITY_LENGTH: usize = 32;

/// IstanbulExtraVanity is a portion of extra-data bytes reserved for validator vanity
pub type IstanbulExtraVanity = [u8; ISTANBUL_EXTRA_VANITY_LENGTH];

#[allow(dead_code)]
pub enum IstanbulMsg {
    PrePrepare,
    Prepare,
    Commit,
    RoundChange,
}

/// IstanbulAggregatedSeal contains the aggregated BLS signature created via IBFT consensus
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct IstanbulAggregatedSeal {
    /// Bitmap is a bitmap having an active bit for each validator that signed this block
    #[serde(with = "crate::serialization::bytes::hexbigint")]
    pub bitmap: Integer,

    /// Signature is an aggregated BLS signature resulting from signatures by each validator that signed this block
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub signature: Vec<u8>,

    /// Round is the round in which the signature was created.
    #[serde(with = "crate::serialization::bytes::hexbigint")]
    pub round: Integer,
}

impl IstanbulAggregatedSeal {
    pub fn new() -> Self {
        Self {
            bitmap: Integer::default(),
            signature: Vec::default(),
            round: Integer::default(),
        }
    }
}

impl Encodable for IstanbulAggregatedSeal {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(3);

        // bitmap
        s.append(&big_int_to_rlp_compat_bytes(&self.bitmap));

        // signature
        s.append(&self.signature);

        // round
        s.append(&big_int_to_rlp_compat_bytes(&self.round));
    }
}

impl Decodable for IstanbulAggregatedSeal {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        Ok(IstanbulAggregatedSeal {
            bitmap: rlp_to_big_int(rlp, 0)?,
            signature: rlp.val_at(1)?,
            round: rlp_to_big_int(rlp, 2)?,
        })
    }
}

/// IstanbulExtra represents IBFT consensus state data
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct IstanbulExtra {
    /// The validators that have been added in the block
    #[serde(with = "crate::serialization::bytes::hexvec")]
    pub added_validators: Vec<Address>,

    /// The BLS public keys for the validators added in the block
    #[serde(with = "crate::serialization::bytes::hexvec")]
    pub added_public_keys: Vec<SerializedPublicKey>,

    /// The G1 BLS public keys for the validators added in the block
    #[serde(with = "crate::serialization::bytes::hexvec")]
    pub added_g1_public_keys: Vec<SerializedG1PublicKey>,

    /// Bitmap having an active bit for each removed validator in the block
    #[serde(with = "crate::serialization::bytes::hexbigint")]
    pub removed_validators: Integer,

    /// ECDSA signature by the proposer
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub seal: Vec<u8>,

    /// Contains the aggregated BLS signature created via IBFT consensus
    pub aggregated_seal: IstanbulAggregatedSeal,

    /// Contains and aggregated BLS signature for the previous block
    pub parent_aggregated_seal: IstanbulAggregatedSeal,
}

impl IstanbulExtra {
    pub fn from_rlp(bytes: &[u8]) -> Result<Self, Kind> {
        if bytes.len() < ISTANBUL_EXTRA_VANITY_LENGTH {
            return Err(Kind::RlpDecodeError);
        }

        rlp::decode(&bytes[ISTANBUL_EXTRA_VANITY_LENGTH..]).map_err(|_e| Kind::RlpDecodeError)
    }

    pub fn to_rlp(&self, vanity: &IstanbulExtraVanity) -> Vec<u8> {
        let payload = rlp::encode(self);

        [&vanity[..], &payload[..]].concat()
    }
}

impl Encodable for IstanbulExtra {
    fn rlp_append(&self, s: &mut RlpStream) {
        // added_validators
        s.begin_list(7);
        s.begin_list(self.added_validators.len());
        for address in self.added_validators.iter() {
            s.append(&address.as_ref());
        }

        // added_public_keys
        s.begin_list(self.added_public_keys.len());
        for address in self.added_public_keys.iter() {
            s.append(&address.as_ref());
        }

        // added_g1_public_keys
        s.begin_list(self.added_g1_public_keys.len());
        for address in self.added_g1_public_keys.iter() {
            s.append(&address.as_ref());
        }

        // removed_validators
        s.append(&big_int_to_rlp_compat_bytes(&self.removed_validators));

        // seal
        s.append(&self.seal);

        // aggregated_seal
        s.append(&self.aggregated_seal);

        // parent_aggregated_seal
        s.append(&self.parent_aggregated_seal);
    }
}

impl Decodable for IstanbulExtra {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let added_validators: Result<Vec<Address>, DecoderError> = rlp
            .at(0)?
            .iter()
            .map(|r| rlp_field_from_bytes(&r))
            .collect();

        let added_public_keys: Result<Vec<SerializedPublicKey>, DecoderError> = rlp
            .at(1)?
            .iter()
            .map(|r| rlp_field_from_bytes(&r))
            .collect();

        let added_g1_public_keys: Result<Vec<SerializedG1PublicKey>, DecoderError> = rlp
            .at(2)?
            .iter()
            .map(|r| rlp_field_from_bytes(&r))
            .collect();

        Ok(IstanbulExtra {
            added_validators: added_validators?,
            added_public_keys: added_public_keys?,
            added_g1_public_keys: added_g1_public_keys?,
            removed_validators: rlp_to_big_int(rlp, 3)?,
            seal: rlp.val_at(4)?,
            aggregated_seal: rlp.val_at(5)?,
            parent_aggregated_seal: rlp.val_at(6)?,
        })
    }
}

impl FromBytes for IstanbulExtraVanity {
    fn from_bytes(data: &[u8]) -> Result<&IstanbulExtraVanity, Kind> {
        slice_as_array_ref!(
            &data[..ISTANBUL_EXTRA_VANITY_LENGTH],
            ISTANBUL_EXTRA_VANITY_LENGTH
        )
    }
}

impl FromBytes for SerializedPublicKey {
    fn from_bytes(data: &[u8]) -> Result<&SerializedPublicKey, Kind> {
        slice_as_array_ref!(&data[..PUBLIC_KEY_LENGTH], PUBLIC_KEY_LENGTH)
    }
}

impl DefaultFrom for SerializedPublicKey {
    fn default() -> Self {
        [0; PUBLIC_KEY_LENGTH]
    }
}

impl FromBytes for SerializedG1PublicKey {
    fn from_bytes(data: &[u8]) -> Result<&SerializedG1PublicKey, Kind> {
        slice_as_array_ref!(&data[..G1_PUBLIC_KEY_LENGTH], G1_PUBLIC_KEY_LENGTH)
    }
}

impl DefaultFrom for SerializedG1PublicKey {
    fn default() -> Self {
        [0; G1_PUBLIC_KEY_LENGTH]
    }
}

// Retrieves the block number within an epoch. The return value will be 1-based.
// There is a special case if the number == 0. It is basically the last block of the 0th epoch,
// and should have a value of epoch_size
pub fn get_number_within_epoch(number: u64, epoch_size: u64) -> u64 {
    let number = number % epoch_size;
    if number == 0 {
        epoch_size
    } else {
        number
    }
}

pub fn get_epoch_number(number: u64, epoch_size: u64) -> u64 {
    let epoch_number = number / epoch_size;

    if is_last_block_of_epoch(number, epoch_size) {
        epoch_number
    } else {
        epoch_number + 1
    }
}

pub fn istanbul_filtered_header(header: &Header, keep_seal: bool) -> Result<Header, Kind> {
    let mut new_header = header.clone();

    let mut extra = IstanbulExtra::from_rlp(&new_header.extra)?;
    if !keep_seal {
        extra.seal = Vec::new();
    }
    extra.aggregated_seal = IstanbulAggregatedSeal::new();

    let payload = extra.to_rlp(IstanbulExtraVanity::from_bytes(&new_header.extra)?);
    new_header.extra = payload;

    Ok(new_header)
}

pub fn is_last_block_of_epoch(number: u64, epoch_size: u64) -> bool {
    get_number_within_epoch(number, epoch_size) == epoch_size
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::common::{Bloom, HASH_LENGTH, NONCE_LENGTH};
    use num_traits::Num;

    // tiny example to assert validity of basic data
    const ISTANBUL_EXTRA_TINY: &str = "f7ea9444add0ec310f115a0e603b2d7db9f067778eaf8a94294fc7e8f22b3bcdcf955dd7ff3ba2ed833f8212c0c00c80c3808080c3808080";

    // random example
    const ISTANBUL_EXTRA_DUMPED: &str = "f901caea9444add0ec310f115a0e603b2d7db9f067778eaf8a94294fc7e8f22b3bcdcf955dd7ff3ba2ed833f8212f90104b8800bb4595ae9b572777e51b9bcf510aaeb14ff679bf4ac6e5daa6f24913ea3184112aa4bfaf020de54956689c2124c40fa3f42554261ef58c85920fe8948ab82e60ada74eb113ef4b9f83ba1f4b6e9fc3592295a99a4ed319d840bfae5a70de6850518087f816ba55f0c4c86e5842b46a8727e393c02a85253401155fedd5d9f39b8800d5d25533019444bb7b219d4e32615d19d9a96cf425f6ccf48d0d3944a26575f1d64d5d0b72c5a5f01f90b7f32ada28b70443dd2140a2a1bc903a1e1b16ffd5101f27240efb2ca5383c30d9622160f97800320647591f686a943d61dbdce4646170f3ea07abac534bfad68d255e6e6d9684e168de4fae399dfb2d07c89e185b8f884b8401d384408a5143b5eb1285d59cee6510a13aabf5df43255178fb672924978235009759cb21b84bda2dd7387e218902b6a9d075c4a7c5218194b4f4ac7da2f6bc0b8402e50813415f43ed535ebcc3401487dbc1fdf1de5e3ce9ed4d00b8d502fdd9ee317c2cc0975ed88a58932ceb9a25288983b00ce74f440c146e1477111a1a370910c8401020304c70a84040506070bc3808080";

    #[test]
    fn encodes_istanbul_extra_to_rlp() {
        for extra_bytes in vec![
            prepend_vanity(ISTANBUL_EXTRA_TINY),
            prepend_vanity(ISTANBUL_EXTRA_DUMPED),
        ] {
            let decoded_ist = IstanbulExtra::from_rlp(&extra_bytes).unwrap();

            println!("decoded_ist: {:?}", decoded_ist);
            let vanity = IstanbulExtraVanity::from_bytes(&extra_bytes);
            let encoded_ist_bytes = decoded_ist.to_rlp(vanity.unwrap());

            assert_eq!(encoded_ist_bytes, extra_bytes);
        }
    }

    #[test]
    fn decodes_istanbul_extra_from_rlp() {
        let expected = vec![IstanbulExtra {
            added_validators: to_address_vec(vec![
                "44add0ec310f115a0e603b2d7db9f067778eaf8a",
                "294fc7e8f22b3bcdcf955dd7ff3ba2ed833f8212",
            ]),
            added_public_keys: vec![],
            added_g1_public_keys: vec![],
            removed_validators: Integer::from(12),
            seal: Vec::new(),
            aggregated_seal: IstanbulAggregatedSeal::new(),
            parent_aggregated_seal: IstanbulAggregatedSeal::new(),
        }];

        for (bytes, expected_ist) in vec![prepend_vanity(ISTANBUL_EXTRA_TINY)]
            .iter()
            .zip(expected)
        {
            let parsed = IstanbulExtra::from_rlp(&bytes).unwrap();

            assert_eq!(parsed, expected_ist);
        }
    }

    #[test]
    fn rejects_insufficient_vanity() {
        let bytes = vec![0; ISTANBUL_EXTRA_VANITY_LENGTH - 1];

        assert!(IstanbulExtra::from_rlp(&bytes).is_err());
    }

    #[test]
    fn serializes_and_deserializes_to_json() {
        for bytes in vec![
            prepend_vanity(ISTANBUL_EXTRA_TINY),
            prepend_vanity(&ISTANBUL_EXTRA_DUMPED),
        ]
        .iter()
        {
            let parsed = IstanbulExtra::from_rlp(&bytes).unwrap();
            let json_string = serde_json::to_string(&parsed).unwrap();
            let deserialized_from_json: IstanbulExtra = serde_json::from_str(&json_string).unwrap();

            assert_eq!(parsed, deserialized_from_json);
        }
    }

    fn prepend_vanity(data: &str) -> Vec<u8> {
        let data = hex::decode(data).unwrap();
        let vanity = IstanbulExtraVanity::default();

        [&vanity[..], &data[..]].concat()
    }

    fn to_address_vec(addresses: Vec<&str>) -> Vec<Address> {
        addresses
            .iter()
            .map(|address| {
                Address::from_bytes(hex::decode(address).unwrap().as_slice())
                    .unwrap()
                    .to_owned()
            })
            .collect()
    }

    #[test]
    fn validates_epoch_math() {
        assert_eq!(
            vec![
                get_epoch_number(0, 3),
                get_epoch_number(3, 3),
                get_epoch_number(4, 3)
            ],
            vec![0, 1, 2]
        );
    }

    #[test]
    fn gen_test_data() {
        let mut header = Header {
            parent_hash: to_hash(
                "7285abd5b24742f184ad676e31f6054663b3529bc35ea2fcad8a3e0f642a46f7",
            ),
            coinbase: to_hash("908D0FDaEAEFbb209BDcb540C2891e75616154b3"),
            root: to_hash("ecc60e00b3fe5ce9f6e1a10e5469764daf51f1fe93c22ec3f9a7583a80357217"),
            tx_hash: to_hash("d35d334d87c0cc0a202e3756bf81fae08b1575f286c7ee7a3f8df4f0f3afc55d"),
            receipt_hash: to_hash(
                "56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
            ),
            bloom: Bloom::default(),
            number: Integer::from(1000),
            gas_limit: Default::default(),
            gas_used: Integer::from(0x5208),
            time: Integer::from(0x5c47775c),
            extra: Vec::default(),
            mix_digest: [0; HASH_LENGTH],
            nonce: [0; NONCE_LENGTH],
            base_fee: Default::default(),
        };

        let extra = IstanbulExtra {
            added_validators: to_address_vec(vec![]),
            added_public_keys: vec![],
            added_g1_public_keys: vec![],
            removed_validators: Integer::from(0),
            seal: Vec::new(),
            aggregated_seal: IstanbulAggregatedSeal::new(),
            parent_aggregated_seal: IstanbulAggregatedSeal::new(),
        };

        header.extra = extra.to_rlp(&IstanbulExtraVanity::default());
        let hash = header.hash_without_seal().unwrap();

        println!("{}", serde_json::to_string(&header).unwrap())
    }

    pub fn to_hash<T>(data: &str) -> T
    where
        T: FromBytes + Clone,
    {
        T::from_bytes(&hex::decode(data).unwrap())
            .unwrap()
            .to_owned()
    }

    #[test]
    pub fn test_bitmap() {
        /*
        total validator:  30
        bitmap [33 252 255 15]  big-endian
        validator : 0   0xb4e1BC0856f70A55764FD6B3f8dD27F2162108E9 sign
        validator : 1   0x7A3a26123DBD9CFeFc1725fe7779580B987251Cb sign
        validator : 2   0x7607c9cdd733d8cDA0A644839Ec2bAc5Fa180eD4 sign
        validator : 3   0x65b3FEe569Bf82FF148bddEd9c3793FB685f9333 sign
        validator : 4   0xa9c044C42591C57362315130A0E9a80f7C3A0C2C nosign
        validator : 5   0x8D3925c8fe63Ab483F113a6A6f52d09e02EC7d47 nosign
        validator : 6   0x171Cea72aED36C6bCb51a5b915646e1f7aA6AC7f nosign
        validator : 7   0x069633a9ADEaEd7A17ff78B24f2729503cff6C90 nosign
        validator : 8   0xce5fC472c7B7D36c14043862FDce03402f2925Eb sign
        validator : 9   0x01734524cC07A49a1237b950D72cAae98D946763 sign
        validator : 10   0x48bcEB30BC96afA7F659a6Fb6BD8d1127a89e8E9 sign
        validator : 11   0x37629d8d17F8cA81a003f76b750207417EED19fa sign
        validator : 12   0x9e5665ACb994906c55bda5F18EC33f05012BB46a sign
        validator : 13   0x8Df2a3A3fE8f1564E8f1F24572a3D69153fbF99E sign
        validator : 14   0x4a032839455E7616054f38aDE3043Fa447a099Bf sign
        validator : 15   0x59FF5a050c17BD2D191f4eB24b27afB50DC5EEdc sign
        validator : 16   0xE4f125a24C0591795Cc880852a8aBc8395275939 nosign
        validator : 17   0x2fE1E683A4A9e89CA9bf66F558d9019CE20060a4 nosign
        validator : 18   0x2048a5809a027537C2Ffb1B0e957CC8b5F598D5b sign
        validator : 19   0x6bE1F2DcDB6a846C9Df353aA77F622bc859574aB sign
        validator : 20   0xEd846b40803E96bd8Abf561D6fC697be1AEd9eB1 sign
        validator : 21   0xEBbFd202a69eC17889043Ab8f5922a9488f464eA sign
        validator : 22   0xc39Ef66E5707ff7C8Fdb5524Aaa375A25652faCD sign
        validator : 23   0x8A213d60FF5F32d74AAe8116d5A266c50873b676 sign
        validator : 24   0xe498d13527524132736d804C42a288d72A979e46 sign
        validator : 25   0x7d32c326ebfC9bFf2908edaF3B02dA543044e4C2 nosign
        validator : 26   0xD98dC429cDC70FE3260Bf6dECA806a3b8e0FC7fA nosign
        validator : 27   0x2dbb721077CFB8b9D18950175dDAE75924BDD1be nosign
        validator : 28   0xDbBd0434BF5A280b19eFB4D4D740dA1E4Af5e9ee nosign
        validator : 29   0xff5B060b0e6595890Cc6eCDF669997314068333f sign
         all signer is 20
         */
        let data = "d683010000846765746886676f312e3139856c696e7578000000000000000000f8dbc0c0c080b8419a4d130b0b47e805e343c2838a48923f1144fe42b2b172c2b5149f70a4066a951e8338adebfb1ea09105e81c4632703225618349ce9697e64ac7cdc6173429b501f8488421fcff0fb84006c928b279c6a75b1ff26fc82a46e1b7a92cd2e298e2edfbcfaafee94ce5b561019ff2d1b934c8200889474e217ca2941da7fa21da6108ebd10e7f3629b1fff580f8488431fdff0fb8401a4d4a81399c1bb275e4e80168bcbfc9fd763eea7b3bc20de403b7f9759eced108d5789ab574267a4d69330a32518fc8537302d2a168360ee5cb5f55fe3981af80";
        let extra = IstanbulExtra::from_rlp(hex::decode(data).unwrap().as_slice()).unwrap();
        let bitmap = extra.aggregated_seal.bitmap;

        assert!(bitmap.bit(0));
        assert!(bitmap.bit(1));
        assert!(bitmap.bit(2));
        assert!(bitmap.bit(3));
        assert!(!bitmap.bit(4));
        assert!(!bitmap.bit(5));
        assert!(!bitmap.bit(6));
        assert!(!bitmap.bit(7));
        assert!(bitmap.bit(8));
        assert!(bitmap.bit(9));
        assert!(bitmap.bit(10));
        assert!(bitmap.bit(11));
        assert!(bitmap.bit(12));
        assert!(bitmap.bit(13));
        assert!(bitmap.bit(14));
        assert!(bitmap.bit(15));
        assert!(!bitmap.bit(16));
        assert!(!bitmap.bit(17));
        assert!(bitmap.bit(18));
        assert!(bitmap.bit(19));
        assert!(bitmap.bit(20));
        assert!(bitmap.bit(21));
        assert!(bitmap.bit(22));
        assert!(bitmap.bit(23));
        assert!(bitmap.bit(24));
        assert!(!bitmap.bit(25));
        assert!(!bitmap.bit(26));
        assert!(!bitmap.bit(27));
        assert!(!bitmap.bit(28));
        assert!(bitmap.bit(29));
    }
}
