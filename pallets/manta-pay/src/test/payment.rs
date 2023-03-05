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

use crate::{
    fp_decode,
    mock::{
        new_test_ext, Assets, MantaAssetConfig, MantaAssetRegistry, MantaPay, Origin as MockOrigin,
        Test,
    },
    types::{fp_encode, AssetId, AssetValue, TransferPost as PalletTransferPost},
    Error, FungibleLedger, StandardAssetId,
};
use frame_support::{assert_noop, assert_ok};
use manta_accounting::transfer::test::value_distribution;
use manta_crypto::{
    arkworks::constraint::fp::Fp,
    merkle_tree::{forest::TreeArrayMerkleForest, full::Full},
    rand::{CryptoRng, OsRng, Rand, RngCore},
};
use manta_pay::{
    config::{
        utxo::MerkleTreeConfiguration, ConstraintField, MultiProvingContext, Parameters,
        UtxoAccumulatorModel,
    },
    parameters::{self, load_transfer_parameters, load_utxo_accumulator_model},
    test,
};

use manta_crypto::accumulator::Accumulator;
use manta_pay::config::{Asset, Authorization, FullParametersRef, Receiver, ToPrivate, ToPublic};
use manta_primitives::{
    assets::{
        AssetConfig, AssetRegistry, AssetRegistryMetadata, AssetStorageMetadata,
        FungibleLedger as _,
    },
    constants::{TEST_DEFAULT_ASSET_ED, TEST_DEFAULT_ASSET_ED2},
};
use std::{env, path::Path};

/// UTXO Accumulator for Building Circuits
type UtxoAccumulator =
    TreeArrayMerkleForest<MerkleTreeConfiguration, Full<MerkleTreeConfiguration>, 256>;

lazy_static::lazy_static! {
    static ref PROVING_CONTEXT: MultiProvingContext = load_proving_context();
    static ref PARAMETERS: Parameters = load_transfer_parameters();
    static ref UTXO_ACCUMULATOR_MODEL: UtxoAccumulatorModel = load_utxo_accumulator_model();
}

/// Loop randomized tests at least 10 times to reduce the change of false positives.
const RANDOMIZED_TESTS_ITERATIONS: usize = 10;

pub const ALICE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([0u8; 32]);
pub const NATIVE_ASSET_ID: StandardAssetId =
    <MantaAssetConfig as AssetConfig<Test>>::NativeAssetId::get();

/// Loads the [`MultiProvingContext`].
#[inline]
fn load_proving_context() -> MultiProvingContext {
    let env = env::var("MANTA_PROVING_DIR");
    if let Ok(path) = env {
        parameters::try_load_proving_context(Path::new(&path))
    } else {
        parameters::load_proving_context(
            tempfile::tempdir()
                .expect("Unable to create temporary directory.")
                .path(),
        )
    }
}

/// Samples a [`Mint`] transaction of `asset` with a random secret.
#[inline]
fn sample_to_private<R>(asset_id: AssetId, value: AssetValue, rng: &mut R) -> PalletTransferPost
where
    R: CryptoRng + RngCore + ?Sized,
{
    let mut utxo_accumulator = UtxoAccumulator::new(UTXO_ACCUMULATOR_MODEL.clone());
    PalletTransferPost::try_from(test::payment::to_private::prove_full(
        &PROVING_CONTEXT.to_private,
        &PARAMETERS,
        &mut utxo_accumulator,
        MantaPay::id_from_field(asset_id).unwrap().into(),
        value,
        rng,
    ))
    .unwrap()
}

/// Samples a [`ToPublic`] transaction of `asset` with a random secret.
#[inline]
fn sample_to_public<R>(asset_id: u128, value: [AssetValue; 2], rng: &mut R) -> PalletTransferPost
where
    R: CryptoRng + RngCore + ?Sized,
{
    let mut utxo_accumulator = UtxoAccumulator::new(UTXO_ACCUMULATOR_MODEL.clone());
    let ([_to_public_input_0, _to_public_input_1], to_public) =
        test::payment::to_public::prove_full(
            &PROVING_CONTEXT,
            &PARAMETERS,
            &mut utxo_accumulator,
            Fp::from(asset_id),
            value,
            rng,
        );
    PalletTransferPost::try_from(to_public).unwrap()
}

