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
        utxo::v2::{self, MerkleTreeConfiguration},
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

pub use manta_pay::config::utxo::v2::{Checkpoint, RawCheckpoint};

/// Encodes the SCALE encodable `value` into a byte array with the given length `N`.
#[inline]
pub(crate) fn encode<T, const N: usize>(value: T) -> [u8; N]
where
    T: Encode,
{
    into_array_unchecked(value.encode())
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

///
pub const PROOF_LENGTH: usize = 192;

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
            id: encode(asset.id),
            value: asset.value,
        }
    }
}

impl TryFrom<Asset> for config::Asset {
    type Error = Error;

    #[inline]
    fn try_from(asset: Asset) -> Result<Self, Self::Error> {
        Ok(Self {
            id: decode(asset.id)?,
            value: asset.value,
        })
    }
}

/// Outgoing Ciphertext
pub type OutgoingCiphertext = [[u8; 32]; 2];

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

    /// Tag
    pub tag: Tag,

    /// Ciphertext
    pub ciphertext: OutgoingCiphertext,
}

impl From<v2::OutgoingNote> for OutgoingNote {
    #[inline]
    fn from(note: v2::OutgoingNote) -> Self {
        Self {
            ephemeral_public_key: encode(note.ciphertext.ephemeral_public_key),
            tag: encode(note.ciphertext.ciphertext.tag.0),
            ciphertext: Array::from_iter(
                note.ciphertext.ciphertext.message[0].0.iter().map(encode),
            )
            .into(),
        }
    }
}

impl TryFrom<OutgoingNote> for v2::OutgoingNote {
    type Error = Error;

