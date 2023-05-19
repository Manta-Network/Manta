// Copyright 2020-2023 Manta Network.
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

#![cfg(feature = "calamari")]

use super::{mock::ExtBuilder, *};
use frame_support::{
    assert_ok,
    dispatch::Dispatchable,
    traits::{PalletInfo, StorageInfo, StorageInfoTrait},
    StorageHasher, Twox128,
};
use pallet_transaction_payment::ChargeTransactionPayment;
use sp_runtime::{
    traits::{Header as HeaderT, SignedExtension},
    Percent,
};

#[test]
fn calamari_vesting_works() {
    ExtBuilder::default().build().execute_with(|| {
        let unvested = 100 * KMA;
        assert_ok!(CalamariVesting::vested_transfer(
            RuntimeOrigin::signed(ALICE.clone()),
            sp_runtime::MultiAddress::Id(BOB.clone()),
            unvested
        ));

        assert_eq!(Balances::free_balance(BOB.clone()), 100 * KMA);
        assert_eq!(Balances::usable_balance(BOB.clone()), 0);

        let schedule = calamari_vesting::Pallet::<Runtime>::vesting_schedule();
        let mut vested = 0;

        for period in 0..schedule.len() {
            // Timestamp expects milliseconds, so multiply by 1_000 to convert from seconds.
            let now = schedule[period].1 * 1_000 + 1;
            Timestamp::set_timestamp(now);
            assert_ok!(CalamariVesting::vest(RuntimeOrigin::signed(BOB.clone())));
            vested += schedule[period].0 * unvested;
            assert_eq!(Balances::usable_balance(BOB.clone()), vested);
        }
    });
}

#[test]
fn verify_pallet_prefixes() {
    fn is_pallet_prefix<P: 'static>(name: &str) {
        // Compares the unhashed pallet prefix in the `StorageInstance` implementation by every
        // storage item in the pallet P. This pallet prefix is used in conjunction with the
        // item name to get the unique storage key: hash(PalletPrefix) + hash(StorageName)
        // https://github.com/paritytech/substrate/blob/master/frame/support/procedural/src/pallet/
        // expand/storage.rs#L389-L401
        assert_eq!(
            <Runtime as frame_system::Config>::PalletInfo::name::<P>(),
            Some(name)
        );
    }

    is_pallet_prefix::<calamari_runtime::System>("System");
    is_pallet_prefix::<calamari_runtime::ParachainSystem>("ParachainSystem");
    is_pallet_prefix::<calamari_runtime::Timestamp>("Timestamp");
    is_pallet_prefix::<calamari_runtime::ParachainInfo>("ParachainInfo");
    is_pallet_prefix::<calamari_runtime::TransactionPause>("TransactionPause");

    is_pallet_prefix::<calamari_runtime::Balances>("Balances");
    is_pallet_prefix::<calamari_runtime::TransactionPayment>("TransactionPayment");

    is_pallet_prefix::<calamari_runtime::Democracy>("Democracy");
    is_pallet_prefix::<calamari_runtime::Council>("Council");
    is_pallet_prefix::<calamari_runtime::CouncilMembership>("CouncilMembership");
    is_pallet_prefix::<calamari_runtime::TechnicalCommittee>("TechnicalCommittee");
    is_pallet_prefix::<calamari_runtime::TechnicalMembership>("TechnicalMembership");

    is_pallet_prefix::<calamari_runtime::ParachainStaking>("ParachainStaking");

    is_pallet_prefix::<calamari_runtime::AuthorInherent>("AuthorInherent");
    is_pallet_prefix::<calamari_runtime::AuraAuthorFilter>("AuraAuthorFilter");

    is_pallet_prefix::<calamari_runtime::Authorship>("Authorship");
    is_pallet_prefix::<calamari_runtime::CollatorSelection>("CollatorSelection");
    is_pallet_prefix::<calamari_runtime::Session>("Session");
    is_pallet_prefix::<calamari_runtime::Aura>("Aura");

    is_pallet_prefix::<calamari_runtime::Treasury>("Treasury");
    is_pallet_prefix::<calamari_runtime::Preimage>("Preimage");

    is_pallet_prefix::<calamari_runtime::Scheduler>("Scheduler");

    is_pallet_prefix::<calamari_runtime::XcmpQueue>("XcmpQueue");
    is_pallet_prefix::<calamari_runtime::PolkadotXcm>("PolkadotXcm");
    is_pallet_prefix::<calamari_runtime::CumulusXcm>("CumulusXcm");
    is_pallet_prefix::<calamari_runtime::DmpQueue>("DmpQueue");
    is_pallet_prefix::<calamari_runtime::XTokens>("XTokens");

    is_pallet_prefix::<calamari_runtime::Utility>("Utility");
    is_pallet_prefix::<calamari_runtime::Multisig>("Multisig");

    is_pallet_prefix::<calamari_runtime::Assets>("Assets");
    is_pallet_prefix::<calamari_runtime::AssetManager>("AssetManager");
    is_pallet_prefix::<calamari_runtime::MantaSbt>("MantaSbt");
    is_pallet_prefix::<calamari_runtime::MantaPay>("MantaPay");

    is_pallet_prefix::<calamari_runtime::CalamariVesting>("CalamariVesting");

    let prefix = |pallet_name, storage_name| {
        let mut res = [0u8; 32];
        res[0..16].copy_from_slice(&Twox128::hash(pallet_name));
        res[16..32].copy_from_slice(&Twox128::hash(storage_name));
        res.to_vec()
    };
    assert_eq!(
        <Timestamp as StorageInfoTrait>::storage_info(),
        vec![
            StorageInfo {
                pallet_name: b"Timestamp".to_vec(),
                storage_name: b"Now".to_vec(),
                prefix: prefix(b"Timestamp", b"Now"),
                max_values: Some(1),
                max_size: Some(8),
            },
            StorageInfo {
                pallet_name: b"Timestamp".to_vec(),
                storage_name: b"DidUpdate".to_vec(),
                prefix: prefix(b"Timestamp", b"DidUpdate"),
                max_values: Some(1),
                max_size: Some(1),
            }
        ]
    );
    assert_eq!(
        <Balances as StorageInfoTrait>::storage_info(),
        vec![
            StorageInfo {
                pallet_name: b"Balances".to_vec(),
                storage_name: b"TotalIssuance".to_vec(),
                prefix: prefix(b"Balances", b"TotalIssuance"),
                max_values: Some(1),
                max_size: Some(16),
            },
            StorageInfo {
                pallet_name: b"Balances".to_vec(),
                storage_name: b"InactiveIssuance".to_vec(),
                prefix: prefix(b"Balances", b"InactiveIssuance"),
                max_values: Some(1),
                max_size: Some(16),
            },
            StorageInfo {
                pallet_name: b"Balances".to_vec(),
                storage_name: b"Account".to_vec(),
                prefix: prefix(b"Balances", b"Account"),
                max_values: None,
                max_size: Some(112),
            },
            StorageInfo {
                pallet_name: b"Balances".to_vec(),
                storage_name: b"Locks".to_vec(),
                prefix: prefix(b"Balances", b"Locks"),
                max_values: None,
                max_size: Some(1299),
            },
            StorageInfo {
                pallet_name: b"Balances".to_vec(),
                storage_name: b"Reserves".to_vec(),
                prefix: prefix(b"Balances", b"Reserves"),
                max_values: None,
                max_size: Some(1249),
            },
        ]
    );
}

