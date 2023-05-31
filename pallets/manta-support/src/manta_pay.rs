// Copyright 2020-2023 Manta Network.
// This file is part of Manta.
//
// Manta is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Manta is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Manta.  If not, see <http://www.gnu.org/licenses/>.

//! Type Definitions for Manta Pay

use alloc::{boxed::Box, vec::Vec};
use core::ops::Add;
use frame_support::sp_runtime::traits::Zero;
use manta_pay::{
    config::{
        self,
        utxo::{self, MerkleTreeConfiguration},
    },
    crypto::poseidon::encryption::{self, BlockArray, CiphertextBlock},
    manta_crypto::{
        encryption::{hybrid, EmptyHeader},
        merkle_tree,
        permutation::duplex,
        signature::schnorr,
    },
};
use manta_util::{Array, BoxArray};
use scale_codec::{Decode, Encode, Error, MaxEncodedLen};
use scale_info::TypeInfo;

#[cfg(feature = "rpc")]
use manta_pay::manta_util::serde::{Deserialize, Serialize};

use manta_crypto::arkworks::{
    algebra::Group as CryptoGroup,
    constraint::fp::Fp,
    ec::{PairingEngine, ProjectiveCurve},
    groth16::Proof as CryptoProof,
};
pub use manta_pay::config::utxo::Checkpoint;
use manta_util::{codec, into_array_unchecked};

/// Standard Asset Id
pub type StandardAssetId = u128;

///
#[inline]
pub fn id_from_field(id: [u8; 32]) -> Option<StandardAssetId> {
    if 0u128.to_le_bytes() == id[16..32] {
        Some(u128::from_le_bytes(
            Array::from_iter(id[0..16].iter().copied()).into(),
        ))
    } else {
        None
    }
}

///
#[inline]
pub fn field_from_id(id: StandardAssetId) -> [u8; 32] {
    into_array_unchecked([id.to_le_bytes(), [0; 16]].concat())
}

/// Decodes the `bytes` array of the given length `N` into the SCALE decodable type `T` returning a
/// blanket error if decoding fails.
#[inline]
pub(crate) fn decode<T, const N: usize>(bytes: [u8; N]) -> Result<T, Error>
where
    T: Decode,
{
    T::decode(&mut bytes.as_slice())
}

pub const FP_ENCODE: &str = "Fp encoding to [u8; 32] failed.";
pub const FP_DECODE: &str = "Vec<u8>(u8; 32) decoding to Fp failed.";
pub const GROUP_ENCODE: &str = "Group encoding to [u8; 32] failed.";
pub const GROUP_DECODE: &str = "Vec<u8>(u8; 32) decoding to Group failed.";
pub const PROOF_ENCODE: &str = "Proof encoding to [u8; 128] failed.";
pub const PROOF_DECODE: &str = "Vec<u8>(u8; 128) decoding to Proof failed.";

/// Field encode to byte array
pub fn fp_encode<T>(fp: Fp<T>) -> Result<[u8; 32], scale_codec::Error>
where
    T: manta_crypto::arkworks::ff::Field,
{
    use manta_util::codec::Encode;
    fp.to_vec()
        .try_into()
        .map_err(|_e| scale_codec::Error::from(FP_ENCODE))
}

/// Field decode from byte array
pub fn fp_decode<T>(fp_bytes: Vec<u8>) -> Result<Fp<T>, scale_codec::Error>
where
    T: manta_crypto::arkworks::ff::Field,
{
    Fp::try_from(fp_bytes).map_err(|_e| scale_codec::Error::from(FP_DECODE))
}

/// Group encode to byte array
pub fn group_encode<T>(group: CryptoGroup<T>) -> Result<[u8; 32], scale_codec::Error>
where
    T: ProjectiveCurve,
{
    use manta_util::codec::Encode;
    group
        .to_vec()
        .try_into()
        .map_err(|_e| scale_codec::Error::from(GROUP_ENCODE))
}

/// Group decode from byte array
pub fn group_decode<T>(group_bytes: Vec<u8>) -> Result<CryptoGroup<T>, scale_codec::Error>
where
    T: ProjectiveCurve,
{
    CryptoGroup::try_from(group_bytes).map_err(|_e| scale_codec::Error::from(GROUP_DECODE))
}

/// Proof encode to byte array
pub fn proof_encode<T>(proof: CryptoProof<T>) -> Result<[u8; 128], scale_codec::Error>
where
    T: PairingEngine,
{
    use manta_util::codec::Encode;
    let bytes = proof.to_vec();
    // The first 8 bytes of the serialization are a meaningless header, so we remove them.
    let u128_bytes = &bytes[8..];
    u128_bytes
        .to_vec()
        .try_into()
        .map_err(|_e| scale_codec::Error::from(PROOF_ENCODE))
}

