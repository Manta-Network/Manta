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

use crate::{
    sp_api_hidden_includes_construct_runtime::hidden_include::traits::OriginTrait,
    Weight,
};
use core::marker::PhantomData;
use frame_support::traits::OnRuntimeUpgrade;
use crate::Balance;

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

        // +1 because migrations execute before this is updated in the on_initialize hook
        let current_block = frame_system::Pallet::<T>::block_number() + 1u32.into();
        let _ = pallet_parachain_staking::Pallet::<T>::initialize_pallet(
            current_block,
            invulnerables,
            crate::currency::inflation_config().unique_saturated_into(),
        );
        // Setting total_selected will take place at the beginning of the next round, so for the first 6 hours
        // our invulnerables will be the only collators
        let _ = pallet_parachain_staking::Pallet::<T>::set_total_selected(
            <T as frame_system::Config>::Origin::root(),
            63u32,
        );

        // 4. Onboard to staking with 400k lock -> join candidates
        // NOTE: We treat our nodes the same as the whitelisted community collators
        // let minstake = T::MinWhitelistCandidateStk::get();
        // for invuln in invulnerables.clone() {
        //     let _ = pallet_parachain_staking::Pallet::<T>::join_candidates(
        //         <T as frame_system::Config>::Origin::signed(invuln.into()),
        //         minstake,
        //         10u32
        //     );
        // }

        // 5. Add them to the storage variable containing the currently eligible block producers

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

        Self::set_temp_storage(invulnerables, "invulnerables");

        // let mut expected_delegator_state_entries = 0u64;
        // let mut expected_requests = 0u64;
        // for (_key, state) in migration::storage_iter::<OldDelegator<T::AccountId, BalanceOf<T>>>(
        //     Self::PALLET_PREFIX,
        //     Self::DELEGATOR_STATE_PREFIX,
        // ) {
        //     Self::set_temp_storage(
        //         state.requests.less_total,
        //         &*format!("expected_delegator-{:?}_decrease_amount", state.id),
        //     );

        //     for (collator, request) in state.requests.requests.iter() {
        //         Self::set_temp_storage(
        //             Self::old_request_to_string(&state.id, &request),
        //             &*format!(
        //                 "expected_collator-{:?}_delegator-{:?}_request",
        //                 collator, state.id,
        //             ),
        //         );
        //     }
        //     expected_delegator_state_entries = expected_delegator_state_entries.saturating_add(1);
        //     expected_requests =
        //         expected_requests.saturating_add(state.requests.requests.len() as u64);
        // }

        // Self::set_temp_storage(
        //     expected_delegator_state_entries,
        //     "expected_delegator_state_entries",
        // );
        // Self::set_temp_storage(expected_requests, "expected_requests");

        // use frame_support::migration;

        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        // Check invulnerables were migrated correctly
        let invulnerables: Vec<T::AccountId> =
            Self::get_temp_storage("invulnerables").expect("must exist");
        assert_eq!(
            invulnerables,
            <T as Config>::CandidatePool::get(),
            "invulnerables must now be candidates",
        );
        assert_eq!(
            invulnerables,
            <T as Config>::SelectedCandidates::get(),
            "invulnerables must be active collators for the first round",
        );
        // // Scheduled decrease amount (bond_less) is correctly migrated
        // let mut actual_delegator_state_entries = 0;
        // for (delegator, state) in <DelegatorState<T>>::iter() {
        //     let expected_delegator_decrease_amount: BalanceOf<T> = Self::get_temp_storage(
        //         &*format!("expected_delegator-{:?}_decrease_amount", delegator),
        //     )
        //     .expect("must exist");
        //     assert_eq!(
        //         expected_delegator_decrease_amount, state.less_total,
        //         "decrease amount did not match for delegator {:?}",
        //         delegator,
        //     );
        //     actual_delegator_state_entries = actual_delegator_state_entries.saturating_add(1);
        // }

        // // Existing delegator state entries are not removed
        // let expected_delegator_state_entries: u64 =
        //     Self::get_temp_storage("expected_delegator_state_entries").expect("must exist");
        // assert_eq!(
        //     expected_delegator_state_entries, actual_delegator_state_entries,
        //     "unexpected change in the number of DelegatorState entries"
        // );

        // // Scheduled requests are correctly migrated
        // let mut actual_requests = 0u64;
        // for (collator, scheduled_requests) in <DelegationScheduledRequests<T>>::iter() {
        //     for request in scheduled_requests {
        //         let expected_delegator_request: String = Self::get_temp_storage(&*format!(
        //             "expected_collator-{:?}_delegator-{:?}_request",
        //             collator, request.delegator,
        //         ))
        //         .expect("must exist");
        //         let actual_delegator_request = Self::new_request_to_string(&request);
        //         assert_eq!(
        //             expected_delegator_request, actual_delegator_request,
        //             "scheduled request did not match for collator {:?}, delegator {:?}",
        //             collator, request.delegator,
        //         );

        //         actual_requests = actual_requests.saturating_add(1);
        //     }
        // }

        // let expected_requests: u64 =
        //     Self::get_temp_storage("expected_requests").expect("must exist");
        // assert_eq!(
        //     expected_requests, actual_requests,
        //     "number of scheduled request entries did not match",
        // );

        Ok(())
    }
}