/// Mints many assets with the given `id` and `value`.
#[inline]
fn mint_private_tokens<R>(id: StandardAssetId, values: &[AssetValue], rng: &mut R)
where
    R: CryptoRng + RngCore + ?Sized,
{
    for value in values {
        assert_ok!(MantaPay::to_private(
            MockOrigin::signed(ALICE),
            sample_to_private(MantaPay::field_from_id(id), *value, rng)
        ));
    }
}

/// Builds `count`-many [`PrivateTransfer`] tests.
#[inline]
fn private_transfer_test<R>(
    count: usize,
    asset_id_option: Option<StandardAssetId>,
    rng: &mut R,
) -> Vec<PalletTransferPost>
where
    R: CryptoRng + RngCore + ?Sized,
{
    let asset_id = match asset_id_option {
        Some(id) => id,
        None => rng.gen(),
    };
    let total_free_balance: AssetValue = rng.gen();
    let balances = value_distribution(count, total_free_balance, rng);
    initialize_test(asset_id, total_free_balance + TEST_DEFAULT_ASSET_ED);
    let mut utxo_accumulator = UtxoAccumulator::new(UTXO_ACCUMULATOR_MODEL.clone());
    let mut posts = Vec::new();
    for balance in balances {
        let ([to_private_0, to_private_1], private_transfer) =
            test::payment::private_transfer::prove_full(
                &PROVING_CONTEXT,
                &PARAMETERS,
                &mut utxo_accumulator,
                Fp::from(asset_id),
                // Divide by 2 in order to not exceed total_supply
                [balance / 2, balance / 2],
                rng,
            );
        assert_ok!(MantaPay::to_private(
            MockOrigin::signed(ALICE),
            PalletTransferPost::try_from(to_private_0).unwrap()
        ));
        assert_ok!(MantaPay::to_private(
            MockOrigin::signed(ALICE),
            PalletTransferPost::try_from(to_private_1).unwrap()
        ));
        assert_ok!(MantaPay::private_transfer(
            MockOrigin::signed(ALICE),
            PalletTransferPost::try_from(private_transfer.clone()).unwrap(),
        ));

        posts.push(PalletTransferPost::try_from(private_transfer).unwrap())
    }
    posts
}

/// Builds `total`-many combined tests.
#[inline]
fn combined_test<R>(rng: &mut R, from: u128, to: u128, total: u128)
where
    R: CryptoRng + RngCore + ?Sized,
{
    for asset_id in from..to {
        initialize_test(
            asset_id,
            100_000_000_000_000_000_000 + TEST_DEFAULT_ASSET_ED,
        );
    }
    let mut utxo_accumulator = UtxoAccumulator::new(UTXO_ACCUMULATOR_MODEL.clone());
    let mut asset_id = 8u128;
    for i in 0..total {
        println!("Current: {i:?}");
        let mint0 = PalletTransferPost::try_from(test::payment::to_private::prove_full(
            &PROVING_CONTEXT.to_private,
            &PARAMETERS,
            &mut utxo_accumulator,
            asset_id.into(),
            1000,
            rng,
        ))
        .unwrap();

        let ([transfer_input_0, transfer_input_1], private_transfer) =
            test::payment::private_transfer::prove_full(
                &PROVING_CONTEXT,
                &PARAMETERS,
                &mut utxo_accumulator,
                Fp::from(asset_id),
                [100, 100],
                rng,
            );

        let ([to_public_input_0, to_public_input_1], to_public) =
            test::payment::to_public::prove_full(
                &PROVING_CONTEXT,
                &PARAMETERS,
                &mut utxo_accumulator,
                Fp::from(asset_id),
                [100, 100],
                rng,
            );

        assert_ok!(MantaPay::to_private(MockOrigin::signed(ALICE), mint0));

        assert_ok!(MantaPay::to_private(
            MockOrigin::signed(ALICE),
            PalletTransferPost::try_from(transfer_input_0).unwrap()
        ));
        assert_ok!(MantaPay::to_private(
            MockOrigin::signed(ALICE),
            PalletTransferPost::try_from(transfer_input_1).unwrap()
        ));
        assert_ok!(MantaPay::private_transfer(
            MockOrigin::signed(ALICE),
            PalletTransferPost::try_from(private_transfer.clone()).unwrap(),
        ));

        assert_ok!(MantaPay::to_private(
            MockOrigin::signed(ALICE),
            PalletTransferPost::try_from(to_public_input_0).unwrap()
        ));
        assert_ok!(MantaPay::to_private(
            MockOrigin::signed(ALICE),
            PalletTransferPost::try_from(to_public_input_1).unwrap()
        ));
        assert_ok!(MantaPay::to_public(
            MockOrigin::signed(ALICE),
            PalletTransferPost::try_from(to_public.clone()).unwrap()
        ));

        asset_id += 1;
        if asset_id == to {
            asset_id = from;
        }
    }
}

