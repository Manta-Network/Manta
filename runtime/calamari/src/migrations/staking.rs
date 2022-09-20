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
    Balance, Vec, Weight,
};
use core::{marker::PhantomData, str::FromStr};
use frame_support::traits::OnRuntimeUpgrade;
use sp_runtime::{traits::UniqueSaturatedInto, AccountId32};
use sp_std::vec;
/// Migration to move old invulnerables to the staking set on upgrade
pub struct UnInitializeStakingPallet<T>(PhantomData<T>);
const INITIAL_MAX_ACTIVE_COLLATORS: u32 = 63;
impl<T> OnRuntimeUpgrade for UnInitializeStakingPallet<T>
where
    T: frame_system::Config + pallet_session::Config + manta_collator_selection::Config,
    <<T as frame_system::Config>::Origin as OriginTrait>::AccountId:
        From<<T as frame_system::Config>::AccountId>,
    <T as frame_system::Config>::AccountId: From<sp_runtime::AccountId32>,
    <T as pallet_session::Config>::ValidatorId: From<sp_runtime::AccountId32>,
{
    fn on_runtime_upgrade() -> Weight
    where
        <<T as frame_system::Config>::Origin as OriginTrait>::AccountId:
            From<<T as frame_system::Config>::AccountId>,
        <T as frame_system::Config>::AccountId: From<sp_runtime::AccountId32>,
        <T as pallet_session::Config>::ValidatorId: From<sp_runtime::AccountId32>,
    {
        log::info!(
            target: "UnInitializeStakingPallet",
            "Migrating invulnerables",
        );
        use sp_std::borrow::ToOwned;
        // 0. get invulnerable keys from parachain staking
        // ```
        // ❯ subalfred key dmtwRyEeNyRW3KApnTxjHahWCjN5b9gDjdvxpizHt6E9zYkXj
        // public-key 0x0027131c176c0d19a2a5cc475ecc657f936085912b846839319249e700f37e79
        // Substrate 5C4uTugykUmqGtA7bYnZjqncdbJLFnyXPUBjM9vW3xfJTupZ
        // ❯ subalfred convert hex2bytes 0x0027131c176c0d19a2a5cc475ecc657f936085912b846839319249e700f37e79
        // [0, 39, 19, 28, 23, 108, 13, 25, 162, 165, 204, 71, 94, 204, 101, 127, 147, 96, 133, 145, 43, 132, 104, 57, 49, 146, 73, 231, 0, 243, 126, 121]
        // ```
        // WARNING: These keys are specific to the **Calamari Parachain Staging** Network
        let inv: Vec<T::AccountId> = vec![
            // AccountId32::from(*("dmtwRyEeNyRW3KApnTxjHahWCjN5b9gDjdvxpizHt6E9zYkXj"()),
            AccountId32::from([
                0, 39, 19, 28, 23, 108, 13, 25, 162, 165, 204, 71, 94, 204, 101, 127, 147, 96, 133,
                145, 43, 132, 104, 57, 49, 146, 73, 231, 0, 243, 126, 121,
            ])
            .into(),
            // AccountId32::from(*("dmud2BmjLyMtbAX2FaVTUtvmutoCKvR3GbARLc4crzGvVMCwu".as_bytes())),
            AccountId32::from([
                30, 88, 211, 195, 144, 12, 124, 230, 198, 216, 33, 82, 190, 203, 69, 191, 123, 211,
                69, 63, 178, 210, 103, 229, 247, 44, 165, 18, 133, 188, 161, 115,
            ])
            .into(),
            // AccountId32::from(*("dmvSXhJWeJEKTZT8CCUieJDaNjNFC4ZFqfUm4Lx1z7J7oFzBf".as_bytes())),
            AccountId32::from([
                66, 148, 178, 167, 22, 206, 169, 29, 208, 8, 214, 148, 210, 100, 254, 234, 249,
                240, 186, 249, 192, 184, 203, 227, 225, 7, 81, 89, 71, 237, 68, 13,
            ])
            .into(),
            // AccountId32::from(*("dmx4vuA3PnQmraqJqeJaKRydUjP1AW4wMVTPLQWgZSpDyQUrp".as_bytes())),
            AccountId32::from([
                138, 147, 224, 247, 86, 68, 128, 48, 220, 179, 1, 141, 37, 215, 92, 123, 249, 122,
                46, 47, 241, 93, 2, 253, 31, 85, 191, 63, 33, 4, 251, 91,
            ])
            .into(),
            // AccountId32::from(*("dmxvZaMQir24EPxvFiCzkhDZaiScPB7ZWpHXUv5x8uct2A3du".as_bytes())),
            AccountId32::from([
                176, 110, 93, 133, 32, 120, 246, 74, 183, 74, 249, 179, 26, 221, 16, 227, 109, 4,
                56, 184, 71, 188, 146, 95, 186, 203, 241, 225, 73, 99, 227, 121,
            ])
            .into(),
        ];
        let inv2: Vec<<T as pallet_session::Config>::ValidatorId> = vec![
            // AccountId32::from(*("dmtwRyEeNyRW3KApnTxjHahWCjN5b9gDjdvxpizHt6E9zYkXj"()),
            AccountId32::from([
                0, 39, 19, 28, 23, 108, 13, 25, 162, 165, 204, 71, 94, 204, 101, 127, 147, 96, 133,
                145, 43, 132, 104, 57, 49, 146, 73, 231, 0, 243, 126, 121,
            ])
            .into(),
            // AccountId32::from(*("dmud2BmjLyMtbAX2FaVTUtvmutoCKvR3GbARLc4crzGvVMCwu".as_bytes())),
            AccountId32::from([
                30, 88, 211, 195, 144, 12, 124, 230, 198, 216, 33, 82, 190, 203, 69, 191, 123, 211,
                69, 63, 178, 210, 103, 229, 247, 44, 165, 18, 133, 188, 161, 115,
            ])
            .into(),
            // AccountId32::from(*("dmvSXhJWeJEKTZT8CCUieJDaNjNFC4ZFqfUm4Lx1z7J7oFzBf".as_bytes())),
            AccountId32::from([
                66, 148, 178, 167, 22, 206, 169, 29, 208, 8, 214, 148, 210, 100, 254, 234, 249,
                240, 186, 249, 192, 184, 203, 227, 225, 7, 81, 89, 71, 237, 68, 13,
            ])
            .into(),
            // AccountId32::from(*("dmx4vuA3PnQmraqJqeJaKRydUjP1AW4wMVTPLQWgZSpDyQUrp".as_bytes())),
            AccountId32::from([
                138, 147, 224, 247, 86, 68, 128, 48, 220, 179, 1, 141, 37, 215, 92, 123, 249, 122,
                46, 47, 241, 93, 2, 253, 31, 85, 191, 63, 33, 4, 251, 91,
            ])
            .into(),
            // AccountId32::from(*("dmxvZaMQir24EPxvFiCzkhDZaiScPB7ZWpHXUv5x8uct2A3du".as_bytes())),
            AccountId32::from([
                176, 110, 93, 133, 32, 120, 246, 74, 183, 74, 249, 179, 26, 221, 16, 227, 109, 4,
                56, 184, 71, 188, 146, 95, 186, 203, 241, 225, 73, 99, 227, 121,
            ])
            .into(),
        ];

        // 1. set invulnerables
        let _ = manta_collator_selection::Pallet::<T>::set_invulnerables(
            <T as frame_system::Config>::Origin::root(),
            inv,
        );

        // 2. and validators
        pallet_session::Validators::<T>::put(&inv2);

        // remove parachain staking storage
        let prefix = b"ParachainStaking";
        frame_support::storage::unhashed::kill_prefix(prefix, None);

        // trigger on_genesis_session probably

        0
    }
}
