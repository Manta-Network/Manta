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

//! Type definitions for pallet-manta-pay

use super::*;

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
	pub ephemeral_public_key: config::PublicKey,
}

impl Default for EncryptedNote {
	#[inline]
	fn default() -> Self {
		Self {
			ciphertext: [0; 36],
			ephemeral_public_key: Default::default(),
		}
	}
}

impl From<config::EncryptedNote> for EncryptedNote {
	#[inline]
	fn from(note: config::EncryptedNote) -> Self {
		Self {
			ciphertext: note.ciphertext.into(),
			ephemeral_public_key: note.ephemeral_public_key,
		}
	}
}

impl From<EncryptedNote> for config::EncryptedNote {
	#[inline]
	fn from(note: EncryptedNote) -> Self {
		Self {
			ciphertext: note.ciphertext.into(),
			ephemeral_public_key: note.ephemeral_public_key,
		}
	}
}

/// Sender Post
#[derive(Clone, Debug, Decode, Encode, Eq, Hash, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct SenderPost {
	/// UTXO Accumulator Output
	pub utxo_accumulator_output: config::UtxoAccumulatorOutput,

	/// Void Number
	pub void_number: config::VoidNumber,
}

impl From<config::SenderPost> for SenderPost {
	#[inline]
	fn from(post: config::SenderPost) -> Self {
		Self {
			utxo_accumulator_output: post.utxo_accumulator_output,
			void_number: post.void_number,
		}
	}
}

impl From<SenderPost> for config::SenderPost {
	#[inline]
	fn from(post: SenderPost) -> Self {
		Self {
			utxo_accumulator_output: post.utxo_accumulator_output,
			void_number: post.void_number,
		}
	}
}

/// Receiver Post
#[derive(Clone, Debug, Decode, Encode, Eq, Hash, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct ReceiverPost {
	/// Unspent Transaction Output
	pub utxo: config::Utxo,

	/// Encrypted Note
	pub note: EncryptedNote,
}

impl From<config::ReceiverPost> for ReceiverPost {
	#[inline]
	fn from(post: config::ReceiverPost) -> Self {
		Self {
			utxo: post.utxo,
			note: post.note.into(),
		}
	}
}

impl From<ReceiverPost> for config::ReceiverPost {
	#[inline]
	fn from(post: ReceiverPost) -> Self {
		Self {
			utxo: post.utxo,
			note: post.note.into(),
		}
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
	pub validity_proof: config::Proof,
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
			validity_proof: post.validity_proof,
		}
	}
}

impl From<TransferPost> for config::TransferPost {
	#[inline]
	fn from(post: TransferPost) -> Self {
		Self {
			asset_id: post.asset_id.map(asset::AssetId),
			sources: post.sources.into_iter().map(asset::AssetValue).collect(),
			sender_posts: post.sender_posts.into_iter().map(Into::into).collect(),
			receiver_posts: post.receiver_posts.into_iter().map(Into::into).collect(),
			sinks: post.sinks.into_iter().map(asset::AssetValue).collect(),
			validity_proof: post.validity_proof,
		}
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