/// Builds `count`-many [`Reclaim`] tests.
fn reclaim_test<R>(
    count: usize,
    total_supply: AssetValue,
    id_option: Option<StandardAssetId>,
    rng: &mut R,
) -> Vec<PalletTransferPost>
where
    R: CryptoRng + RngCore + ?Sized,
{
    let asset_id = match id_option {
        Some(id) => id,
        None => rng.gen(),
    };
    let balances = value_distribution(count, total_supply, rng);
    initialize_test(asset_id, total_supply + TEST_DEFAULT_ASSET_ED);
    let mut utxo_accumulator = UtxoAccumulator::new(UTXO_ACCUMULATOR_MODEL.clone());
    let mut posts = Vec::new();
    for balance in balances {
        let ([to_private_0, to_private_1], to_public) = test::payment::to_public::prove_full(
            &PROVING_CONTEXT,
            &PARAMETERS,
            &mut utxo_accumulator,
            Fp::from(asset_id),
            // Divide by 2 in order to not exceed total_supply
            [balance / 2, balance / 2],
            rng,
        );
        assert_ok!(MantaPay::to_private(
            MockOrigin::signed(ALICE),
            PalletTransferPost::try_from(to_private_0).unwrap()
        ));
        assert_ok!(MantaPay::to_private(
            MockOrigin::signed(ALICE),
            PalletTransferPost::try_from(to_private_1).unwrap()
        ));
        assert_ok!(MantaPay::to_public(
            MockOrigin::signed(ALICE),
            PalletTransferPost::try_from(to_public.clone()).unwrap()
        ));
        posts.push(PalletTransferPost::try_from(to_public).unwrap());
    }
    posts
}

/// Initializes a test by allocating `value`-many assets of the given `id` to the default account.
#[inline]
fn initialize_test(id: StandardAssetId, value: AssetValue) {
    let metadata = AssetRegistryMetadata {
        metadata: AssetStorageMetadata {
            name: b"Calamari".to_vec(),
            symbol: b"KMA".to_vec(),
            decimals: 12,
            is_frozen: false,
        },
        min_balance: TEST_DEFAULT_ASSET_ED,
        is_sufficient: true,
    };
    assert_ok!(MantaAssetRegistry::create_asset(
        id,
        metadata.into(),
        TEST_DEFAULT_ASSET_ED,
        true
    ));
    assert_ok!(FungibleLedger::<Test>::deposit_minting(id, &ALICE, value));
    assert_ok!(FungibleLedger::<Test>::deposit_minting(
        id,
        &MantaPay::account_id(),
        TEST_DEFAULT_ASSET_ED
    ));
}