/// Proof decode from byte array
pub fn proof_decode<T>(proof_bytes: Vec<u8>) -> Result<CryptoProof<T>, scale_codec::Error>
where
    T: PairingEngine,
{
    CryptoProof::try_from(proof_bytes).map_err(|_e| scale_codec::Error::from(PROOF_DECODE))
}

/// AssetValue(u128) to byte array [u8; 16]
pub fn asset_value_encode(asset_value: AssetValue) -> [u8; 16] {
    asset_value.to_le_bytes()
}

/// Byte array [u8; 16] to AssetValue(u128)
pub fn asset_value_decode(bytes: [u8; 16]) -> AssetValue {
    u128::from_le_bytes(bytes)
}

///
pub const TAG_LENGTH: usize = 32;

/// Tag Type
pub type Tag = [u8; TAG_LENGTH];

///
pub const SCALAR_LENGTH: usize = 32;

/// Scalar Type
pub type Scalar = [u8; SCALAR_LENGTH];

///
pub const GROUP_LENGTH: usize = 32;

/// Group Type
pub type Group = [u8; GROUP_LENGTH];

///
pub const UTXO_COMMITMENT_LENGTH: usize = 32;

/// UTXO Commitment Type
pub type UtxoCommitment = [u8; UTXO_COMMITMENT_LENGTH];

///
pub const NULLIFIER_COMMITMENT_LENGTH: usize = 32;

/// Nullifier Commitment Type
pub type NullifierCommitment = [u8; NULLIFIER_COMMITMENT_LENGTH];

///
pub const UTXO_ACCUMULATOR_OUTPUT_LENGTH: usize = 32;

/// UTXO Accumulator Output Type
pub type UtxoAccumulatorOutput = [u8; UTXO_ACCUMULATOR_OUTPUT_LENGTH];

/// Compressed size of 2 g1 curve points + 1 g2 curve point
/// A, C from g1 curve, B from g2 curve
pub const PROOF_LENGTH: usize = 128;

/// Transfer Proof Type
pub type Proof = [u8; PROOF_LENGTH];

/// Asset Identifier Type
pub type AssetId = [u8; 32];

/// Asset Value Type
pub type AssetValue = u128;

/// Account Identifier Type
pub type AccountId = [u8; 32];

/// Transfer Proof encoded value
/// Compatability for JS u128 and Encode/Decode from parity_scale_codec
pub type EncodedAssetValue = [u8; 16];

/// Asset
#[cfg_attr(
    feature = "rpc",
    derive(Deserialize, Serialize),
    serde(crate = "manta_util::serde", deny_unknown_fields)
)]
#[derive(
    Clone,
    Copy,
    Debug,
    Decode,
    Default,
    Encode,
    Eq,
    Hash,
    MaxEncodedLen,
    Ord,
    PartialEq,
    PartialOrd,
    TypeInfo,
)]
pub struct Asset {
    /// Asset Id
    pub id: AssetId,

    /// Asset Value
    pub value: EncodedAssetValue,
}

impl Asset {
    /// Builds a new [`Asset`] from `id` and `value`.
    #[inline]
    pub fn new(id: AssetId, value: EncodedAssetValue) -> Self {
        Self { id, value }
    }
}

impl Zero for Asset {
    fn zero() -> Self {
        Self {
            id: [0u8; 32],
            value: [0u8; 16],
        }
    }

    fn is_zero(&self) -> bool {
        let zero_asset = Self {
            id: [0u8; 32],
            value: [0u8; 16],
        };
        *self == zero_asset
    }
}

impl Add for Asset {
    type Output = Asset;

    fn add(self, _rhs: Asset) -> Self::Output {
        Self {
            id: [0u8; 32],
            value: [0u8; 16],
        }
    }
}

impl TryFrom<config::Asset> for Asset {
    type Error = Error;

    #[inline]
    fn try_from(asset: config::Asset) -> Result<Self, Error> {
        Ok(Self {
            id: fp_encode(asset.id)?,
            value: asset_value_encode(asset.value),
        })
    }
}

impl TryFrom<Asset> for config::Asset {
    type Error = Error;

    #[inline]
    fn try_from(asset: Asset) -> Result<Self, Self::Error> {
        Ok(Self {
            id: fp_decode(asset.id.to_vec())?,
            value: asset_value_decode(asset.value),
        })
    }
}

