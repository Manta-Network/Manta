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

//! Pallet that allows block authors to include their identity in a block via an inherent.
//! Currently the author does not _prove_ their identity, just states it. So it should not be used,
//! for things like equivocation slashing that require authenticated authorship information.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::FindAuthor;
use nimbus_primitives::{
	AccountLookup, CanAuthor, EventHandler, NimbusId, SlotBeacon, INHERENT_IDENTIFIER,
	NIMBUS_ENGINE_ID,
};
use parity_scale_codec::{Decode, Encode};
use sp_inherents::{InherentIdentifier, IsFatalError};
use sp_runtime::{ConsensusEngineId, RuntimeString};

mod exec;
pub use exec::BlockExecutor;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// The Author Inherent pallet. The core of the nimbus consensus framework's runtime presence.
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// A type to convert between AuthorId and AccountId. This is useful when you want to associate
		/// Block authoring behavior with an AccoutId for rewards or slashing. If you do not need to
		/// hold an AccountID responsible for authoring use `()` which acts as an identity mapping.
		type AccountLookup: AccountLookup<Self::AccountId>;

		/// Other pallets that want to be informed about block authorship
		type EventHandler: EventHandler<Self::AccountId>;

		/// The final word on whether the reported author can author at this height.
		/// This will be used when executing the inherent. This check is often stricter than the
		/// Preliminary check, because it can use more data.
		/// If the pallet that implements this trait depends on an inherent, that inherent **must**
		/// be included before this one.
		type CanAuthor: CanAuthor<Self::AccountId>;

		/// Some way of determining the current slot for purposes of verifying the author's eligibility
		type SlotBeacon: SlotBeacon;
	}

	impl<T> sp_runtime::BoundToRuntimeAppPublic for Pallet<T> {
		type Public = NimbusId;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Author already set in block.
		AuthorAlreadySet,
		/// No AccountId was found to be associated with this author
		NoAccountId,
		/// The author in the inherent is not an eligible author.
		CannotBeAuthor,
	}

	/// Author of current block.
	#[pallet::storage]
	pub type Author<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_: T::BlockNumber) -> Weight {
			// Start by clearing out the previous block's author
			<Author<T>>::kill();

			// Now extract the author from the digest
			let digest = <frame_system::Pallet<T>>::digest();

			let pre_runtime_digests = digest.logs.iter().filter_map(|d| d.as_pre_runtime());
			Self::find_author(pre_runtime_digests).map(|author_account| {
				// Store the author so we can confirm eligibility after the inherents have executed
				<Author<T>>::put(&author_account);

				//TODO, should we reuse the same trait that Pallet Authorship uses?
				// Notify any other pallets that are listening (eg rewards) about the author
				T::EventHandler::note_author(author_account);
			});

			T::DbWeight::get().write * 2
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// This inherent is a workaround to run code after the "real" inherents have executed,
		/// but before transactions are executed.
		// This should go into on_post_inherents when it is ready https://github.com/paritytech/substrate/pull/10128
		// TODO better weight. For now we just set a somewhat conservative fudge factor
		#[pallet::weight((10 * T::DbWeight::get().write, DispatchClass::Mandatory))]
		pub fn kick_off_authorship_validation(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_none(origin)?;

			let author = <Author<T>>::get()
				.expect("Block invalid, no authorship information supplied in preruntime digest.");

			assert!(
				T::CanAuthor::can_author(&author, &T::SlotBeacon::slot()),
				"Block invalid, supplied author is not eligible."
			);

			Ok(Pays::No.into())
		}
	}

	#[pallet::inherent]
	impl<T: Config> ProvideInherent for Pallet<T> {
		type Call = Call<T>;
		type Error = InherentError;
		const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

		fn is_inherent_required(_: &InherentData) -> Result<Option<Self::Error>, Self::Error> {
			// Return Ok(Some(_)) unconditionally because this inherent is required in every block
			// If it is not found, throw an AuthorInherentRequired error.
			Ok(Some(InherentError::Other(
				sp_runtime::RuntimeString::Borrowed(
					"Inherent required to manually initiate author validation",
				),
			)))
		}

		// Regardless of whether the client is still supplying the author id,
		// we will create the new empty-payload inherent extrinsic.
		fn create_inherent(_data: &InherentData) -> Option<Self::Call> {
			Some(Call::kick_off_authorship_validation {})
		}

		fn is_inherent(call: &Self::Call) -> bool {
			matches!(call, Call::kick_off_authorship_validation { .. })
		}
	}

	impl<T: Config> FindAuthor<T::AccountId> for Pallet<T> {
		fn find_author<'a, I>(digests: I) -> Option<T::AccountId>
		where
			I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
		{
			for (id, mut data) in digests.into_iter() {
				if id == NIMBUS_ENGINE_ID {
					let author_id = NimbusId::decode(&mut data)
						.expect("NimbusId encoded in preruntime digest must be valid");

					let author_account = T::AccountLookup::lookup_account(&author_id)
						.expect("No Account Mapped to this NimbusId");

					return Some(author_account);
				}
			}

			None
		}
	}

	/// To learn whether a given NimbusId can author, as opposed to an account id, you
	/// can ask this pallet directly. It will do the mapping for you.
	impl<T: Config> CanAuthor<NimbusId> for Pallet<T> {
		fn can_author(author: &NimbusId, slot: &u32) -> bool {
			let account = match T::AccountLookup::lookup_account(&author) {
				Some(account) => account,
				// Authors whose account lookups fail will not be eligible
				None => {
					return false;
				}
			};

			T::CanAuthor::can_author(&account, slot)
		}
	}
}

#[derive(Encode)]
#[cfg_attr(feature = "std", derive(Debug, Decode))]
pub enum InherentError {
	Other(RuntimeString),
}

impl IsFatalError for InherentError {
	fn is_fatal_error(&self) -> bool {
		match *self {
			InherentError::Other(_) => true,
		}
	}
}

impl InherentError {
	/// Try to create an instance ouf of the given identifier and data.
	#[cfg(feature = "std")]
	pub fn try_from(id: &InherentIdentifier, data: &[u8]) -> Option<Self> {
		if id == &INHERENT_IDENTIFIER {
			<InherentError as parity_scale_codec::Decode>::decode(&mut &data[..]).ok()
		} else {
			None
		}
	}
}
