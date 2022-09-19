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
    sp_api_hidden_includes_construct_runtime::hidden_include::traits::{Currency, OriginTrait},
    Balance, Get, Vec, Weight,
};
use core::marker::PhantomData;
use frame_support::traits::OnRuntimeUpgrade;
#[cfg(feature = "try-runtime")]
use frame_support::traits::OnRuntimeUpgradeHelpersExt;
use sp_runtime::traits::UniqueSaturatedInto;

/// Migration to move old invulnerables to the staking set on upgrade
pub struct InitializeStakingPallet<T>(PhantomData<T>);
const INITIAL_MAX_ACTIVE_COLLATORS: u32 = 63;
impl<T> OnRuntimeUpgrade for InitializeStakingPallet<T>
where
    T: frame_system::Config
        + pallet_parachain_staking::Config
        + pallet_session::Config
        + manta_collator_selection::Config,
    <<T as frame_system::Config>::Origin as OriginTrait>::AccountId:
        From<<T as frame_system::Config>::AccountId>,
    pallet_parachain_staking::BalanceOf<T>: Into<Balance> + From<Balance>,
    <<T as manta_collator_selection::Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance: From<
        <<T as pallet_parachain_staking::Config>::Currency as Currency<
            <T as frame_system::Config>::AccountId,
        >>::Balance,
    >,
{
    fn on_runtime_upgrade() -> Weight
    where
        <<T as frame_system::Config>::Origin as OriginTrait>::AccountId:
            From<<T as frame_system::Config>::AccountId>,
        pallet_parachain_staking::BalanceOf<T>: Into<Balance> + From<Balance>,
    {
        log::info!(
            target: "InitializeStakingPallet",
            "Migrating invulnerables from manta_collator_selection to
            pallet_parachain_staking"
        );

        // 1. Find current invulnerables
        let invulnerables = manta_collator_selection::Pallet::<T>::invulnerables();

        // 2. Clear the invulnerables list
        let _ = manta_collator_selection::Pallet::<T>::set_invulnerables(
            <T as frame_system::Config>::Origin::root(),
            Vec::new(),
        );

        // 3. Register invulnerables to whitelist
        // 3.1 Ensure DesiredCandidates can take the additional invulnerables
        let n_of_candidates = manta_collator_selection::Pallet::<T>::candidates().len() as u32;
        let new_n_of_candidates = n_of_candidates + invulnerables.len() as u32;
        let desired_candidates = manta_collator_selection::Pallet::<T>::desired_candidates();
        if new_n_of_candidates > desired_candidates {
            let _ = manta_collator_selection::Pallet::<T>::set_desired_candidates(
                <T as frame_system::Config>::Origin::root(),
                new_n_of_candidates,
            );
        }
        // 3.2 Ensure the candidacy bond for collator_selection is actually 400k
        // NOTE: This is needed to migrate already deployed testnets like Baikal
        let _ = manta_collator_selection::Pallet::<T>::set_candidacy_bond(
            <T as frame_system::Config>::Origin::root(),
            T::MinWhitelistCandidateStk::get().unique_saturated_into(),
        );

        // 3.3 onboard with manta_collator_selection::registerCandidate
        for invuln in invulnerables.clone() {
            log::info!(
                "Migrating account {:?} with initial free_balance {:?}",
                invuln.clone(),
                <T as pallet_parachain_staking::Config>::Currency::free_balance(&invuln)
            );
            let _ = manta_collator_selection::Pallet::<T>::register_candidate(
                <T as frame_system::Config>::Origin::root(),
                invuln.clone(),
            );
            log::info!(
                "Migrating account {:?} with free_balance after collator_selection::candidates {:?}",
                invuln.clone(),
                <T as pallet_parachain_staking::Config>::Currency::free_balance(&invuln)
            );
        }

        // 4. Initialize parachain staking pallet to genesis-equivalent state
        //    and onboard invulnerables to staking with 400k lock

        // +1 because migrations execute before this is updated in the on_initialize hook
        let current_block = frame_system::Pallet::<T>::block_number() + 1u32.into();
        pallet_parachain_staking::Pallet::<T>::initialize_pallet(
            current_block,
            invulnerables,
            crate::currency::inflation_config::<T>(),
        )
        .unwrap_or_else(|err| {
            log::error!(
                "pallet_parachain_staking initialization failed with {:?}. Chain is likely effed",
                err
            );
        });

        // Setting total_selected will take effect at the beginning of the next round, so for the first 6 hours
        // our invulnerables will be the only collators
        let _ = pallet_parachain_staking::Pallet::<T>::set_total_selected(
            <T as frame_system::Config>::Origin::root(),
            INITIAL_MAX_ACTIVE_COLLATORS,
        );

        T::BlockWeights::get().max_block // simply use the whole block
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        // Before beginning the migration invulnerables must have 400k KMA in free balance
        let invulnerables = manta_collator_selection::Pallet::<T>::invulnerables();
        for invulnerable in invulnerables.clone() {
            assert!(
                <T as pallet_parachain_staking::Config>::Currency::free_balance(&invulnerable)
                    >= T::MinWhitelistCandidateStk::get()
            );
        }

        // Also ensure there's enough space in collator_selection's Candidates to add all invulnerables
        assert!(
            <T as manta_collator_selection::Config>::MaxCandidates::get()
                >= (manta_collator_selection::Pallet::<T>::candidates().len() + invulnerables.len())
                    as u32
        );

        Self::set_temp_storage(invulnerables, "invulnerables");
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        // Invulnerables were migrated correctly
        let invulnerables: Vec<T::AccountId> =
            Self::get_temp_storage("invulnerables").expect("must exist");
        for invuln in invulnerables {
            assert!(
                !manta_collator_selection::Pallet::<T>::candidates()
                    .iter()
                    .any(|x| x.who == invuln),
                "invulnerables must no longer be candidates with collator_selection",
            );
            let bond = pallet_parachain_staking::Bond {
                owner: invuln.clone(),
                amount: T::MinWhitelistCandidateStk::get(),
            };
            assert!(
                pallet_parachain_staking::Pallet::<T>::candidate_pool().contains(&bond),
                "invulnerables must now be candidates",
            );
            assert!(
                pallet_parachain_staking::Pallet::<T>::selected_candidates().contains(&invuln),
                "invulnerables must be active collators for the first round",
            );
        }
        // Other pallet parameters are set
        // Inflation is set to 3%
        use sp_arithmetic::{PerThing, Perbill};
        let infla = pallet_parachain_staking::Range {
            min: Perbill::from_rational_with_rounding(5u32, 200u32, sp_arithmetic::Rounding::Down)
                .expect("constant denom is not 0. qed"), // = 2.5%
            ideal: Perbill::from_percent(3),
            max: Perbill::from_percent(3),
        };
        assert_eq!(
            pallet_parachain_staking::Pallet::<T>::inflation_config().annual,
            infla
        );

        // TotalSelected is 63
        assert_eq!(
            pallet_parachain_staking::Pallet::<T>::total_selected(),
            INITIAL_MAX_ACTIVE_COLLATORS
        );

        // Round is 1
        let current_block = frame_system::Pallet::<T>::block_number() + 1u32.into();
        let round_info = pallet_parachain_staking::Pallet::<T>::round();
        assert_eq!(round_info.current, 1u32);
        assert_eq!(round_info.first, current_block);

        Ok(())
    }
}