/// AssetId and (AssetValue + AESTag)
pub const OUTGOING_CIPHER_TEXT_COMPONENTS_COUNT: usize = 2;
/// AssetId is BN254 field element, so 32 bytes
/// AssetValue is u128 so 16 bytes and AESTag is 16 bytes, so combined is 32 bytes
pub const OUTGOING_CIPHER_TEXT_COMPONENT_SIZE: usize = 32;
/// Outgoing Ciphertext
pub type OutgoingCiphertext =
    [[u8; OUTGOING_CIPHER_TEXT_COMPONENT_SIZE]; OUTGOING_CIPHER_TEXT_COMPONENTS_COUNT];

/// Outgoing Note
#[cfg_attr(
    feature = "rpc",
    derive(Deserialize, Serialize),
    serde(crate = "manta_util::serde", deny_unknown_fields)
)]
#[derive(Clone, Debug, Decode, Default, Encode, Eq, Hash, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct OutgoingNote {
    /// Ephemeral Public Key
    pub ephemeral_public_key: Group,

    /// Ciphertext
    pub ciphertext: OutgoingCiphertext,
}

impl TryFrom<utxo::OutgoingNote> for OutgoingNote {
    type Error = Error;

    #[inline]
    fn try_from(note: utxo::OutgoingNote) -> Result<Self, Error> {
        let encoded = note.ciphertext.ciphertext.encode();
        let mut encoded_ciphertext =
            [[0u8; OUTGOING_CIPHER_TEXT_COMPONENT_SIZE]; OUTGOING_CIPHER_TEXT_COMPONENTS_COUNT];
        for (outer_ind, array) in encoded_ciphertext.into_iter().enumerate() {
            for (inner_ind, _) in array.into_iter().enumerate() {
                encoded_ciphertext[outer_ind][inner_ind] = encoded[outer_ind * 32 + inner_ind];
            }
        }
        Ok(Self {
            ephemeral_public_key: group_encode(note.ciphertext.ephemeral_public_key)?,
            ciphertext: encoded_ciphertext,
        })
    }
}

impl TryFrom<OutgoingNote> for utxo::OutgoingNote {
    type Error = Error;

    #[inline]
    fn try_from(note: OutgoingNote) -> Result<Self, Self::Error> {
        let mut flat_outgoing_ciphertext =
            [0u8; OUTGOING_CIPHER_TEXT_COMPONENT_SIZE * OUTGOING_CIPHER_TEXT_COMPONENTS_COUNT];
        let mut index = 0;
        for component in note.ciphertext {
            for byte in component {
                flat_outgoing_ciphertext[index as usize] = byte;
                index += 1;
            }
        }
        let decoded_outgoing_ciphertext: [u8; 64] = decode(flat_outgoing_ciphertext)?;
        Ok(Self {
            header: EmptyHeader::default(),
            ciphertext: hybrid::Ciphertext {
                ephemeral_public_key: group_decode(note.ephemeral_public_key.to_vec())?,
                ciphertext: decoded_outgoing_ciphertext.into(),
            },
        })
    }
}

/// Sender Post
#[derive(Clone, Debug, Decode, Encode, Eq, Hash, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct SenderPost {
    /// UTXO Accumulator Output
    pub utxo_accumulator_output: UtxoAccumulatorOutput,

    /// Nullifier Commitment
    pub nullifier_commitment: NullifierCommitment,

    /// Outgoing Note
    pub outgoing_note: OutgoingNote,
}

impl TryFrom<config::SenderPost> for SenderPost {
    type Error = Error;

    #[inline]
    fn try_from(post: config::SenderPost) -> Result<Self, Error> {
        Ok(Self {
            utxo_accumulator_output: fp_encode(post.utxo_accumulator_output)?,
            nullifier_commitment: fp_encode(post.nullifier.nullifier.commitment)?,
            outgoing_note: TryFrom::try_from(post.nullifier.outgoing_note)?,
        })
    }
}

impl TryFrom<SenderPost> for config::SenderPost {
    type Error = Error;

    #[inline]
    fn try_from(post: SenderPost) -> Result<Self, Self::Error> {
        Ok(Self {
            utxo_accumulator_output: fp_decode(post.utxo_accumulator_output.to_vec())?,
            nullifier: config::Nullifier {
                nullifier: manta_accounting::transfer::utxo::protocol::Nullifier {
                    commitment: fp_decode(post.nullifier_commitment.to_vec())?,
                },
                outgoing_note: TryFrom::try_from(post.outgoing_note)?,
            },
        })
    }
}

/// AssetId and (AssetValue + AESTag) and UTXORandomness
pub const INCOMING_CIPHER_TEXT_COMPONENTS_COUNT: usize = 3;
/// AssetId is BN254 field element, so 32 bytes
/// AssetValue is u128 so 16 bytes and AESTag is 16 bytes, so combined is 32 bytes
/// UTXORandomness is 32 bytes
pub const INCOMING_CIPHER_TEXT_COMPONENT_SIZE: usize = 32;
/// Incoming Ciphertext Type
pub type IncomingCiphertext =
    [[u8; INCOMING_CIPHER_TEXT_COMPONENT_SIZE]; INCOMING_CIPHER_TEXT_COMPONENTS_COUNT];