// impl<T: frame_system::Config> MigrateInvulnerables<T> {
//     const PALLET_PREFIX: &'static [u8] = b"ParachainStaking";
//     const DELEGATOR_STATE_PREFIX: &'static [u8] = b"DelegatorState";

//     #[allow(deprecated)]
//     #[cfg(feature = "try-runtime")]
//     fn old_request_to_string(
//         delegator: &T::AccountId,
//         request: &crate::deprecated::DelegationRequest<T::AccountId, BalanceOf<T>>,
//     ) -> String {
//         match request.action {
//             DelegationChange::Revoke => {
//                 format!(
//                     "delegator({:?})_when({})_Revoke({:?})",
//                     delegator, request.when_executable, request.amount
//                 )
//             }
//             DelegationChange::Decrease => {
//                 format!(
//                     "delegator({:?})_when({})_Decrease({:?})",
//                     delegator, request.when_executable, request.amount
//                 )
//             }
//         }
//     }

//     #[cfg(feature = "try-runtime")]
//     fn new_request_to_string(request: &ScheduledRequest<T::AccountId, BalanceOf<T>>) -> String {
//         match request.action {
//             DelegationAction::Revoke(v) => {
//                 format!(
//                     "delegator({:?})_when({})_Revoke({:?})",
//                     request.delegator, request.when_executable, v
//                 )
//             }
//             DelegationAction::Decrease(v) => {
//                 format!(
//                     "delegator({:?})_when({})_Decrease({:?})",
//                     request.delegator, request.when_executable, v
//                 )
//             }
//         }
//     }
// }

// #![allow(clippy::unnecessary_cast)]

// use core::marker::PhantomData;
// #[allow(deprecated)]
// use frame_support::migration::remove_storage_prefix;
// use frame_support::{
//     migration::have_storage_value,
//     pallet_prelude::Weight,
//     traits::{Get, OnRuntimeUpgrade},
// };
// pub struct MigrateInvulnerablesToStaking<T>(PhantomData<T>);
// impl<T: frame_system::Config> OnRuntimeUpgrade for MigrateInvulnerablesToStaking<T> {
//     fn on_runtime_upgrade() -> Weight {
//         // add invulns to candidate set
//         let invulns = pallet_session::invulnerables();
//         for invuln in invulns
//         {
//             let result = pallet_parachain_staking::join_candidates(invuln);
//         }

//         // if have_storage_value(b"Sudo", b"Key", b"") {
//         //     #[allow(deprecated)]
//         //     remove_storage_prefix(b"Sudo", b"Key", b"");
//         //     #[allow(deprecated)]
//         //     remove_storage_prefix(b"Sudo", b":__STORAGE_VERSION__:", b"");
//         //     log::info!(target: "OnRuntimeUpgrade", "✅ Sudo key has been removed.");
//         //     log::info!(target: "OnRuntimeUpgrade", "✅ The pallet version has been removed.");
//         //     T::DbWeight::get()
//         //         .reads(1 as Weight)
//         //         .saturating_add(T::DbWeight::get().writes(1 as Weight))
//         // } else {
//         //     T::DbWeight::get().reads(1 as Weight)
//         // }
//     }

//     #[cfg(feature = "try-runtime")]
//     fn pre_upgrade() -> Result<(), &'static str> {
//         if have_storage_value(b"Sudo", b"Key", b"") {
//             log::info!(target: "OnRuntimeUpgrade", "✅ Sudo key will be removed soon.");
//             log::info!(target: "OnRuntimeUpgrade", "✅ The pallet version will be removed soon.");
//             Ok(())
//         } else {
//             Err("Sudo doesn't exist.")
//         }
//     }

//     #[cfg(feature = "try-runtime")]
//     fn post_upgrade() -> Result<(), &'static str> {
//         if have_storage_value(b"Sudo", b"Key", b"") {
//             Err("Failed to remove sudo module.")
//         } else {
//             Ok(())
//         }
//     }
// }