    #[inline]
    fn try_from(note: OutgoingNote) -> Result<Self, Self::Error> {
        Ok(Self {
            header: EmptyHeader::default(),
            ciphertext: hybrid::Ciphertext {
                ephemeral_public_key: decode(note.ephemeral_public_key)?,
                ciphertext: duplex::Ciphertext {
                    tag: encryption::Tag(decode(note.tag)?),
                    message: BlockArray(BoxArray(Box::new([CiphertextBlock(
                        note.ciphertext
                            .into_iter()
                            .map(decode)
                            .collect::<Result<Vec<_>, _>>()?
                            .into(),
                    )]))),
                },
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
            utxo_accumulator_output: encode(post.utxo_accumulator_output),
            nullifier_commitment: encode(post.nullifier.commitment),
            outgoing_note: From::from(post.nullifier.outgoing_note),
        }
    }
}

impl TryFrom<SenderPost> for config::SenderPost {
    type Error = Error;

    #[inline]
    fn try_from(post: SenderPost) -> Result<Self, Self::Error> {
        Ok(Self {
            utxo_accumulator_output: decode(post.utxo_accumulator_output)?,
            nullifier: config::Nullifier {
                commitment: decode(post.nullifier_commitment)?,
                outgoing_note: TryFrom::try_from(post.outgoing_note)?,
            },
        })
    }
}

/// Incoming Ciphertext Type
pub type IncomingCiphertext = [[u8; 32]; 3];

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

impl From<v2::IncomingNote> for IncomingNote {
    #[inline]
    fn from(note: v2::IncomingNote) -> Self {
        Self {
            ephemeral_public_key: encode(note.ciphertext.ephemeral_public_key),
            tag: encode(note.ciphertext.ciphertext.tag.0),
            ciphertext: Array::from_iter(
                note.ciphertext.ciphertext.message[0].0.iter().map(encode),
            )
            .into(),
        }
    }
}

impl TryFrom<IncomingNote> for v2::IncomingNote {
    type Error = Error;

    #[inline]
    fn try_from(note: IncomingNote) -> Result<Self, Self::Error> {
        Ok(Self {
            header: EmptyHeader::default(),
            ciphertext: hybrid::Ciphertext {
                ephemeral_public_key: decode(note.ephemeral_public_key)?,
                ciphertext: duplex::Ciphertext {
                    tag: encryption::Tag(decode(note.tag)?),
                    message: BlockArray(BoxArray(Box::new([CiphertextBlock(
                        note.ciphertext
                            .into_iter()
                            .map(decode)
                            .collect::<Result<Vec<_>, _>>()?
                            .into(),
                    )]))),
                },
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
}

impl From<v2::FullIncomingNote> for FullIncomingNote {
    #[inline]
    fn from(note: v2::FullIncomingNote) -> Self {
        Self {
            address_partition: note.address_partition,
            incoming_note: IncomingNote::from(note.incoming_note),
        }
    }
}

impl TryFrom<FullIncomingNote> for v2::FullIncomingNote {
    type Error = Error;

    #[inline]
    fn try_from(note: FullIncomingNote) -> Result<Self, Self::Error> {
        Ok(Self {
            address_partition: note.address_partition,
            incoming_note: note.incoming_note.try_into()?,
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
    pub fn from(utxo: v2::Utxo) -> Utxo {
        Self {
            is_transparent: utxo.is_transparent,
            public_asset: utxo.public_asset.into(),
            commitment: encode(utxo.commitment),
        }
    }

    ///
    #[inline]
    pub fn try_into(self) -> Result<v2::Utxo, Error> {
        Ok(v2::Utxo {
            is_transparent: self.is_transparent,
            public_asset: self.public_asset.try_into()?,
            commitment: decode(self.commitment)?,
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

impl From<v2::AuthorizationSignature> for AuthorizationSignature {
    #[inline]
    fn from(signature: v2::AuthorizationSignature) -> Self {
        Self {
            authorization_key: encode(signature.authorization_key),
            signature: (
                encode(signature.signature.scalar),
                encode(signature.signature.nonce_point),
            ),
        }
    }
}

impl TryFrom<AuthorizationSignature> for v2::AuthorizationSignature {
    type Error = Error;

    #[inline]
    fn try_from(signature: AuthorizationSignature) -> Result<Self, Self::Error> {
        Ok(Self {
            authorization_key: decode(signature.authorization_key)?,
            signature: schnorr::Signature {
                scalar: decode(signature.signature.0)?,
                nonce_point: decode(signature.signature.1)?,
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
        Self {
            authorization_signature: post.authorization_signature.map(Into::into),
            asset_id: post.body.asset_id.map(encode),
            sources: post.body.sources,
            sender_posts: post.body.sender_posts.into_iter().map(Into::into).collect(),
            receiver_posts: post
                .body
                .receiver_posts
                .into_iter()
                .map(Into::into)
                .collect(),
            sinks: post.body.sinks,
            proof: encode(post.body.proof),
        }
    }
}

impl TryFrom<TransferPost> for config::TransferPost {
    type Error = Error;

    #[inline]
    fn try_from(post: TransferPost) -> Result<Self, Self::Error> {
        Ok(Self {
            authorization_signature: post
                .authorization_signature
                .map(TryInto::try_into)
                .transpose()?,
            body: config::TransferPostBody {
                asset_id: post.asset_id.map(decode).transpose()?,
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
                proof: decode(post.proof)?,
            },
        })
    }
}

/// Leaf Digest Type
pub type LeafDigest = merkle_tree::LeafDigest<MerkleTreeConfiguration>;

/// Inner Digest Type
pub type InnerDigest = merkle_tree::InnerDigest<MerkleTreeConfiguration>;

/// Merkle Tree Current Path
#[derive(Clone, Debug, Decode, Default, Encode, Eq, PartialEq, TypeInfo)]
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
            .saturating_add(LeafDigest::max_encoded_len())
            .saturating_add(u32::max_encoded_len())
            .saturating_add(
                // NOTE: We know that these paths don't exceed the path length.
                InnerDigest::max_encoded_len().saturating_mul(
                    manta_crypto::merkle_tree::path_length::<MerkleTreeConfiguration, ()>(),
                ),
            )
    }
}

impl From<merkle_tree::CurrentPath<MerkleTreeConfiguration>> for CurrentPath {
    #[inline]
    fn from(path: merkle_tree::CurrentPath<MerkleTreeConfiguration>) -> Self {
        Self {
            sibling_digest: path.sibling_digest,
            leaf_index: path.inner_path.leaf_index.0 as u32,
            inner_path: path.inner_path.path,
        }
    }
}

impl From<CurrentPath> for merkle_tree::CurrentPath<MerkleTreeConfiguration> {
    #[inline]
    fn from(path: CurrentPath) -> Self {
        Self::new(
            path.sibling_digest,
            (path.leaf_index as usize).into(),
            path.inner_path,
        )
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