/// Light Incoming Ciphertext Type
pub type LightIncomingCiphertext =
    [[u8; INCOMING_CIPHER_TEXT_COMPONENT_SIZE]; INCOMING_CIPHER_TEXT_COMPONENTS_COUNT];

/// Incoming Note
#[cfg_attr(
    feature = "rpc",
    derive(Deserialize, Serialize),
    serde(crate = "manta_util::serde", deny_unknown_fields)
)]
#[derive(Clone, Debug, Decode, Default, Encode, Eq, Hash, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct IncomingNote {
    /// Ephemeral Public Key
    pub ephemeral_public_key: Group,

    /// Tag
    pub tag: Tag,

    /// Ciphertext
    pub ciphertext: IncomingCiphertext,
}

impl TryFrom<utxo::IncomingNote> for IncomingNote {
    type Error = Error;

    #[inline]
    fn try_from(note: utxo::IncomingNote) -> Result<Self, Error> {
        Ok(Self {
            ephemeral_public_key: group_encode(note.ciphertext.ephemeral_public_key)?,
            tag: fp_encode(note.ciphertext.ciphertext.tag.0)?,
            ciphertext: Array::from_iter(
                note.ciphertext.ciphertext.message[0]
                    .0
                    .iter()
                    .map(|fp| fp_encode(*fp))
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .into(),
        })
    }
}

impl TryFrom<IncomingNote> for utxo::IncomingNote {
    type Error = Error;

    #[inline]
    fn try_from(note: IncomingNote) -> Result<Self, Self::Error> {
        Ok(Self {
            header: EmptyHeader::default(),
            ciphertext: hybrid::Ciphertext {
                ephemeral_public_key: group_decode(note.ephemeral_public_key.to_vec())?,
                ciphertext: duplex::Ciphertext {
                    tag: encryption::Tag(fp_decode(note.tag.to_vec())?),
                    message: BlockArray(BoxArray(Box::new([CiphertextBlock(
                        note.ciphertext
                            .into_iter()
                            .map(|x| fp_decode(x.to_vec()))
                            .collect::<Result<Vec<_>, _>>()?
                            .into(),
                    )]))),
                },
            },
        })
    }
}

/// Incoming Note
#[cfg_attr(
    feature = "rpc",
    derive(Deserialize, Serialize),
    serde(crate = "manta_util::serde", deny_unknown_fields)
)]
#[derive(Clone, Debug, Decode, Default, Encode, Eq, Hash, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct LightIncomingNote {
    /// Ephemeral Public Key
    pub ephemeral_public_key: Group,

    /// Ciphertext
    pub ciphertext: LightIncomingCiphertext,
}

impl TryFrom<utxo::LightIncomingNote> for LightIncomingNote {
    type Error = Error;

    #[inline]
    fn try_from(note: utxo::LightIncomingNote) -> Result<Self, Error> {
        let encoded = note.ciphertext.ciphertext.encode();
        let mut encoded_arrays =
            [[0u8; INCOMING_CIPHER_TEXT_COMPONENT_SIZE]; INCOMING_CIPHER_TEXT_COMPONENTS_COUNT];
        for (outer_ind, array) in encoded_arrays.into_iter().enumerate() {
            for (inner_ind, _) in array.into_iter().enumerate() {
                encoded_arrays[outer_ind][inner_ind] =
                    encoded[outer_ind * INCOMING_CIPHER_TEXT_COMPONENT_SIZE + inner_ind];
            }
        }
        Ok(Self {
            ephemeral_public_key: group_encode(note.ciphertext.ephemeral_public_key)?,
            ciphertext: encoded_arrays,
        })
    }
}

impl TryFrom<LightIncomingNote> for utxo::LightIncomingNote {
    type Error = Error;

    #[inline]
    fn try_from(note: LightIncomingNote) -> Result<Self, Self::Error> {
        let mut encoded_incoming_ciphertext =
            [0u8; INCOMING_CIPHER_TEXT_COMPONENT_SIZE * INCOMING_CIPHER_TEXT_COMPONENTS_COUNT];
        let mut ind = 0;
        for component in note.ciphertext {
            for byte in component {
                encoded_incoming_ciphertext[ind as usize] = byte;
                ind += 1;
            }
        }
        let decoded_incoming_ciphertext: [u8; 96] = decode(encoded_incoming_ciphertext)?;
        Ok(Self {
            header: EmptyHeader::default(),
            ciphertext: hybrid::Ciphertext {
                ephemeral_public_key: group_decode(note.ephemeral_public_key.to_vec())?,
                ciphertext: decoded_incoming_ciphertext.into(),
            },
        })
    }
}

