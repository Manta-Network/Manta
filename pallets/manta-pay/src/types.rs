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

use super::*;
use manta_pay::{
    config::utxo::v1::{self, MerkleTreeConfiguration},
    manta_crypto::{
        encryption::{hybrid, EmptyHeader},
        permutation::duplex,
        signature::schnorr,
    },
    manta_util::into_array_unchecked,
};
use manta_util::Array;
use scale_codec::Error;

#[cfg(feature = "rpc")]
use manta_pay::manta_util::serde::{Deserialize, Serialize};

pub use manta_pay::config::utxo::v1::{Checkpoint, RawCheckpoint};

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
pub const UTXO_LENGTH: usize = 32;

/// UTXO Type
pub type Utxo = [u8; UTXO_LENGTH];

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

impl From<v1::OutgoingNote> for OutgoingNote {
    #[inline]
    fn from(note: v1::OutgoingNote) -> Self {
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

impl TryFrom<OutgoingNote> for v1::OutgoingNote {
    type Error = Error;

    #[inline]
    fn try_from(note: OutgoingNote) -> Result<Self, Self::Error> {
        todo!()
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
pub type IncomingCiphertext = [[u8; 32]; 4];

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

impl From<v1::IncomingNote> for IncomingNote {
    #[inline]
    fn from(note: v1::IncomingNote) -> Self {
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

impl TryFrom<IncomingNote> for v1::IncomingNote {
    type Error = Error;

    #[inline]
    fn try_from(note: IncomingNote) -> Result<Self, Self::Error> {
        /*
        Ok(Self {
            header: EmptyHeader::default(),
            ciphertext: hybrid::Ciphertext {
                ephemeral_public_key: decode(note.ephemeral_public_key)?,
                ciphertext: duplex::Ciphertext {
                    tag: decode(note.tag)?.into(),
                    message: decode(note.message)?,
                },
            },
        })
        */
        todo!()
    }
}

/// Receiver Post
#[derive(Clone, Debug, Decode, Encode, Eq, Hash, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct ReceiverPost {
    /// Unspent Transaction Output
    pub utxo: Utxo,

    /// Incoming Note
    pub incoming_note: IncomingNote,
}

impl From<config::ReceiverPost> for ReceiverPost {
    #[inline]
    fn from(post: config::ReceiverPost) -> Self {
        /*
        Self {
            utxo: encode(post.utxo),
            note: post.note.into(),
        }
        */
        todo!()
    }
}

impl TryFrom<ReceiverPost> for config::ReceiverPost {
    type Error = Error;

    #[inline]
    fn try_from(post: ReceiverPost) -> Result<Self, Self::Error> {
        /* TODO:
        Ok(Self {
            utxo: decode(post.utxo)?,
            note: post.note.try_into()?,
        })
        */
        todo!()
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

impl From<v1::AuthorizationSignature> for AuthorizationSignature {
    #[inline]
    fn from(signature: v1::AuthorizationSignature) -> Self {
        Self {
            authorization_key: encode(signature.authorization_key),
            signature: (
                encode(signature.signature.scalar),
                encode(signature.signature.nonce_point),
            ),
        }
    }
}

impl TryFrom<AuthorizationSignature> for v1::AuthorizationSignature {
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
pub type ReceiverChunk = Vec<(Utxo, IncomingNote)>;

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
}