/// Initializes a test by allocating `value` amount asset to ALICE.
fn initialize_test_wt_deposit_pallet(id: StandardAssetId, value: AssetValue) {
    let metadata = AssetRegistryMetadata {
        metadata: AssetStorageMetadata {
            name: b"Calamari".to_vec(),
            symbol: b"KMA".to_vec(),
            decimals: 12,
            is_frozen: false,
        },
        min_balance: TEST_DEFAULT_ASSET_ED2,
        is_sufficient: true,
    };
    assert_ok!(MantaAssetRegistry::create_asset(
        id,
        metadata.into(),
        TEST_DEFAULT_ASSET_ED2,
        true
    ));
    assert_ok!(FungibleLedger::<Test>::deposit_minting(id, &ALICE, value));
}

/// Tests multiple to_private from some total supply.
#[test]
fn to_private_should_work() {
    let mut rng = OsRng;
    for _ in 0..RANDOMIZED_TESTS_ITERATIONS {
        new_test_ext().execute_with(|| {
            let asset_id = rng.gen();
            let total_free_supply = rng.gen();
            initialize_test(asset_id, total_free_supply + TEST_DEFAULT_ASSET_ED);
            mint_private_tokens(
                asset_id,
                &value_distribution(5, total_free_supply, &mut rng),
                &mut rng,
            );
        });
    }
}

/// Tests to_private with zero balance should failed.
#[test]
fn to_private_with_zero_should_not_work() {
    let mut rng = OsRng;
    new_test_ext().execute_with(|| {
        let asset_id = rng.gen();
        let total_free_supply: AssetValue = rng.gen();
        initialize_test(asset_id, total_free_supply + TEST_DEFAULT_ASSET_ED);
        assert_noop!(
            MantaPay::to_private(
                MockOrigin::signed(ALICE),
                sample_to_private(MantaPay::field_from_id(asset_id), 0, &mut rng)
            ),
            Error::<Test>::ZeroTransfer
        );
    });
}

/// Tests to_public with zero balance should failed.
#[test]
fn to_public_with_zero_should_not_work() {
    let mut rng = OsRng;
    new_test_ext().execute_with(|| {
        let asset_id = rng.gen();
        let total_free_supply: AssetValue = rng.gen();
        initialize_test(asset_id, total_free_supply + TEST_DEFAULT_ASSET_ED);
        assert_noop!(
            MantaPay::to_public(
                MockOrigin::signed(ALICE),
                sample_to_public(asset_id, [0, 0], &mut rng)
            ),
            Error::<Test>::ZeroTransfer
        );
    });
}

/// Tests a [`ToPrivate`] transaction with native currency.
#[test]
fn native_asset_to_private_should_work() {
    let mut rng = OsRng;
    for _ in 0..RANDOMIZED_TESTS_ITERATIONS {
        new_test_ext().execute_with(|| {
            let total_free_supply = rng.gen();
            initialize_test(NATIVE_ASSET_ID, total_free_supply + TEST_DEFAULT_ASSET_ED);
            mint_private_tokens(
                NATIVE_ASSET_ID,
                &value_distribution(5, total_free_supply, &mut rng),
                &mut rng,
            );
        });
    }
}

/// Tests a mint that would overdraw the total supply.
#[test]
fn overdrawn_mint_should_not_work() {
    let mut rng = OsRng;
    for _ in 0..RANDOMIZED_TESTS_ITERATIONS {
        new_test_ext().execute_with(|| {
            let asset_id = rng.gen();
            let total_supply: u128 = rng.gen();
            initialize_test(asset_id, total_supply + TEST_DEFAULT_ASSET_ED);
            assert_noop!(
                MantaPay::to_private(
                    MockOrigin::signed(ALICE),
                    sample_to_private(
                        MantaPay::field_from_id(asset_id),
                        total_supply + TEST_DEFAULT_ASSET_ED + 1,
                        &mut rng
                    )
                ),
                Error::<Test>::InvalidSourceAccount
            );
        });
    }
}