/// Full Incoming Note
#[cfg_attr(
    feature = "rpc",
    derive(Deserialize, Serialize),
    serde(crate = "manta_util::serde", deny_unknown_fields)
)]
#[derive(Clone, Debug, Decode, Default, Encode, Eq, Hash, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct FullIncomingNote {
    /// Address Partition
    pub address_partition: u8,

    /// Incoming Note
    pub incoming_note: IncomingNote,

    pub light_incoming_note: LightIncomingNote,
}

impl TryFrom<utxo::FullIncomingNote> for FullIncomingNote {
    type Error = Error;

    #[inline]
    fn try_from(note: utxo::FullIncomingNote) -> Result<Self, Error> {
        Ok(Self {
            address_partition: note.address_partition,
            incoming_note: IncomingNote::try_from(note.incoming_note)?,
            light_incoming_note: LightIncomingNote::try_from(note.light_incoming_note)?,
        })
    }
}

impl TryFrom<FullIncomingNote> for utxo::FullIncomingNote {
    type Error = Error;

    #[inline]
    fn try_from(note: FullIncomingNote) -> Result<Self, Self::Error> {
        Ok(Self {
            address_partition: note.address_partition,
            incoming_note: note.incoming_note.try_into()?,
            light_incoming_note: note.light_incoming_note.try_into()?,
        })
    }
}

/// UTXO
#[cfg_attr(
    feature = "rpc",
    derive(Deserialize, Serialize),
    serde(crate = "manta_util::serde", deny_unknown_fields)
)]
#[derive(
    Clone, Copy, Debug, Decode, Default, Encode, Eq, Hash, MaxEncodedLen, PartialEq, TypeInfo,
)]
pub struct Utxo {
    /// Transparency Flag
    pub is_transparent: bool,

    /// Public Asset
    pub public_asset: Asset,

    /// UTXO Commitment
    pub commitment: UtxoCommitment,
}

impl Utxo {
    ///
    #[inline]
    pub fn try_from(utxo: utxo::Utxo) -> Result<Utxo, Error> {
        Ok(Self {
            is_transparent: utxo.is_transparent,
            public_asset: utxo.public_asset.try_into()?,
            commitment: fp_encode(utxo.commitment)?,
        })
    }

    ///
    #[inline]
    pub fn try_into(self) -> Result<utxo::Utxo, Error> {
        Ok(utxo::Utxo {
            is_transparent: self.is_transparent,
            public_asset: self.public_asset.try_into()?,
            commitment: fp_decode(self.commitment.to_vec())?,
        })
    }
}

/// Receiver Post
#[derive(Clone, Debug, Decode, Encode, Eq, Hash, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct ReceiverPost {
    /// Unspent Transaction Output
    pub utxo: Utxo,

    /// Full Incoming Note
    pub full_incoming_note: FullIncomingNote,
}

impl TryFrom<config::ReceiverPost> for ReceiverPost {
    type Error = Error;

    #[inline]
    fn try_from(post: config::ReceiverPost) -> Result<Self, Error> {
        Ok(Self {
            utxo: Utxo::try_from(post.utxo)?,
            full_incoming_note: FullIncomingNote::try_from(post.note)?,
        })
    }
}

impl TryFrom<ReceiverPost> for config::ReceiverPost {
    type Error = Error;

    #[inline]
    fn try_from(post: ReceiverPost) -> Result<Self, Self::Error> {
        Ok(Self {
            utxo: post.utxo.try_into()?,
            note: post.full_incoming_note.try_into()?,
        })
    }
}

/// Authorization Signature
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq, TypeInfo)]
pub struct AuthorizationSignature {
    /// Authorization Key
    pub authorization_key: Group,

    /// Signature
    pub signature: (Scalar, Group),
}

impl TryFrom<utxo::AuthorizationSignature> for AuthorizationSignature {
    type Error = Error;

    #[inline]
    fn try_from(signature: utxo::AuthorizationSignature) -> Result<Self, Error> {
        Ok(Self {
            authorization_key: group_encode(signature.authorization_key)?,
            signature: (
                fp_encode(signature.signature.scalar)?,
                group_encode(signature.signature.nonce_point)?,
            ),
        })
    }
}

impl TryFrom<AuthorizationSignature> for utxo::AuthorizationSignature {
    type Error = Error;

