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

//! Tests for Manta-SBT

use crate::{
    mock::{new_test_ext, Balances, MantaSBTPallet, Origin as MockOrigin, Test},
    Error, ReservedIds,
};
use frame_support::{assert_noop, assert_ok, traits::Get};
use manta_crypto::{
    arkworks::constraint::fp::Fp,
    merkle_tree::{forest::TreeArrayMerkleForest, full::Full},
    rand::{CryptoRng, OsRng, RngCore},
};
use manta_pay::{
    config::{
        utxo::MerkleTreeConfiguration, MultiProvingContext, Parameters, UtxoAccumulatorModel,
    },
    parameters::{self, load_transfer_parameters, load_utxo_accumulator_model},
    test,
};
use manta_support::manta_pay::{
    field_from_id, id_from_field, AssetId, AssetValue, TransferPost as PalletTransferPost,
};

/// UTXO Accumulator for Building Circuits
type UtxoAccumulator =
    TreeArrayMerkleForest<MerkleTreeConfiguration, Full<MerkleTreeConfiguration>, 256>;

lazy_static::lazy_static! {
    static ref PROVING_CONTEXT: MultiProvingContext = load_proving_context();
    static ref PARAMETERS: Parameters = load_transfer_parameters();
    static ref UTXO_ACCUMULATOR_MODEL: UtxoAccumulatorModel = load_utxo_accumulator_model();
}

pub const ALICE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([0u8; 32]);

/// Turns vec! into BoundedVec
macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

/// Loads the [`MultiProvingContext`].
#[inline]
fn load_proving_context() -> MultiProvingContext {
    parameters::load_proving_context(
        tempfile::tempdir()
            .expect("Unable to create temporary directory.")
            .path(),
    )
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
        id_from_field(asset_id).unwrap().into(),
        value,
        rng,
    ))
    .unwrap()
}

/// Initializes a test by funding accounts and reserving_sbt
#[inline]
fn initialize_test() {
    assert_ok!(Balances::set_balance(
        MockOrigin::root(),
        ALICE,
        1_000_000_000_000_000,
        0
    ));
    assert_ok!(MantaSBTPallet::reserve_sbt(MockOrigin::signed(ALICE)));
}

/// Tests that single to_private tx works
#[test]
fn to_private_should_work() {
    let mut rng = OsRng;
    new_test_ext().execute_with(|| {
        initialize_test();
        let value = 1;
        let id = field_from_id(ReservedIds::<Test>::get(ALICE).unwrap().0);

        let post = sample_to_private(id, value, &mut rng);
        assert_ok!(MantaSBTPallet::to_private(
            MockOrigin::signed(ALICE),
            Box::new(post),
            bvec![0]
        ));
    });
}

/// Tests that it mints the number that corresponds with number of AssetIds incremented in AssetManager
#[test]
fn max_reserved_to_private_works() {
    let mut rng = OsRng;
    new_test_ext().execute_with(|| {
        initialize_test();
        let value = 1;
        let mints_per_reserve = <Test as crate::pallet::Config>::MintsPerReserve::get();
        for _ in 0..mints_per_reserve {
            let id = field_from_id(ReservedIds::<Test>::get(ALICE).unwrap().0);
            let post = sample_to_private(id, value, &mut rng);
            assert_ok!(MantaSBTPallet::to_private(
                MockOrigin::signed(ALICE),
                Box::new(post),
                bvec![0]
            ));
        }
    });
}

/// Tests that `ReservedIds` are successfully removed from storage after minting all the designated number SBTs
#[test]
#[should_panic]
fn overflow_reserved_ids_fails() {
    let mut rng = OsRng;
    new_test_ext().execute_with(|| {
        initialize_test();
        let value = 1;
        let mints_per_reserve: u16 = <Test as crate::pallet::Config>::MintsPerReserve::get();
        for _ in 0..mints_per_reserve + 1 {
            let id = field_from_id(ReservedIds::<Test>::get(ALICE).unwrap().0);
            let post = sample_to_private(id, value, &mut rng);
            assert_ok!(MantaSBTPallet::to_private(
                MockOrigin::signed(ALICE),
                Box::new(post),
                bvec![0]
            ));
        }
    });
}

/// Must `reserve_sbt` before minting
#[test]
fn not_reserved_fails() {
    let mut rng = OsRng;
    new_test_ext().execute_with(|| {
        let value = 1;
        let id = field_from_id(10);
        let post = sample_to_private(id, value, &mut rng);
        assert_noop!(
            MantaSBTPallet::to_private(MockOrigin::signed(ALICE), Box::new(post), bvec![0]),
            Error::<Test>::NotReserved
        );
    });
}

/// Private transfer will fail. `to_private` checks that post is correct shape.
#[test]
fn private_transfer_fails() {
    let mut rng = OsRng;
    new_test_ext().execute_with(|| {
        initialize_test();
        let value = 1;
        let id = field_from_id(ReservedIds::<Test>::get(ALICE).unwrap().0);

        assert_ok!(MantaSBTPallet::to_private(
            MockOrigin::signed(ALICE),
            Box::new(sample_to_private(id, value, &mut rng)),
            bvec![]
        ));
        let mut utxo_accumulator = UtxoAccumulator::new(UTXO_ACCUMULATOR_MODEL.clone());

        let (_, private_transfer) = test::payment::private_transfer::prove_full(
            &PROVING_CONTEXT,
            &PARAMETERS,
            &mut utxo_accumulator,
            id_from_field(id).unwrap().into(),
            [value, value],
            &mut rng,
        );

        let post = PalletTransferPost::try_from(private_transfer).unwrap();
        assert_noop!(
            MantaSBTPallet::to_private(MockOrigin::signed(ALICE), Box::new(post), bvec![]),
            Error::<Test>::InvalidShape
        );
    });
}

