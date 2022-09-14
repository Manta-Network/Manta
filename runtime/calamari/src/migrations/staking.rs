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

use crate::Balance;
use crate::{
    sp_api_hidden_includes_construct_runtime::hidden_include::traits::OriginTrait, Weight,
};
use core::marker::PhantomData;
use frame_support::traits::OnRuntimeUpgrade;

/// Migration to move old invulnerables to the staking set on upgrade
/// [DelegationScheduledRequests] storage item.
/// Additionally [DelegatorState] is migrated from [OldDelegator] to [Delegator].
pub struct MigrateInvulnerables<T>(PhantomData<T>);

#[allow(deprecated)]
impl<T> OnRuntimeUpgrade for MigrateInvulnerables<T>
where
    T: frame_system::Config
        + pallet_parachain_staking::Config
        + pallet_session::Config
        + manta_collator_selection::Config,
    <<T as frame_system::Config>::Origin as OriginTrait>::AccountId:
        From<<T as frame_system::Config>::AccountId>,
    pallet_parachain_staking::BalanceOf<T>: Into<Balance> + From<Balance>,
{
    fn on_runtime_upgrade() -> Weight
    where
        <<T as frame_system::Config>::Origin as OriginTrait>::AccountId:
            From<<T as frame_system::Config>::AccountId>,
        pallet_parachain_staking::BalanceOf<T>: Into<Balance> + From<Balance>,
    {
        use crate::*;

        log::info!(
            target: "SplitDelegatorStateIntoDelegationScheduledRequests",
            "running migration for DelegatorState to new version and DelegationScheduledRequests \
            storage item"
        );

        let mut reads: Weight = 0;
        let mut writes: Weight = 0;

        // 1. Find current invulnerables
        let invulnerables = manta_collator_selection::Pallet::<T>::invulnerables();

        // 2. Remove from invulnerables list
        let _ = manta_collator_selection::Pallet::<T>::set_invulnerables(
            <T as frame_system::Config>::Origin::root(),
            // <T as frame_system::Config>::Origin::signed(invunlerable),
            vec![],
        );

        // 3. Register to whitelist (onboard with manta_collator_selection::registerCandidate)
        for invuln in invulnerables.clone() {
            let _ = manta_collator_selection::Pallet::<T>::register_candidate(
                <T as frame_system::Config>::Origin::root(),
                // <T as frame_system::Config>::Origin::signed(invunlerable),
                invuln,
            );
        }

        // 4. Initialize parachain staking pallet to genesis-equivalent state
        //    and onboard invulnerables to staking with 400k lock

        // +1 because migrations execute before this is updated in the on_initialize hook
        let current_block = frame_system::Pallet::<T>::block_number() + 1u32.into();
        let _ = pallet_parachain_staking::Pallet::<T>::initialize_pallet(
            current_block,
            invulnerables,
            crate::currency::inflation_config::<T>(),
        );
        // Setting total_selected will take effect at the beginning of the next round, so for the first 6 hours
        // our invulnerables will be the only collators
        let _ = pallet_parachain_staking::Pallet::<T>::set_total_selected(
            <T as frame_system::Config>::Origin::root(),
            63u32,
        );

        // TODO: Get correct weight from the extrinsics
        T::DbWeight::get().reads_writes(reads, writes)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        // 2. Ensure they each have 400k KMA
        let invulnerables = manta_collator_selection::Pallet::<T>::invulnerables();
        for invulnerable in invulnerables {
            assert!(
                <T as Config>::Currency::free_balance(invulnerable)
                    > T::MinWhitelistCandidateStk::get()
            );
        }

        // use frame_support::migration;
        Self::set_temp_storage(invulnerables, "invulnerables");

        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        // Invulnerables were migrated correctly
        let invulnerables: Vec<T::AccountId> =
            Self::get_temp_storage("invulnerables").expect("must exist");
        assert_eq!(
            invulnerables,
            <T as Config>::CandidatePool::get(),
            "invulnerables must now be candidates",
        );
        assert_eq!(invulnerables, <T as Config>::CandidatePool::get());
        assert_eq!(
            invulnerables,
            <T as Config>::SelectedCandidates::get(),
            "invulnerables must be active collators for the first round",
        );

        // Round 1 has begun
        let current_block = frame_system::Pallet::<T>::block_number() + 1u32.into();
        let coll_count : u32 = invulnerables.len() as u32;
        assert_event_emitted!(Event::NewRound {
                starting_block: current_block,
                round: 1u32,
                selected_collators_number: coll_count,
                total_balance: 400_000 * coll_count * KMA,
            });

        // Other pallet parameters are set
        // Inflation is set to 3%
        assert_eq!(
            <T as Config>::inflation_config().annual.ideal,
            Perbill::from_percent(3)
        );

        // TotalSelected is 63
        assert_eq!(<T as Config>::total_selected(), 63u32);

        // Round is 1
        assert_eq!(<T as Config>::round(), 1u32);

        Ok(())
    }
}