    #[inline]
    fn try_from(signature: AuthorizationSignature) -> Result<Self, Self::Error> {
        Ok(Self {
            authorization_key: group_decode(signature.authorization_key.to_vec())?,
            signature: schnorr::Signature {
                scalar: fp_decode(signature.signature.0.to_vec())?,
                nonce_point: group_decode(signature.signature.1.to_vec())?,
            },
        })
    }
}

/// Transfer Post
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq, TypeInfo)]
pub struct TransferPost {
    /// Authorization Signature
    pub authorization_signature: Option<AuthorizationSignature>,

    /// Asset Id
    pub asset_id: Option<AssetId>,

    /// Sources
    /// Using EncodedAssetValue as JS/JSON does not handle u128 well
    pub sources: Vec<EncodedAssetValue>,

    /// Sender Posts
    pub sender_posts: Vec<SenderPost>,

    /// Receiver Posts
    pub receiver_posts: Vec<ReceiverPost>,

    /// Sinks
    pub sinks: Vec<EncodedAssetValue>,

    /// Proof
    pub proof: Proof,

    /// Sink Accounts
    pub sink_accounts: Vec<AccountId>,
}

impl TransferPost {
    /// Constructs an [`Asset`] against the `asset_id` of `self` and `value`.
    #[inline]
    fn construct_asset(&self, value: &EncodedAssetValue) -> Option<Asset> {
        Some(Asset::new(self.asset_id?, *value))
    }

    /// Returns the `k`-th source in the transfer.
    #[inline]
    pub fn source(&self, k: usize) -> Option<Asset> {
        self.sources
            .get(k)
            .and_then(|value| self.construct_asset(value))
    }

    /// Returns the `k`-th sink in the transfer.
    #[inline]
    pub fn sink(&self, k: usize) -> Option<Asset> {
        self.sinks
            .get(k)
            .and_then(|value| self.construct_asset(value))
    }
}

impl TryFrom<config::TransferPost> for TransferPost {
    type Error = Error;

    #[inline]
    fn try_from(post: config::TransferPost) -> Result<Self, Error> {
        let authorization_signature = post
            .authorization_signature
            .map(TryInto::try_into)
            .map_or(Ok(None), |r| r.map(Some))?;
        let asset_id = post
            .body
            .asset_id
            .map(fp_encode)
            .map_or(Ok(None), |r| r.map(Some))?;
        let sources = post
            .body
            .sources
            .into_iter()
            .map(|v| Ok::<[u8; 16], Self::Error>(v.to_le_bytes()))
            .collect::<Result<_, _>>()?;
        let sender_posts = post
            .body
            .sender_posts
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()?;
        let receiver_posts = post
            .body
            .receiver_posts
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()?;
        let sinks = post
            .body
            .sinks
            .into_iter()
            .map(|v| Ok::<[u8; 16], Self::Error>(v.to_le_bytes()))
            .collect::<Result<_, _>>()?;
        let proof = proof_encode(post.body.proof)?;
        let sink_accounts = post.sink_accounts;
        Ok(Self {
            authorization_signature,
            asset_id,
            sources,
            sender_posts,
            receiver_posts,
            sinks,
            proof,
            sink_accounts,
        })
    }
}

impl TryFrom<TransferPost> for config::TransferPost {
    type Error = Error;

    #[inline]
    fn try_from(post: TransferPost) -> Result<Self, Self::Error> {
        let proof = proof_decode(post.proof.to_vec())?;
        Ok(Self {
            authorization_signature: post
                .authorization_signature
                .map(TryInto::try_into)
                .transpose()?,
            body: config::TransferPostBody {
                asset_id: post.asset_id.map(|x| fp_decode(x.to_vec())).transpose()?,
                sources: post.sources.into_iter().map(u128::from_le_bytes).collect(),
                sender_posts: post
                    .sender_posts
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
                receiver_posts: post
                    .receiver_posts
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
                sinks: post.sinks.into_iter().map(u128::from_le_bytes).collect(),
                proof,
            },
            sink_accounts: post.sink_accounts,
        })
    }
}

/// Leaf Digest Type
pub type LeafDigest = [u8; 32];

/// Inner Digest Type
pub type InnerDigest = [u8; 32];

/// Merkle Tree Current Path
#[cfg_attr(
    feature = "serde",
    derive(Deserialize, Serialize),
    serde(crate = "manta_util::serde", deny_unknown_fields)
)]
#[derive(Clone, Debug, Decode, Default, Encode, Eq, Hash, PartialEq, TypeInfo)]
pub struct CurrentPath {
    /// Sibling Digest
    pub sibling_digest: LeafDigest,

    /// Leaf Index
    pub leaf_index: u32,

    /// Inner Path
    pub inner_path: Vec<InnerDigest>,
}

