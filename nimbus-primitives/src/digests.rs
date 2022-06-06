// Copyright 2019-2021 PureStake Inc.
// This file is part of Nimbus.

// Nimbus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Nimbus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Nimbus.  If not, see <http://www.gnu.org/licenses/>.

//! A convenient interface over the digests used in nimbus.
//!
//! Currently Nimbus has two digests;
//! 1. A consensus digest that contains the block author identity
//!    This information is copied from the author inehrent.
//!    This may be replaced with a pre-runtime digest in the future.
//! 2. A seal digest that contains a signature over the rest of the
//!    block including the first digest.

use crate::{NimbusId, NimbusSignature, NIMBUS_ENGINE_ID};
use parity_scale_codec::Encode;
use sp_runtime::generic::DigestItem;

/// A digest item which is usable with aura consensus.
pub trait CompatibleDigestItem: Sized {
	/// Construct a pre-runtime digest from the given AuthorId
	fn nimbus_pre_digest(author: NimbusId) -> Self;

	/// If this item is a nimbus pre-runtime digest, return the author
	fn as_nimbus_pre_digest(&self) -> Option<NimbusId>;

	/// Construct a seal digest item from the given signature
	fn nimbus_seal(signature: NimbusSignature) -> Self;

	/// If this item is a nimbus seal, return the signature.
	fn as_nimbus_seal(&self) -> Option<NimbusSignature>;

	/// This will be deprecated in the future
	/// Construct a consensus digest from the given AuthorId
	fn nimbus_consensus_digest(author: NimbusId) -> Self;

	/// This will be deprecated in the future
	/// If this item is a nimbus consensus digest, return the author
	fn as_nimbus_consensus_digest(&self) -> Option<NimbusId>;
}

impl CompatibleDigestItem for DigestItem {
	fn nimbus_pre_digest(author: NimbusId) -> Self {
		DigestItem::PreRuntime(NIMBUS_ENGINE_ID, author.encode())
	}

	fn as_nimbus_pre_digest(&self) -> Option<NimbusId> {
		self.pre_runtime_try_to(&NIMBUS_ENGINE_ID)
	}

	fn nimbus_seal(signature: NimbusSignature) -> Self {
		DigestItem::Seal(NIMBUS_ENGINE_ID, signature.encode())
	}

	fn as_nimbus_seal(&self) -> Option<NimbusSignature> {
		self.seal_try_to(&NIMBUS_ENGINE_ID)
	}

	// Remove this once deprecated
	fn nimbus_consensus_digest(author: NimbusId) -> Self {
		DigestItem::Consensus(NIMBUS_ENGINE_ID, author.encode())
	}

	// Remove this once deprecated. I don't think it is used anyway.
	// Notice that it calls the pre_runtime helper function.
	fn as_nimbus_consensus_digest(&self) -> Option<NimbusId> {
		self.pre_runtime_try_to(&NIMBUS_ENGINE_ID)
	}
}