/// Tests a mint that would overdraw from a non-existent supply.
#[test]
fn to_private_without_init_should_not_work() {
    let mut rng = OsRng;
    for _ in 0..RANDOMIZED_TESTS_ITERATIONS {
        new_test_ext().execute_with(|| {
            assert_noop!(
                MantaPay::to_private(
                    MockOrigin::signed(ALICE),
                    sample_to_private(MantaPay::field_from_id(rng.gen()), 100, &mut rng)
                ),
                Error::<Test>::InvalidSourceAccount,
            );
        });
    }
}

/// Tests that a double-spent [`Mint`] will fail.
#[test]
fn mint_existing_coin_should_not_work() {
    let mut rng = OsRng;
    for _ in 0..RANDOMIZED_TESTS_ITERATIONS {
        new_test_ext().execute_with(|| {
            let asset_id = rng.gen();
            initialize_test(asset_id, 32579u128);
            let mint_post = sample_to_private(MantaPay::field_from_id(asset_id), 100, &mut rng);
            assert_ok!(MantaPay::to_private(
                MockOrigin::signed(ALICE),
                mint_post.clone()
            ));
            assert_noop!(
                MantaPay::to_private(MockOrigin::signed(ALICE), mint_post),
                Error::<Test>::AssetRegistered
            );
        });
    }
}

/// Tests a [`PrivateTransfer`] transaction.
#[test]
fn private_transfer_should_work() {
    for _ in 0..RANDOMIZED_TESTS_ITERATIONS {
        new_test_ext().execute_with(|| private_transfer_test(10, None, &mut OsRng));
    }
}

/// Test a [`PrivateTransfer`] transaction with native currency
#[test]
fn private_transfer_native_asset_should_work() {
    for _ in 0..RANDOMIZED_TESTS_ITERATIONS {
        new_test_ext().execute_with(|| {
            private_transfer_test(10, Some(NATIVE_ASSET_ID), &mut OsRng);
        });
    }
}

/// Tests multiple [`PrivateTransfer`] transactions.
#[test]
fn private_transfer_10_times_should_work() {
    for _ in 0..RANDOMIZED_TESTS_ITERATIONS {
        new_test_ext().execute_with(|| private_transfer_test(10, None, &mut OsRng));
    }
}

/// Tests that a double-spent [`PrivateTransfer`] will fail.
#[test]
fn double_spend_in_private_transfer_should_not_work() {
    for _ in 0..RANDOMIZED_TESTS_ITERATIONS {
        new_test_ext().execute_with(|| {
            for private_transfer in private_transfer_test(10, None, &mut OsRng) {
                assert_noop!(
                    MantaPay::private_transfer(MockOrigin::signed(ALICE), private_transfer),
                    Error::<Test>::AssetSpent,
                );
            }
        });
    }
}

/// Tests a [`Reclaim`] transaction.
#[test]
fn reclaim_should_work() {
    let mut rng = OsRng;
    for _ in 0..RANDOMIZED_TESTS_ITERATIONS {
        new_test_ext().execute_with(|| reclaim_test(10, rng.gen(), None, &mut rng));
    }
}

/// Test a [`Reclaim`] of native currency
#[test]
fn reclaim_native_should_work() {
    let mut rng = OsRng;
    for _ in 0..RANDOMIZED_TESTS_ITERATIONS {
        new_test_ext()
            .execute_with(|| reclaim_test(10, rng.gen(), Some(NATIVE_ASSET_ID), &mut rng));
    }
}

/// Tests multiple [`Reclaim`] transactions.
#[test]
fn reclaim_10_times_should_work() {
    let mut rng = OsRng;
    for _ in 0..RANDOMIZED_TESTS_ITERATIONS {
        new_test_ext().execute_with(|| reclaim_test(10, rng.gen(), None, &mut rng));
    }
}

/// Tests multiple sequences of ToPrivate, ToPrivate, ToPrivate, ToPrivateTransfer, ToPrivate, ToPrivate, ToPublic
#[test]
fn combined_should_work() {
    let mut rng = OsRng;
    new_test_ext().execute_with(|| combined_test(&mut rng, 8u128, 18u128, 50));
}