impl MaxEncodedLen for CurrentPath {
    #[inline]
    fn max_encoded_len() -> usize {
        0_usize
            .saturating_add(<LeafDigest>::max_encoded_len())
            .saturating_add(u32::max_encoded_len())
            .saturating_add(
                // NOTE: We know that these paths don't exceed the path length.
                <InnerDigest>::max_encoded_len().saturating_mul(
                    manta_crypto::merkle_tree::path_length::<MerkleTreeConfiguration, ()>(),
                ),
            )
    }
}

impl TryFrom<merkle_tree::CurrentPath<MerkleTreeConfiguration>> for CurrentPath {
    type Error = Error;

    #[inline]
    fn try_from(path: merkle_tree::CurrentPath<MerkleTreeConfiguration>) -> Result<Self, Error> {
        Ok(Self {
            sibling_digest: fp_encode(path.sibling_digest)?,
            leaf_index: path.inner_path.leaf_index.0 as u32,
            inner_path: path
                .inner_path
                .path
                .into_iter()
                .map(fp_encode)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl TryFrom<CurrentPath> for merkle_tree::CurrentPath<MerkleTreeConfiguration> {
    type Error = Error;

    #[inline]
    fn try_from(path: CurrentPath) -> Result<Self, Error> {
        Ok(Self::new(
            fp_decode(path.sibling_digest.to_vec())?,
            (path.leaf_index as usize).into(),
            path.inner_path
                .into_iter()
                .map(|x| fp_decode(x.to_vec()))
                .collect::<Result<_, _>>()?,
        ))
    }
}

/// UTXO Merkle Tree Path
#[derive(Clone, Debug, Decode, Default, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct UtxoMerkleTreePath {
    /// Current Leaf Digest
    pub leaf_digest: Option<LeafDigest>,

    /// Current Path
    pub current_path: CurrentPath,
}

/// Receiver Chunk Data Type
pub type ReceiverChunk = Vec<(Utxo, FullIncomingNote)>;

/// Utxo Chunk Data Type
pub type UtxoChunk = Vec<Utxo>;

/// Merkle Tree [`CurrentPath`] Chunk Data Type
pub type CurrentPathChunk = Vec<CurrentPath>;

/// Sender Chunk Data Type
pub type SenderChunk = Vec<(NullifierCommitment, OutgoingNote)>;

/// Initial Sync Response
#[cfg_attr(
    feature = "serde",
    derive(Deserialize, Serialize),
    serde(crate = "manta_util::serde", deny_unknown_fields)
)]
#[derive(Clone, Debug, Decode, Default, Encode, Eq, Hash, PartialEq, TypeInfo)]
pub struct InitialSyncResponse {
    /// Initial Sync Continuation Flag
    ///
    /// The `should_continue` flag is set to `true` if the client should request more data from the
    /// ledger to finish the pull.
    pub should_continue: bool,

    /// Ledger Utxo Chunk
    pub utxo_data: UtxoChunk,

    /// Ledger [`CurrentPath`] Chunk
    pub membership_proof_data: CurrentPathChunk,

    /// Nullifier Count
    pub nullifier_count: u128,
}

/// Ledger Source Dense Pull Response
#[cfg_attr(
    feature = "serde",
    derive(Deserialize, Serialize),
    serde(crate = "manta_util::serde", deny_unknown_fields)
)]
#[derive(Clone, Debug, Encode, Default, Eq, Hash, Decode, PartialEq, TypeInfo)]
pub struct DenseInitialSyncResponse {
    /// Pull Continuation Flag
    ///
    /// The `should_continue` flag is set to `true` if the client should request more data from the
    /// ledger to finish the pull.
    pub should_continue: bool,

    /// Ledger Utxo Chunk
    #[codec(skip)]
    pub utxo_data: alloc::string::String,

    /// Ledger [`CurrentPath`] Chunk
    #[codec(skip)]
    pub membership_proof_data: alloc::string::String,

    /// Nullifier Count
    pub nullifier_count: u128,
}

impl From<InitialSyncResponse> for DenseInitialSyncResponse {
    #[inline]
    fn from(resp: InitialSyncResponse) -> DenseInitialSyncResponse {
        Self {
            should_continue: resp.should_continue,
            utxo_data: base64::encode(resp.utxo_data.encode()),
            membership_proof_data: base64::encode(resp.membership_proof_data.encode()),
            nullifier_count: resp.nullifier_count,
        }
    }
}

/// Ledger Source Pull Response
#[cfg_attr(
    feature = "serde",
    derive(Deserialize, Serialize),
    serde(crate = "manta_util::serde", deny_unknown_fields)
)]
#[derive(Clone, Debug, Decode, Default, Encode, Eq, Hash, PartialEq, TypeInfo)]
pub struct PullResponse {
    /// Pull Continuation Flag
    ///
    /// The `should_continue` flag is set to `true` if the client should request more data from the
    /// ledger to finish the pull.
    pub should_continue: bool,

