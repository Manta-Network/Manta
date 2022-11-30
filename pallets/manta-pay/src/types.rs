// Copyright 2020-2022 Manta Network.
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
use manta_crypto::merkle_tree;
use manta_pay::{
    config::{
        self,
        utxo::{self, MerkleTreeConfiguration},
    },
    crypto::poseidon::encryption::{self, BlockArray, CiphertextBlock},
    manta_crypto::{
        encryption::{hybrid, EmptyHeader},
        permutation::duplex,
        signature::schnorr,
    },
    manta_util::into_array_unchecked,
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

/// Decodes the `bytes` array of the given length `N` into the SCALE decodable type `T` returning a
/// blanket error if decoding fails.
#[inline]
pub(crate) fn decode<T, const N: usize>(bytes: [u8; N]) -> Result<T, Error>
where
    T: Decode,
{
    T::decode(&mut bytes.as_slice())
}

pub const FP_ENCODE: &str = "Fp encode should not failed.";
pub const FP_DECODE: &str = "Fp decode should not failed.";
pub const GROUP_ENCODE: &str = "Group encode should not failed.";
pub const GROUP_DECODE: &str = "Group decode should not failed.";
pub const PROOF_ENCODE: &str = "Proof encode should not failed.";
pub const PROOF_DECODE: &str = "Proof decode should not failed.";

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
pub fn fp_decode<T>(bytes: Vec<u8>) -> Result<Fp<T>, scale_codec::Error>
where
    T: manta_crypto::arkworks::ff::Field,
{
    Fp::try_from(bytes).map_err(|_e| scale_codec::Error::from(FP_DECODE))
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
pub fn group_decode<T>(bytes: Vec<u8>) -> Result<CryptoGroup<T>, scale_codec::Error>
where
    T: ProjectiveCurve,
{
    CryptoGroup::try_from(bytes).map_err(|_e| scale_codec::Error::from(GROUP_DECODE))
}

/// Proof encode to byte array
pub fn proof_encode<T>(group: CryptoProof<T>) -> Result<[u8; 128], scale_codec::Error>
where
    T: PairingEngine,
{
    use manta_util::codec::Encode;
    group
        .to_vec()
        .try_into()
        .map_err(|_e| scale_codec::Error::from(PROOF_ENCODE))
}

/// Proof decode from byte array
pub fn proof_decode<T>(bytes: Vec<u8>) -> Result<CryptoProof<T>, scale_codec::Error>
where
    T: PairingEngine,
{
    CryptoProof::try_from(bytes).map_err(|_e| scale_codec::Error::from(PROOF_DECODE))
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

///
pub type AssetId = [u8; 32];

///
pub type AssetValue = u128;

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
    pub value: AssetValue,
}

impl Asset {
    /// Builds a new [`Asset`] from `id` and `value`.
    #[inline]
    pub fn new(id: AssetId, value: AssetValue) -> Self {
        Self { id, value }
    }
}

impl From<config::Asset> for Asset {
    #[inline]
    fn from(asset: config::Asset) -> Self {
        Self {
            id: fp_encode(asset.id).expect(FP_ENCODE),
            value: asset.value,
        }
    }
}

impl TryFrom<Asset> for config::Asset {
    type Error = Error;

    #[inline]
    fn try_from(asset: Asset) -> Result<Self, Self::Error> {
        Ok(Self {
            id: fp_decode(asset.id.to_vec())?,
            value: asset.value,
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

impl From<utxo::OutgoingNote> for OutgoingNote {
    #[inline]
    fn from(note: utxo::OutgoingNote) -> Self {
        let encoded = note.ciphertext.ciphertext.encode();
        let mut encoded_ciphertext =
            [[0u8; OUTGOING_CIPHER_TEXT_COMPONENT_SIZE]; OUTGOING_CIPHER_TEXT_COMPONENTS_COUNT];
        for (outer_ind, array) in encoded_ciphertext.into_iter().enumerate() {
            for (inner_ind, _) in array.into_iter().enumerate() {
                encoded_ciphertext[outer_ind][inner_ind] = encoded[outer_ind * 32 + inner_ind];
            }
        }
        Self {
            ephemeral_public_key: group_encode(note.ciphertext.ephemeral_public_key)
                .expect(GROUP_ENCODE),
            ciphertext: encoded_ciphertext,
        }
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

impl From<config::SenderPost> for SenderPost {
    #[inline]
    fn from(post: config::SenderPost) -> Self {
        Self {
            utxo_accumulator_output: fp_encode(post.utxo_accumulator_output).expect(FP_ENCODE),
            nullifier_commitment: fp_encode(post.nullifier.nullifier.commitment).expect(FP_ENCODE),
            outgoing_note: From::from(post.nullifier.outgoing_note),
        }
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

impl From<utxo::IncomingNote> for IncomingNote {
    #[inline]
    fn from(note: utxo::IncomingNote) -> Self {
        Self {
            ephemeral_public_key: group_encode(note.ciphertext.ephemeral_public_key)
                .expect(GROUP_ENCODE),
            tag: fp_encode(note.ciphertext.ciphertext.tag.0).expect(FP_ENCODE),
            ciphertext: Array::from_iter(
                note.ciphertext.ciphertext.message[0]
                    .0
                    .iter()
                    .map(|fp| fp_encode(*fp).expect(FP_ENCODE)),
            )
            .into(),
        }
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

impl From<utxo::LightIncomingNote> for LightIncomingNote {
    #[inline]
    fn from(note: utxo::LightIncomingNote) -> Self {
        let encoded = note.ciphertext.ciphertext.encode();
        let mut encoded_arrays =
            [[0u8; INCOMING_CIPHER_TEXT_COMPONENT_SIZE]; INCOMING_CIPHER_TEXT_COMPONENTS_COUNT];
        for (outer_ind, array) in encoded_arrays.into_iter().enumerate() {
            for (inner_ind, _) in array.into_iter().enumerate() {
                encoded_arrays[outer_ind][inner_ind] =
                    encoded[outer_ind * INCOMING_CIPHER_TEXT_COMPONENT_SIZE + inner_ind];
            }
        }
        Self {
            ephemeral_public_key: group_encode(note.ciphertext.ephemeral_public_key)
                .expect(GROUP_ENCODE),
            ciphertext: encoded_arrays,
        }
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

impl From<utxo::FullIncomingNote> for FullIncomingNote {
    #[inline]
    fn from(note: utxo::FullIncomingNote) -> Self {
        Self {
            address_partition: note.address_partition,
            incoming_note: IncomingNote::from(note.incoming_note),
            light_incoming_note: LightIncomingNote::from(note.light_incoming_note),
        }
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
    pub fn from(utxo: utxo::Utxo) -> Utxo {
        Self {
            is_transparent: utxo.is_transparent,
            public_asset: utxo.public_asset.into(),
            commitment: fp_encode(utxo.commitment).expect(FP_ENCODE),
        }
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

impl From<config::ReceiverPost> for ReceiverPost {
    #[inline]
    fn from(post: config::ReceiverPost) -> Self {
        Self {
            utxo: Utxo::from(post.utxo),
            full_incoming_note: FullIncomingNote::from(post.note),
        }
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

impl From<utxo::AuthorizationSignature> for AuthorizationSignature {
    #[inline]
    fn from(signature: utxo::AuthorizationSignature) -> Self {
        Self {
            authorization_key: group_encode(signature.authorization_key).expect(GROUP_ENCODE),
            signature: (
                fp_encode(signature.signature.scalar).expect(FP_ENCODE),
                group_encode(signature.signature.nonce_point).expect(GROUP_ENCODE),
            ),
        }
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
    pub sources: Vec<AssetValue>,

    /// Sender Posts
    pub sender_posts: Vec<SenderPost>,

    /// Receiver Posts
    pub receiver_posts: Vec<ReceiverPost>,

    /// Sinks
    pub sinks: Vec<AssetValue>,

    /// Proof
    pub proof: Proof,
}

impl TransferPost {
    /// Constructs an [`Asset`] against the `asset_id` of `self` and `value`.
    #[inline]
    fn construct_asset(&self, value: &AssetValue) -> Option<Asset> {
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

impl From<config::TransferPost> for TransferPost {
    #[inline]
    fn from(post: config::TransferPost) -> Self {
        let authorization_signature = post.authorization_signature.map(Into::into);
        let asset_id = post.body.asset_id.map(|x| fp_encode(x).expect(FP_ENCODE));
        let sender_posts = post.body.sender_posts.into_iter().map(Into::into).collect();
        let receiver_posts = post
            .body
            .receiver_posts
            .into_iter()
            .map(Into::into)
            .collect();
        let proof = proof_encode(post.body.proof).expect(PROOF_ENCODE);
        Self {
            authorization_signature,
            asset_id,
            sources: post.body.sources,
            sender_posts,
            receiver_posts,
            sinks: post.body.sinks,
            proof,
        }
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
                sources: post.sources.into_iter().map(Into::into).collect(),
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
                sinks: post.sinks.into_iter().map(Into::into).collect(),
                proof,
            },
        })
    }
}

/// Merkle Tree Current Path
#[derive(Clone, Debug, Decode, Default, Encode, Eq, PartialEq, TypeInfo)]
pub struct CurrentPath {
    /// Sibling Digest
    pub sibling_digest: [u8; 32],

    /// Leaf Index
    pub leaf_index: u32,

    /// Inner Path
    pub inner_path: Vec<[u8; 32]>,
}

impl MaxEncodedLen for CurrentPath {
    #[inline]
    fn max_encoded_len() -> usize {
        0_usize
            .saturating_add(<[u8; 32]>::max_encoded_len())
            .saturating_add(u32::max_encoded_len())
            .saturating_add(
                // NOTE: We know that these paths don't exceed the path length.
                <[u8; 32]>::max_encoded_len().saturating_mul(
                    manta_crypto::merkle_tree::path_length::<MerkleTreeConfiguration, ()>(),
                ),
            )
    }
}

impl From<merkle_tree::CurrentPath<MerkleTreeConfiguration>> for CurrentPath {
    #[inline]
    fn from(path: merkle_tree::CurrentPath<MerkleTreeConfiguration>) -> Self {
        Self {
            sibling_digest: fp_encode(path.sibling_digest).expect(FP_ENCODE),
            leaf_index: path.inner_path.leaf_index.0 as u32,
            inner_path: path
                .inner_path
                .path
                .into_iter()
                .map(|x| fp_encode(x).expect(FP_ENCODE))
                .collect::<Vec<[u8; 32]>>(),
        }
    }
}

impl From<CurrentPath> for merkle_tree::CurrentPath<MerkleTreeConfiguration> {
    #[inline]
    fn from(path: CurrentPath) -> Self {
        Self::new(
            fp_decode(path.sibling_digest.to_vec()).expect(FP_DECODE),
            (path.leaf_index as usize).into(),
            path.inner_path
                .into_iter()
                .map(|x| fp_decode(x.to_vec()).expect(FP_DECODE))
                .collect(),
        )
    }
}

/// UTXO Merkle Tree Path
#[derive(Clone, Debug, Decode, Default, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct UtxoMerkleTreePath {
    /// Current Leaf Digest
    pub leaf_digest: Option<[u8; 32]>,

    /// Current Path
    pub current_path: CurrentPath,
}

/// Receiver Chunk Data Type
pub type ReceiverChunk = Vec<(Utxo, FullIncomingNote)>;

/// Sender Chunk Data Type
pub type SenderChunk = Vec<(NullifierCommitment, OutgoingNote)>;

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
    pub senders_receivers_total: u128,
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
