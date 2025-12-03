// Copyright 2020-2024 Manta Network.
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

//! Benchmarking setup for pallet-chameleon-bridge

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as ChameleonBridge;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_core::{H160, H256};

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn add_validator() {
        let validator: T::AccountId = account("validator", 0, 0);

        #[extrinsic_call]
        _(RawOrigin::Root, validator.clone());

        assert!(BridgeValidators::<T>::get(&validator));
    }

    #[benchmark]
    fn remove_validator() {
        let validator: T::AccountId = account("validator", 0, 0);
        BridgeValidators::<T>::insert(&validator, true);

        #[extrinsic_call]
        _(RawOrigin::Root, validator.clone());

        assert!(!BridgeValidators::<T>::get(&validator));
    }

    #[benchmark]
    fn mint_wrapped_asset() {
        let validator: T::AccountId = account("validator", 0, 0);
        let recipient: T::AccountId = account("recipient", 1, 0);
        BridgeValidators::<T>::insert(&validator, true);
        
        let eth_tx_hash = H256::from([1u8; 32]);
        let asset = BridgeableAsset::ETH;
        let amount = 1_000_000_000_000_000_000u128; // 1 ETH

        #[extrinsic_call]
        _(RawOrigin::Signed(validator), eth_tx_hash, asset, amount, recipient.clone());

        assert!(PendingDeposits::<T>::contains_key(eth_tx_hash));
    }

    #[benchmark]
    fn initiate_withdrawal() {
        let user: T::AccountId = account("user", 0, 0);
        let asset = BridgeableAsset::ETH;
        let amount = 1_000_000_000_000_000_000u128; // 1 ETH
        let eth_destination = H160::from([1u8; 20]);

        #[extrinsic_call]
        _(RawOrigin::Signed(user), asset, amount, eth_destination);

        assert!(PendingWithdrawals::<T>::contains_key(0));
    }

    #[benchmark]
    fn sign_withdrawal() {
        let validator: T::AccountId = account("validator", 0, 0);
        let user: T::AccountId = account("user", 1, 0);
        BridgeValidators::<T>::insert(&validator, true);
        
        // Setup withdrawal
        let asset = BridgeableAsset::ETH;
        let amount = 1_000_000_000_000_000_000u128; // 1 ETH
        let eth_destination = H160::from([1u8; 20]);
        
        let withdrawal = BridgeWithdrawal {
            withdrawal_id: 0,
            from: user,
            eth_destination,
            asset,
            amount,
            status: WithdrawalStatus::Pending,
            signature_count: 0,
        };
        PendingWithdrawals::<T>::insert(0, withdrawal);
        
        let signature = frame_support::BoundedVec::try_from(vec![1, 2, 3, 4]).unwrap();

        #[extrinsic_call]
        _(RawOrigin::Signed(validator.clone()), 0, signature.clone());

        assert!(WithdrawalSignatures::<T>::contains_key(0, &validator));
    }

    #[benchmark]
    fn report_lock_event() {
        let validator: T::AccountId = account("validator", 0, 0);
        let recipient: T::AccountId = account("recipient", 1, 0);
        BridgeValidators::<T>::insert(&validator, true);
        
        let eth_tx_hash = H256::from([2u8; 32]);
        let asset = BridgeableAsset::USDC;
        let amount = 1_000_000u128; // 1 USDC

        #[extrinsic_call]
        _(RawOrigin::Signed(validator), eth_tx_hash, recipient, asset, amount);

        assert!(PendingDeposits::<T>::contains_key(eth_tx_hash));
    }

    #[benchmark]
    fn pause_bridge() {
        #[extrinsic_call]
        _(RawOrigin::Root);

        assert!(IsPaused::<T>::get());
    }

    #[benchmark]
    fn resume_bridge() {
        IsPaused::<T>::put(true);

        #[extrinsic_call]
        _(RawOrigin::Root);

        assert!(!IsPaused::<T>::get());
    }

    impl_benchmark_test_suite!(ChameleonBridge, crate::mock::new_test_ext(), crate::mock::Test);
}