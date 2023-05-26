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

#![cfg(feature = "manta")]

use super::{mock::ExtBuilder, *};
use frame_support::{
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

    is_pallet_prefix::<System>("System");
    is_pallet_prefix::<ParachainSystem>("ParachainSystem");
    is_pallet_prefix::<Timestamp>("Timestamp");
    is_pallet_prefix::<ParachainInfo>("ParachainInfo");

    is_pallet_prefix::<Balances>("Balances");
    is_pallet_prefix::<TransactionPayment>("TransactionPayment");

    is_pallet_prefix::<ParachainStaking>("ParachainStaking");

    is_pallet_prefix::<AuthorInherent>("AuthorInherent");
    is_pallet_prefix::<AuraAuthorFilter>("AuraAuthorFilter");

    is_pallet_prefix::<Authorship>("Authorship");
    is_pallet_prefix::<CollatorSelection>("CollatorSelection");
    is_pallet_prefix::<Session>("Session");
    is_pallet_prefix::<Aura>("Aura");

    is_pallet_prefix::<Treasury>("Treasury");

    is_pallet_prefix::<Preimage>("Preimage");

    is_pallet_prefix::<XcmpQueue>("XcmpQueue");
    is_pallet_prefix::<PolkadotXcm>("PolkadotXcm");
    is_pallet_prefix::<CumulusXcm>("CumulusXcm");
    is_pallet_prefix::<DmpQueue>("DmpQueue");
    is_pallet_prefix::<XTokens>("XTokens");

    is_pallet_prefix::<Utility>("Utility");
    is_pallet_prefix::<Multisig>("Multisig");

    is_pallet_prefix::<manta_runtime::Sudo>("Sudo");

    is_pallet_prefix::<Assets>("Assets");
    is_pallet_prefix::<AssetManager>("AssetManager");

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
fn verify_manta_pallet_indices() {
    fn is_pallet_index<P: 'static>(index: usize) {
        assert_eq!(
            <Runtime as frame_system::Config>::PalletInfo::index::<P>(),
            Some(index)
        );
    }

    is_pallet_index::<System>(0);
    is_pallet_index::<ParachainSystem>(1);
    is_pallet_index::<Timestamp>(2);
    is_pallet_index::<ParachainInfo>(3);

    is_pallet_index::<Balances>(10);
    is_pallet_index::<TransactionPayment>(11);

    is_pallet_index::<ParachainStaking>(48);

    is_pallet_index::<AuthorInherent>(60);
    is_pallet_index::<AuraAuthorFilter>(63);

    is_pallet_index::<Authorship>(20);
    is_pallet_index::<CollatorSelection>(21);
    is_pallet_index::<Session>(22);
    is_pallet_index::<Aura>(23);

    is_pallet_index::<Treasury>(26);

    is_pallet_index::<Preimage>(28);

    is_pallet_index::<XcmpQueue>(30);
    is_pallet_index::<PolkadotXcm>(31);
    is_pallet_index::<CumulusXcm>(32);
    is_pallet_index::<DmpQueue>(33);
    is_pallet_index::<XTokens>(34);

    is_pallet_index::<Utility>(40);
    is_pallet_index::<Multisig>(41);

    is_pallet_index::<manta_runtime::Sudo>(42);

    is_pallet_index::<Assets>(45);
    is_pallet_index::<AssetManager>(46);
}

#[test]
fn reward_fees_to_block_author_and_treasury() {
    ExtBuilder::default()
        .with_balances(vec![
            (ALICE.clone(), INITIAL_BALANCE),
            (BOB.clone(), INITIAL_BALANCE),
            (CHARLIE.clone(), INITIAL_BALANCE),
            (Treasury::account_id(), INITIAL_BALANCE),
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
                value: 10 * MANTA,
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

            // Fees split: 90% to treasury, 10% to author.
            let author_percent = Percent::from_percent(FEES_PERCENTAGE_TO_AUTHOR);
            let expected_fee =
                TransactionPayment::compute_actual_fee(len as u32, &info, &post_info, 0);
            assert_eq!(author_received_reward, author_percent * expected_fee);

            let treasury_percent = Percent::from_percent(FEES_PERCENTAGE_TO_TREASURY);
            assert_eq!(
                Balances::free_balance(Treasury::account_id()) - INITIAL_BALANCE,
                treasury_percent * expected_fee
            );
        });
}