/// Tests that a double-spent [`Reclaim`] will fail.
#[test]
fn double_spend_in_reclaim_should_not_work() {
    for _ in 0..RANDOMIZED_TESTS_ITERATIONS {
        new_test_ext().execute_with(|| {
            let mut rng = OsRng;
            let total_supply: u128 = rng.gen();
            for reclaim in reclaim_test(10, total_supply / 2, None, &mut rng) {
                assert_noop!(
                    MantaPay::to_public(MockOrigin::signed(ALICE), reclaim),
                    Error::<Test>::AssetSpent,
                );
            }
        });
    }
}

#[test]
fn public_lower_than_ed_should_not_work0() {
    let mut rng = OsRng;
    new_test_ext().execute_with(|| {
        let asset_id = rng.gen();
        let value = 1000u128;
        initialize_test_wt_deposit_pallet(asset_id, value * 4);

        let asset_info = Assets::balance(asset_id, MantaPay::account_id());
        assert_eq!(asset_info, 0);
        let asset_info = Assets::balance(asset_id, ALICE.clone());
        assert_eq!(asset_info, value * 4);

        let mut utxo_accumulator = UtxoAccumulator::new(UTXO_ACCUMULATOR_MODEL.clone());
        let spending_key = rng.gen();
        let address = PARAMETERS.address_from_spending_key(&spending_key);
        let mut authorization =
            Authorization::from_spending_key(&PARAMETERS, &spending_key, &mut rng);
        let asset_0 = Asset::new(Fp::from(asset_id), value);
        let asset_1 = Asset::new(Fp::from(asset_id), value);

        // First ToPrivate
        let (to_private_0, pre_sender_0) = ToPrivate::internal_pair(
            &PARAMETERS,
            &mut authorization.context,
            address,
            asset_0,
            Default::default(),
            &mut rng,
        );
        let sender_0 = pre_sender_0
            .insert_and_upgrade(&PARAMETERS, &mut utxo_accumulator)
            .expect("");
        let to_private_0 = to_private_0
            .into_post(
                FullParametersRef::new(&PARAMETERS, utxo_accumulator.model()),
                &PROVING_CONTEXT.to_private,
                None,
                &mut rng,
            )
            .expect("Unable to build TO_PRIVATE proof.")
            .expect("Did not match transfer shape.");
        assert_ok!(MantaPay::to_private(
            MockOrigin::signed(ALICE),
            PalletTransferPost::try_from(to_private_0).unwrap()
        ));
        let asset_info = Assets::balance(asset_id, MantaPay::account_id());
        assert_eq!(asset_info, value);

        // Second ToPrivate
        let (to_private_1, pre_sender_1) = ToPrivate::internal_pair(
            &PARAMETERS,
            &mut authorization.context,
            address,
            asset_1,
            Default::default(),
            &mut rng,
        );
        let sender_1 = pre_sender_1
            .insert_and_upgrade(&PARAMETERS, &mut utxo_accumulator)
            .expect("");
        let to_private_1 = to_private_1
            .into_post(
                FullParametersRef::new(&PARAMETERS, utxo_accumulator.model()),
                &PROVING_CONTEXT.to_private,
                None,
                &mut rng,
            )
            .expect("Unable to build TO_PRIVATE proof.")
            .expect("Did not match transfer shape.");
        assert_ok!(MantaPay::to_private(
            MockOrigin::signed(ALICE),
            PalletTransferPost::try_from(to_private_1).unwrap()
        ));
        let asset_info = Assets::balance(asset_id, MantaPay::account_id());
        assert_eq!(asset_info, value * 2);

        // ToPublic will use previous two ToPrivate.
        // 0,2000 if ED=1 -> PublicUpdateInvalidTransfer
        // 1,1999 if ED=2 -> PublicUpdateInvalidTransfer
        let asset_2 = Asset::new(Fp::from(asset_id), 1);
        let asset_3 = Asset::new(Fp::from(asset_id), 1999);
        let receiver_1 =
            Receiver::sample(&PARAMETERS, address, asset_2, Default::default(), &mut rng);
        receiver_1.insert_utxo(&PARAMETERS, &mut utxo_accumulator);
        let to_public = ToPublic::build(
            authorization,
            [sender_0.clone(), sender_1.clone()],
            [receiver_1],
            asset_3,
        )
        .into_post(
            FullParametersRef::new(&PARAMETERS, utxo_accumulator.model()),
            &PROVING_CONTEXT.to_public,
            Some(&spending_key),
            &mut rng,
        )
        .expect("Unable to build TO_PUBLIC proof.")
        .expect("");
        assert_noop!(
            MantaPay::to_public(
                MockOrigin::signed(ALICE),
                PalletTransferPost::try_from(to_public).unwrap()
            ),
            Error::<Test>::PublicUpdateInvalidTransfer
        );

        // 2,1998 if ED=2 -> ToPublic OK
        let asset_2 = Asset::new(Fp::from(asset_id), 2);
        let asset_3 = Asset::new(Fp::from(asset_id), 1998);
        let receiver_1 =
            Receiver::sample(&PARAMETERS, address, asset_2, Default::default(), &mut rng);
        receiver_1.insert_utxo(&PARAMETERS, &mut utxo_accumulator);
        let to_public = ToPublic::build(authorization, [sender_0, sender_1], [receiver_1], asset_3)
            .into_post(
                FullParametersRef::new(&PARAMETERS, utxo_accumulator.model()),
                &PROVING_CONTEXT.to_public,
                Some(&spending_key),
                &mut rng,
            )
            .expect("Unable to build TO_PUBLIC proof.")
            .expect("");
        assert_ok!(MantaPay::to_public(
            MockOrigin::signed(ALICE),
            PalletTransferPost::try_from(to_public).unwrap()
        ));
        let asset_info = Assets::balance(asset_id, MantaPay::account_id());
        assert_eq!(asset_info, 2);
    });
}