#[test]
fn test_collectives_storage_item_prefixes() {
    for StorageInfo { pallet_name, .. } in
        <calamari_runtime::CouncilMembership as StorageInfoTrait>::storage_info()
    {
        assert_eq!(pallet_name, b"CouncilMembership".to_vec());
    }

    for StorageInfo { pallet_name, .. } in
        <calamari_runtime::TechnicalMembership as StorageInfoTrait>::storage_info()
    {
        assert_eq!(pallet_name, b"TechnicalMembership".to_vec());
    }
}

#[test]
fn verify_calamari_pallet_indices() {
    fn is_pallet_index<P: 'static>(index: usize) {
        assert_eq!(
            <calamari_runtime::Runtime as frame_system::Config>::PalletInfo::index::<P>(),
            Some(index)
        );
    }

    is_pallet_index::<calamari_runtime::System>(0);
    is_pallet_index::<calamari_runtime::ParachainSystem>(1);
    is_pallet_index::<calamari_runtime::Timestamp>(2);
    is_pallet_index::<calamari_runtime::ParachainInfo>(3);
    is_pallet_index::<calamari_runtime::TransactionPause>(9);

    is_pallet_index::<calamari_runtime::Balances>(10);
    is_pallet_index::<calamari_runtime::TransactionPayment>(11);

    is_pallet_index::<calamari_runtime::Democracy>(14);
    is_pallet_index::<calamari_runtime::Council>(15);
    is_pallet_index::<calamari_runtime::CouncilMembership>(16);
    is_pallet_index::<calamari_runtime::TechnicalCommittee>(17);
    is_pallet_index::<calamari_runtime::TechnicalMembership>(18);

    is_pallet_index::<calamari_runtime::ParachainStaking>(48);

    is_pallet_index::<calamari_runtime::AuthorInherent>(60);
    is_pallet_index::<calamari_runtime::AuraAuthorFilter>(63);

    is_pallet_index::<calamari_runtime::Authorship>(20);
    is_pallet_index::<calamari_runtime::CollatorSelection>(21);
    is_pallet_index::<calamari_runtime::Session>(22);
    is_pallet_index::<calamari_runtime::Aura>(23);

    is_pallet_index::<calamari_runtime::Treasury>(26);

    is_pallet_index::<calamari_runtime::Preimage>(28);

    is_pallet_index::<calamari_runtime::Scheduler>(29);

    is_pallet_index::<calamari_runtime::XcmpQueue>(30);
    is_pallet_index::<calamari_runtime::PolkadotXcm>(31);
    is_pallet_index::<calamari_runtime::CumulusXcm>(32);
    is_pallet_index::<calamari_runtime::DmpQueue>(33);
    is_pallet_index::<calamari_runtime::XTokens>(34);

    is_pallet_index::<calamari_runtime::Utility>(40);
    is_pallet_index::<calamari_runtime::Multisig>(41);

    is_pallet_index::<calamari_runtime::Assets>(45);
    is_pallet_index::<calamari_runtime::AssetManager>(46);
    is_pallet_index::<calamari_runtime::MantaPay>(47);
    is_pallet_index::<calamari_runtime::MantaSbt>(49);

    is_pallet_index::<calamari_runtime::CalamariVesting>(50);

    // Check removed pallets.
    ExtBuilder::default().build().execute_with(|| {
        use frame_support::metadata::{v14::META_RESERVED, RuntimeMetadata};

        let runtime_metadata = calamari_runtime::Runtime::metadata();
        assert_eq!(runtime_metadata.0, META_RESERVED);
        if let RuntimeMetadata::V14(v14) = runtime_metadata.1 {
            // Ensure sudo=42 has been removed, no one is taking this index.
            assert!(v14.pallets.iter().any(|pallet| pallet.index != 42));
            // AuraExt
            assert!(v14.pallets.iter().any(|pallet| pallet.index != 24));
        }
    });
}