    /// Ledger Receiver Chunk
    pub receivers: ReceiverChunk,

    /// Ledger Sender Chunk
    pub senders: SenderChunk,

    /// Total Number of Senders/Receivers in Ledger
    pub senders_receivers_total: [u8; 16],
}

/// Ledger Source Dense Pull Response
#[cfg_attr(
    feature = "serde",
    derive(Deserialize, Serialize),
    serde(crate = "manta_util::serde", deny_unknown_fields)
)]
#[derive(Clone, Debug, Encode, Default, Eq, Hash, Decode, PartialEq, TypeInfo)]
pub struct DensePullResponse {
    /// Pull Continuation Flag
    ///
    /// The `should_continue` flag is set to `true` if the client should request more data from the
    /// ledger to finish the pull.
    pub should_continue: bool,

    /// Ledger Receiver Chunk
    // we decode the receivers/senders with our own way
    #[codec(skip)]
    pub receivers: alloc::string::String,

    /// Ledger Sender Chunk
    #[codec(skip)]
    pub senders: alloc::string::String,

    /// Total Number of Senders/Receivers in Ledger
    pub senders_receivers_total: [u8; 16],

    /// Next request checkpoint calculated from server.
    /// If should_continue = false, this data makes no sense.
    /// Else, the client can just use this one as next request cursor,
    /// It avoids complex computing on the client side,
    /// and the potential risk of inconsistent computing rules between the client and server
    #[codec(skip)]
    pub next_checkpoint: Option<Checkpoint>,
}

impl From<PullResponse> for DensePullResponse {
    #[inline]
    fn from(resp: PullResponse) -> DensePullResponse {
        Self {
            should_continue: resp.should_continue,
            receivers: base64::encode(resp.receivers.encode()),
            senders: base64::encode(resp.senders.encode()),
            senders_receivers_total: resp.senders_receivers_total,
            next_checkpoint: None,
        }
    }
}

/// Raw Checkpoint for Encoding and Decoding
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Decode, Encode)]
pub struct RawCheckpoint {
    /// Receiver Index
    pub receiver_index: [u64; MerkleTreeConfiguration::FOREST_WIDTH],

    /// Sender Index
    pub sender_index: u64,
}

impl RawCheckpoint {
    /// Builds a new [`RawCheckpoint`] from `receiver_index` and `sender_index`.
    #[inline]
    pub fn new(
        receiver_index: [u64; MerkleTreeConfiguration::FOREST_WIDTH],
        sender_index: u64,
    ) -> Self {
        Self {
            receiver_index,
            sender_index,
        }
    }
}

impl Default for RawCheckpoint {
    #[inline]
    fn default() -> Self {
        Self::new([0; MerkleTreeConfiguration::FOREST_WIDTH], 0)
    }
}

impl From<Checkpoint> for RawCheckpoint {
    #[inline]
    fn from(checkpoint: Checkpoint) -> Self {
        Self::new(
            (*checkpoint.receiver_index).map(|i| i as u64),
            checkpoint.sender_index as u64,
        )
    }
}

impl From<RawCheckpoint> for Checkpoint {
    #[inline]
    fn from(checkpoint: RawCheckpoint) -> Self {
        Self::new(
            checkpoint.receiver_index.map(|i| i as usize).into(),
            checkpoint.sender_index as usize,
        )
    }
}

/// Merkle Tree Parameters Decode Error Type
pub type MTParametersError = codec::DecodeError<
    <&'static [u8] as codec::Read>::Error,
    <config::UtxoAccumulatorModel as codec::Decode>::Error,
>;

/// Utxo Accumulator Item Hash Decode Error Type
pub type UtxoItemHashError = codec::DecodeError<
    <&'static [u8] as codec::Read>::Error,
    <config::utxo::UtxoAccumulatorItemHash as codec::Decode>::Error,
>;

/// Verification Context Decode Error Type
pub type VerifyingContextError = codec::DecodeError<
    <&'static [u8] as codec::Read>::Error,
    <config::VerifyingContext as codec::Decode>::Error,
>;

/// Wrap Type
#[derive(Clone, Copy)]
pub struct Wrap<T>(pub T);

impl<T> AsRef<T> for Wrap<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.0
    }
}

/// Wrap Pair Type
#[derive(Clone, Copy)]
pub struct WrapPair<L, R>(pub L, pub R);

impl<L, R> AsRef<R> for WrapPair<L, R> {
    #[inline]
    fn as_ref(&self) -> &R {
        &self.1
    }
}
