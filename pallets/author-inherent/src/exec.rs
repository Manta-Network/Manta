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

//! Block executive to be used by relay chain validators when validating parachain blocks built
//! with the nimubs consensus family.

use frame_support::traits::ExecuteBlock;
use sp_api::{BlockT, HeaderT};
// For some reason I can't get these logs to actually print
use log::debug;
use nimbus_primitives::{digests::CompatibleDigestItem, NimbusId, NIMBUS_ENGINE_ID};
use sp_application_crypto::ByteArray;
use sp_runtime::{generic::DigestItem, RuntimeAppPublic};

/// Block executive to be used by relay chain validators when validating parachain blocks built
/// with the nimubs consensus family.
///
/// This will strip the seal digest, and confirm that it contains a valid signature
/// By the block author reported in the author inherent.
///
/// Essentially this contains the logic of the verifier plus the inner executive.
/// TODO Degisn improvement:
/// Can we share code with the verifier?
/// Can this struct take a verifier as an associated type?
/// Or maybe this will just get simpler in general when https://github.com/paritytech/polkadot/issues/2888 lands
pub struct BlockExecutor<T, I>(sp_std::marker::PhantomData<(T, I)>);

impl<Block, T, I> ExecuteBlock<Block> for BlockExecutor<T, I>
where
	Block: BlockT,
	I: ExecuteBlock<Block>,
{
	fn execute_block(block: Block) {
		let (mut header, extrinsics) = block.deconstruct();

		debug!(target: "executive", "In hacked Executive. Initial digests are {:?}", header.digest());

		// Set the seal aside for checking.
		let seal = header
			.digest_mut()
			.pop()
			.expect("Seal digest is present and is last item");

		debug!(target: "executive", "In hacked Executive. digests after stripping {:?}", header.digest());
		debug!(target: "executive", "The seal we got {:?}", seal);

		let signature = seal
			.as_nimbus_seal()
			.unwrap_or_else(|| panic!("HeaderUnsealed"));

		debug!(target: "executive", "🪲 Header hash after popping digest {:?}", header.hash());

		debug!(target: "executive", "🪲 Signature according to executive is {:?}", signature);

		// Grab the author information from the preruntime digest
		//TODO use the trait
		let claimed_author = header
			.digest()
			.logs
			.iter()
			.find_map(|digest| match *digest {
				DigestItem::PreRuntime(id, ref author_id) if id == NIMBUS_ENGINE_ID => {
					Some(author_id.clone())
				}
				_ => None,
			})
			.expect("Expected pre-runtime digest that contains author id bytes");

		debug!(target: "executive", "🪲 Claimed Author according to executive is {:?}", claimed_author);

		// Verify the signature
		let valid_signature = NimbusId::from_slice(&claimed_author)
			.expect("Expected claimed author to be a valid NimbusId.")
			.verify(&header.hash(), &signature);

		debug!(target: "executive", "🪲 Valid signature? {:?}", valid_signature);

		if !valid_signature {
			panic!("Block signature invalid");
		}

		// Now that we've verified the signature, hand execution off to the inner executor
		// which is probably the normal frame executive.
		I::execute_block(Block::new(header, extrinsics));
	}
}