/// ToPublic Post should fail. `to_private` checks that post is correct shape
#[test]
fn to_public_fails() {
    let mut rng = OsRng;
    new_test_ext().execute_with(|| {
        initialize_test();
        let value = 1;
        let id = field_from_id(ReservedIds::<Test>::get(ALICE).unwrap().0);

        assert_ok!(MantaSBTPallet::to_private(
            MockOrigin::signed(ALICE),
            Box::new(sample_to_private(id, value, &mut rng)),
            bvec![]
        ));
        let mut utxo_accumulator = UtxoAccumulator::new(UTXO_ACCUMULATOR_MODEL.clone());

        let (_, private_transfer) = test::payment::to_public::prove_full(
            &PROVING_CONTEXT,
            &PARAMETERS,
            &mut utxo_accumulator,
            id_from_field(id).unwrap().into(),
            [value, value],
            &mut rng,
        );

        let post = PalletTransferPost::try_from(private_transfer).unwrap();
        assert_noop!(
            MantaSBTPallet::to_private(MockOrigin::signed(ALICE), Box::new(post), bvec![]),
            Error::<Test>::InvalidShape
        );
    });
}

/// Cannot mint arbitrary asset_ids
#[test]
fn wrong_asset_id_fails() {
    let mut rng = OsRng;

    new_test_ext().execute_with(|| {
        initialize_test();
        let asset_id = field_from_id(10);
        let value = 1;

        assert_noop!(
            MantaSBTPallet::to_private(
                MockOrigin::signed(ALICE),
                Box::new(sample_to_private(asset_id, value, &mut rng)),
                bvec![]
            ),
            Error::<Test>::InvalidAssetId
        );
    });
}

#[test]
fn only_value_of_one_allowed() {
    let mut rng = OsRng;
    new_test_ext().execute_with(|| {
        initialize_test();
        let value = 10;
        let id = field_from_id(ReservedIds::<Test>::get(ALICE).unwrap().0);

        assert_noop!(
            MantaSBTPallet::to_private(
                MockOrigin::signed(ALICE),
                Box::new(sample_to_private(id, value, &mut rng)),
                bvec![]
            ),
            Error::<Test>::ValueNotOne
        );
    });
}

/// Tests `PrivateTransfer` by manually calling `Ledger`, should already filter this out by checking post shape.
#[test]
fn private_transfer_ledger() {
    let mut rng = OsRng;
    new_test_ext().execute_with(|| {
        let asset_id = 10u128;
        let mut utxo_accumulator = UtxoAccumulator::new(UTXO_ACCUMULATOR_MODEL.clone());

        let ([to_private_0, to_private_1], private_transfer) =
            test::payment::private_transfer::prove_full(
                &PROVING_CONTEXT,
                &PARAMETERS,
                &mut utxo_accumulator,
                Fp::from(asset_id),
                // Divide by 2 in order to not exceed total_supply
                [100, 100],
                &mut rng,
            );

        assert_ok!(MantaSBTPallet::post_transaction(
            vec![ALICE],
            PalletTransferPost::try_from(to_private_0).unwrap(),
        ));
        assert_ok!(MantaSBTPallet::post_transaction(
            vec![ALICE],
            PalletTransferPost::try_from(to_private_1).unwrap(),
        ));
        assert_noop!(
            MantaSBTPallet::post_transaction(
                vec![],
                PalletTransferPost::try_from(private_transfer).unwrap(),
            ),
            Error::<Test>::NoSenderLedger
        );
    });
}

/// Tests `ToPublic` by manually calling `Ledger`, should already filter this out by checking post shape.
#[test]
fn to_public_sbt_fails() {
    let mut rng = OsRng;
    new_test_ext().execute_with(|| {
        let asset_id = 10u128;
        let mut utxo_accumulator = UtxoAccumulator::new(UTXO_ACCUMULATOR_MODEL.clone());
        let ([to_public_input_0, to_public_input_1], to_public) =
            test::payment::to_public::prove_full(
                &PROVING_CONTEXT,
                &PARAMETERS,
                &mut utxo_accumulator,
                Fp::from(asset_id),
                [100, 100],
                &mut rng,
            );

        assert_ok!(MantaSBTPallet::post_transaction(
            vec![ALICE],
            PalletTransferPost::try_from(to_public_input_0).unwrap(),
        ));
        assert_ok!(MantaSBTPallet::post_transaction(
            vec![ALICE],
            PalletTransferPost::try_from(to_public_input_1).unwrap(),
        ));
        assert_noop!(
            MantaSBTPallet::post_transaction(
                vec![],
                PalletTransferPost::try_from(to_public).unwrap(),
            ),
            Error::<Test>::InvalidShape
        );
    });
}

/// Increments Counter correctly
#[test]
fn sbt_counter_increments() {
    new_test_ext().execute_with(|| {
        let init_value = MantaSBTPallet::next_sbt_id_and_increment().unwrap();
        // initializes value to one
        assert_eq!(init_value, 1);
        let next_value = MantaSBTPallet::next_sbt_id_and_increment().unwrap();
        assert_eq!(next_value, 2);
    });
}
