// Copyright 2019-2022 Manta Network.
// This file is part of pallet-manta-pay.
//
// pallet-manta-pay is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pallet-manta-pay is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pallet-manta-pay.  If not, see <http://www.gnu.org/licenses/>.

//! Type Definitions for Manta Pay

use super::*;
use manta_util::into_array_unchecked;

/// Encodes the SCALE encodable `value` into a byte array with the given length `N`.
#[inline]
fn encode<T, const N: usize>(value: T) -> [u8; N]
where
	T: Encode,
{
	into_array_unchecked(value.encode())
}

/// Decodes the `bytes` array of the given length `N` into the SCALE decodable type `T` returning a
/// blanket error if decoding fails.
#[inline]
fn decode<T, const N: usize>(bytes: [u8; N]) -> Result<T, ()>
where
	T: Decode,
{
	T::decode(&mut bytes.as_slice()).map_err(|_| ())
}

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
	pub value: Balance,
}

impl Asset {
	/// Builds a new [`Asset`] from `id` and `value`.
	#[inline]
	pub fn new(id: AssetId, value: Balance) -> Self {
		Self { id, value }
	}
}

/// Encrypted Note
#[derive(Clone, Debug, Decode, Encode, Eq, Hash, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct EncryptedNote {
	/// Ciphertext
	pub ciphertext: [u8; 36],

	/// Ephemeral Public Key
	pub ephemeral_public_key: [u8; 32],
}

impl Default for EncryptedNote {
	#[inline]
	fn default() -> Self {
		Self {
			ciphertext: [0; 36],
			ephemeral_public_key: [0; 32],
		}
	}
}

impl From<config::EncryptedNote> for EncryptedNote {
	#[inline]
	fn from(note: config::EncryptedNote) -> Self {
		Self {
			ciphertext: note.ciphertext.into(),
			ephemeral_public_key: encode(note.ephemeral_public_key),
		}
	}
}

impl TryFrom<EncryptedNote> for config::EncryptedNote {
	type Error = ();

	#[inline]
	fn try_from(note: EncryptedNote) -> Result<Self, Self::Error> {
		Ok(Self {
			ciphertext: note.ciphertext.into(),
			ephemeral_public_key: decode(note.ephemeral_public_key)?,
		})
	}
}

/// Sender Post
#[derive(Clone, Debug, Decode, Encode, Eq, Hash, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct SenderPost {
	/// UTXO Accumulator Output
	pub utxo_accumulator_output: [u8; 32],

	/// Void Number
	pub void_number: [u8; 32],
}

impl From<config::SenderPost> for SenderPost {
	#[inline]
	fn from(post: config::SenderPost) -> Self {
		Self {
			utxo_accumulator_output: encode(post.utxo_accumulator_output),
			void_number: encode(post.void_number),
		}
	}
}

impl TryFrom<SenderPost> for config::SenderPost {
	type Error = ();

	#[inline]
	fn try_from(post: SenderPost) -> Result<Self, Self::Error> {
		Ok(Self {
			utxo_accumulator_output: decode(post.utxo_accumulator_output)?,
			void_number: decode(post.void_number)?,
		})
	}
}

/// Receiver Post
#[derive(Clone, Debug, Decode, Encode, Eq, Hash, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct ReceiverPost {
	/// Unspent Transaction Output
	pub utxo: [u8; 32],

	/// Encrypted Note
	pub note: EncryptedNote,
}

impl From<config::ReceiverPost> for ReceiverPost {
	#[inline]
	fn from(post: config::ReceiverPost) -> Self {
		Self {
			utxo: encode(post.utxo),
			note: post.note.into(),
		}
	}
}

impl TryFrom<ReceiverPost> for config::ReceiverPost {
	type Error = ();

	#[inline]
	fn try_from(post: ReceiverPost) -> Result<Self, Self::Error> {
		Ok(Self {
			utxo: decode(post.utxo)?,
			note: post.note.try_into()?,
		})
	}
}

/// Transfer Post
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq, TypeInfo)]
pub struct TransferPost {
	/// Asset Id
	pub asset_id: Option<AssetId>,

	/// Sources
	pub sources: Vec<Balance>,

	/// Sender Posts
	pub sender_posts: Vec<SenderPost>,

	/// Receiver Posts
	pub receiver_posts: Vec<ReceiverPost>,

	/// Sinks
	pub sinks: Vec<Balance>,

	/// Validity Proof
	pub validity_proof: [u8; 192],
}

impl From<config::TransferPost> for TransferPost {
	#[inline]
	fn from(post: config::TransferPost) -> Self {
		Self {
			asset_id: post.asset_id.map(|id| id.0),
			sources: post.sources.into_iter().map(|s| s.0).collect(),
			sender_posts: post.sender_posts.into_iter().map(Into::into).collect(),
			receiver_posts: post.receiver_posts.into_iter().map(Into::into).collect(),
			sinks: post.sinks.into_iter().map(|s| s.0).collect(),
			validity_proof: encode(post.validity_proof),
		}
	}
}

impl TryFrom<TransferPost> for config::TransferPost {
	type Error = ();

	#[inline]
	fn try_from(post: TransferPost) -> Result<Self, Self::Error> {
		Ok(Self {
			asset_id: post.asset_id.map(asset::AssetId),
			sources: post.sources.into_iter().map(asset::AssetValue).collect(),
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
			sinks: post.sinks.into_iter().map(asset::AssetValue).collect(),
			validity_proof: decode(post.validity_proof)?,
		})
	}
}

/// Leaf Digest Type
pub type LeafDigest = merkle_tree::LeafDigest<config::MerkleTreeConfiguration>;

/// Inner Digest Type
pub type InnerDigest = merkle_tree::InnerDigest<config::MerkleTreeConfiguration>;

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
					manta_crypto::merkle_tree::path_length::<config::MerkleTreeConfiguration>(),
				),
			)
	}
}

impl From<merkle_tree::CurrentPath<config::MerkleTreeConfiguration>> for CurrentPath {
	#[inline]
	fn from(path: merkle_tree::CurrentPath<config::MerkleTreeConfiguration>) -> Self {
		Self {
			sibling_digest: path.sibling_digest,
			leaf_index: path.inner_path.leaf_index.0 as u32,
			inner_path: path.inner_path.path,
		}
	}
}

impl From<CurrentPath> for merkle_tree::CurrentPath<config::MerkleTreeConfiguration> {
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