#[test]
fn reward_fees_to_block_author_and_treasury() {
    ExtBuilder::default()
        .with_balances(vec![
            (ALICE.clone(), INITIAL_BALANCE),
            (BOB.clone(), INITIAL_BALANCE),
            (CHARLIE.clone(), INITIAL_BALANCE),
        ])
        .with_authorities(vec![(
            ALICE.clone(),
            SessionKeys::from_seed_unchecked("Alice"),
        )])
        .build()
        .execute_with(|| {
            let author = ALICE.clone();
            let mut header = seal_header(
                Header::new(
                    0,
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                ),
                author.clone(),
            );

            header.digest_mut().pop(); // pop the seal off.
            System::initialize(&1, &Default::default(), header.digest());
            assert_eq!(Authorship::author().unwrap(), author);

            let call = RuntimeCall::Balances(pallet_balances::Call::transfer {
                dest: sp_runtime::MultiAddress::Id(CHARLIE.clone()),
                value: 10 * KMA,
            });

            let len = 10;
            let info = info_from_weight(Weight::from_ref_time(100));
            let maybe_pre = ChargeTransactionPayment::<Runtime>::from(0)
                .pre_dispatch(&BOB, &call, &info, len)
                .unwrap();

            let res = call.dispatch(RuntimeOrigin::signed(BOB.clone()));

            let post_info = match res {
                Ok(info) => info,
                Err(err) => err.post_info,
            };

            let _res = ChargeTransactionPayment::<Runtime>::post_dispatch(
                Some(maybe_pre),
                &info,
                &post_info,
                len,
                &res.map(|_| ()).map_err(|e| e.error),
            );

            let author_received_reward = Balances::free_balance(ALICE.clone()) - INITIAL_BALANCE;
            println!("The rewarded_amount is: {author_received_reward:?}");

            // Fees split: 40% burned, 40% to treasury, 10% to author.
            let author_percent = Percent::from_percent(FEES_PERCENTAGE_TO_AUTHOR);
            let expected_fee =
                TransactionPayment::compute_actual_fee(len as u32, &info, &post_info, 0);
            assert_eq!(author_received_reward, author_percent * expected_fee);

            let treasury_percent = Percent::from_percent(FEES_PERCENTAGE_TO_TREASURY);
            assert_eq!(
                Balances::free_balance(Treasury::account_id()),
                treasury_percent * expected_fee
            );
        });
}