#[test]
fn check_number_conversions() {
    let mut rng = OsRng;

    let start = rng.gen();
    let expected = MantaPay::field_from_id(start);

    let fp = Fp::<ConstraintField>::from(start);
    let encoded = fp_encode(fp).unwrap();

    assert_eq!(expected, encoded);

    let id_from_field = MantaPay::id_from_field(encoded).unwrap();
    let decoded: Fp<ConstraintField> = fp_decode(expected.to_vec()).unwrap();
    assert_eq!(start, id_from_field);
    assert_eq!(fp, decoded);
}

#[test]
fn pull_ledger_diff_should_work() {
    use scale_codec::Decode;
    new_test_ext().execute_with(|| {
        for _ in 0..2 {
            let mut rng = OsRng;
            let asset_id = rng.gen();
            let total_free_supply = rng.gen();
            initialize_test(asset_id, total_free_supply + TEST_DEFAULT_ASSET_ED);
            mint_private_tokens(
                asset_id,
                &value_distribution(5, total_free_supply, &mut rng),
                &mut rng,
            );
        }

        // retrieving 128 receivers/senders can get all existing Utxos
        let (max_receivers, max_senders) = (128, 128);
        let check_point = crate::Checkpoint::default();
        let runtime_pull_response =
            MantaPay::pull_ledger_diff(check_point, max_receivers, max_senders);

        // ensure all Utxos have been returned.
        assert!(!runtime_pull_response.should_continue);

        // convert runtime response into native response
        let dense_pull_response: crate::types::DensePullResponse =
            runtime_pull_response.clone().into();

        let dense_receivers = base64::decode(dense_pull_response.receivers).unwrap();
        let mut slice_of = dense_receivers.as_slice();
        let decoded_receivers = <crate::ReceiverChunk as Decode>::decode(&mut slice_of).unwrap();
        assert_eq!(runtime_pull_response.receivers, decoded_receivers);

        let dense_senders = base64::decode(dense_pull_response.senders).unwrap();
        let mut slice_of = dense_senders.as_slice();
        let decoded_senders = <crate::SenderChunk as Decode>::decode(&mut slice_of).unwrap();
        assert_eq!(runtime_pull_response.senders, decoded_senders);
    });
}
