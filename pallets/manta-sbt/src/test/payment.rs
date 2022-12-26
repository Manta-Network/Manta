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
    fp_decode,
    mock::{new_test_ext, Balances, ImplementItemId, MantaSBTPallet, Origin as MockOrigin, Test},
    types::{fp_encode, AssetId, AssetValue, TransferPost as PalletTransferPost},
    Error, IncrementItemId, ItemIdCounter,
};
use frame_support::{assert_noop, assert_ok};
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
        MantaSBTPallet::id_from_field(asset_id).unwrap().into(),
        value,
        rng,
    ))
    .unwrap()
}

/// Initializes a test by allocating `value`-many assets of the given `id` to the default account.
#[inline]
fn initialize_test() {
    assert_ok!(Balances::set_balance(
        MockOrigin::root(),
        ALICE,
        1_000_000_000_000_000,
        0
    ));
}

/// Tests multiple to_private from some total supply.
#[test]
fn to_private_should_work() {
    let mut rng = OsRng;

    for _ in 0..RANDOMIZED_TESTS_ITERATIONS {
        new_test_ext().execute_with(|| {
            initialize_test();
            let value = rng.gen();
            let id =
                ImplementItemId::encode_item_id(ItemIdCounter::<Test>::get().unwrap_or(0)).unwrap();

            assert_ok!(MantaSBTPallet::to_private(
                MockOrigin::signed(ALICE),
                sample_to_private(id, value, &mut rng)
            ));
        });
    }
}

#[test]
fn wrong_asset_id_fails() {
    let mut rng = OsRng;

    new_test_ext().execute_with(|| {
        initialize_test();
        let asset_id = MantaSBTPallet::field_from_id(10);
        let value = rng.gen();

        assert_noop!(
            MantaSBTPallet::to_private(
                MockOrigin::signed(ALICE),
                sample_to_private(asset_id, value, &mut rng)
            ),
            Error::<Test>::InvalidAssetId
        );
    })
}

#[test]
fn check_number_conversions() {
    let mut rng = OsRng;

    let start = rng.gen();
    let expected = MantaSBTPallet::field_from_id(start);

    let fp = Fp::<ConstraintField>::from(start);
    let encoded = fp_encode(fp).unwrap();

    assert_eq!(expected, encoded);

    let id_from_field = MantaSBTPallet::id_from_field(encoded).unwrap();
    let decoded: Fp<ConstraintField> = fp_decode(expected.to_vec()).unwrap();
    assert_eq!(start, id_from_field);
    assert_eq!(fp, decoded);
}

#[test]
fn pull_ledger_diff_should_work() {
    use scale_codec::Decode;

    new_test_ext().execute_with(|| {
        initialize_test();
        /*for _ in 0..2 {
            let mut rng = OsRng;
            let total_free_supply = rng.gen();
        }*/

        let (max_receivers, max_senders) = (128, 128);
        let check_point = crate::Checkpoint::default();
        let pull_response =
            MantaSBTPallet::pull_ledger_diff(check_point, max_receivers, max_senders);
        let dense_pull_response =
            MantaSBTPallet::dense_pull_ledger_diff(check_point, max_receivers, max_senders);
        assert_eq!(
            pull_response.senders_receivers_total,
            dense_pull_response.senders_receivers_total
        );
        assert_eq!(
            pull_response.should_continue,
            dense_pull_response.should_continue
        );
        assert_eq!(
            pull_response.should_continue,
            dense_pull_response.should_continue
        );

        let dense_receivers = base64::decode(dense_pull_response.receivers).unwrap();
        let mut slice_of = dense_receivers.as_slice();
        let decoded_receivers = <crate::ReceiverChunk as Decode>::decode(&mut slice_of).unwrap();
        assert_eq!(pull_response.receivers, decoded_receivers);

        let dense_senders = base64::decode(dense_pull_response.senders).unwrap();
        let mut slice_of = dense_senders.as_slice();
        let decoded_senders = <crate::SenderChunk as Decode>::decode(&mut slice_of).unwrap();
        assert_eq!(pull_response.senders, decoded_senders);
    });
}
