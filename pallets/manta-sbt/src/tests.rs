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
    mock::{new_test_ext, Balances, MantaSBTPallet, Origin as MockOrigin, Test, Timestamp},
    AllowlistAccount, DispatchError, Error, EvmAddress, EvmAddressAllowlist, EvmAddressType,
    MintChainInfo, MintChainInfos, MintStatus, MintType, Moment, ReservedIds, SbtMetadata,
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
use sp_io::hashing::keccak_256;

/// UTXO Accumulator for Building Circuits
type UtxoAccumulator =
    TreeArrayMerkleForest<MerkleTreeConfiguration, Full<MerkleTreeConfiguration>, 256>;

lazy_static::lazy_static! {
    static ref PROVING_CONTEXT: MultiProvingContext = load_proving_context();
    static ref PARAMETERS: Parameters = load_transfer_parameters();
    static ref UTXO_ACCUMULATOR_MODEL: UtxoAccumulatorModel = load_utxo_accumulator_model();
}

pub const ALICE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([0u8; 32]);
pub const BOB: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([1u8; 32]);

/// Alice eth account
pub fn alice_eth() -> libsecp256k1::SecretKey {
    libsecp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap()
}

/// Bob eth account
pub fn bob_eth() -> libsecp256k1::SecretKey {
    libsecp256k1::SecretKey::parse(&keccak_256(b"Bob")).unwrap()
}

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
        assert_eq!(SbtMetadata::<Test>::get(1).unwrap().extra, Some(bvec![0]));
        assert_eq!(
            SbtMetadata::<Test>::get(1).unwrap().mint_type,
            MintType::Manta
        );
    });
}

/// Tests that it mints the number that corresponds with number of AssetIds reserved
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
fn overflow_reserved_ids_fails() {
    let mut rng = OsRng;
    new_test_ext().execute_with(|| {
        initialize_test();
        let value = 1;
        let mints_per_reserve: u16 = <Test as crate::pallet::Config>::MintsPerReserve::get();
        for i in 0..mints_per_reserve + 1 {
            if i < mints_per_reserve {
                let id = field_from_id(ReservedIds::<Test>::get(ALICE).unwrap().0);
                let post = sample_to_private(id, value, &mut rng);
                assert!(ReservedIds::<Test>::contains_key(ALICE));
                assert_ok!(MantaSBTPallet::to_private(
                    MockOrigin::signed(ALICE),
                    Box::new(post),
                    bvec![0]
                ));
            } else {
                assert!(!ReservedIds::<Test>::contains_key(ALICE));
            }
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
            ALICE.into(),
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
                ALICE.into(),
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

#[test]
fn change_allowlist_account_works() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            MantaSBTPallet::change_allowlist_account(MockOrigin::signed(ALICE), Some(ALICE)),
            DispatchError::BadOrigin
        );
        assert_eq!(AllowlistAccount::<Test>::get(), None);

        assert_ok!(MantaSBTPallet::change_allowlist_account(
            MockOrigin::root(),
            Some(ALICE)
        ));
        assert_eq!(AllowlistAccount::<Test>::get().unwrap(), ALICE);
        assert_ok!(MantaSBTPallet::change_allowlist_account(
            MockOrigin::root(),
            None
        ));
        assert_eq!(AllowlistAccount::<Test>::get(), None);
    })
}

#[test]
fn allowlist_account_works() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            MantaSBTPallet::allowlist_evm_account(
                MockOrigin::signed(ALICE),
                EvmAddressType::Bab(EvmAddress::default())
            ),
            Error::<Test>::MintNotAvailable,
        );
        assert_ok!(MantaSBTPallet::set_mint_chain_info(
            MockOrigin::root(),
            MintType::Bab,
            5,
            None
        ));
        Timestamp::set_timestamp(2);
        assert_noop!(
            MantaSBTPallet::allowlist_evm_account(
                MockOrigin::signed(ALICE),
                EvmAddressType::Bab(EvmAddress::default())
            ),
            Error::<Test>::MintNotAvailable,
        );

        Timestamp::set_timestamp(10);
        assert_noop!(
            MantaSBTPallet::allowlist_evm_account(
                MockOrigin::signed(ALICE),
                EvmAddressType::Bab(EvmAddress::default())
            ),
            Error::<Test>::NotAllowlistAccount,
        );
        assert_eq!(
            EvmAddressAllowlist::<Test>::get(EvmAddressType::Bab(EvmAddress::default())),
            None
        );

        assert_ok!(MantaSBTPallet::change_allowlist_account(
            MockOrigin::root(),
            Some(ALICE)
        ));
        assert_ok!(MantaSBTPallet::allowlist_evm_account(
            MockOrigin::signed(ALICE),
            EvmAddressType::Bab(EvmAddress::default())
        ));
        assert_eq!(
            EvmAddressAllowlist::<Test>::get(EvmAddressType::Bab(EvmAddress::default())).unwrap(),
            MintStatus::Available(1)
        );

        assert_noop!(
            MantaSBTPallet::allowlist_evm_account(
                MockOrigin::signed(ALICE),
                EvmAddressType::Bab(EvmAddress::default())
            ),
            Error::<Test>::AlreadyInAllowlist,
        );

        assert_noop!(
            MantaSBTPallet::allowlist_evm_account(
                MockOrigin::signed(BOB),
                EvmAddressType::Bab(EvmAddress::default())
            ),
            Error::<Test>::NotAllowlistAccount,
        );
    })
}

#[test]
fn mint_sbt_eth_works() {
    let mut rng = OsRng;
    new_test_ext().execute_with(|| {
        assert_ok!(MantaSBTPallet::change_allowlist_account(
            MockOrigin::root(),
            Some(ALICE)
        ));
        let evm_mint_type = EvmAddressType::Bab(MantaSBTPallet::eth_address(&alice_eth()));
        Timestamp::set_timestamp(10);
        assert_ok!(MantaSBTPallet::set_mint_chain_info(
            MockOrigin::root(),
            MintType::Bab,
            0,
            None
        ));

        let value = 1;
        let storage_id = 1;
        let id = field_from_id(storage_id);
        let post = Box::new(sample_to_private(id, value, &mut rng));

        // Account has not been allowlisted
        assert_noop!(
            MantaSBTPallet::mint_sbt_eth(
                MockOrigin::signed(ALICE),
                post.clone(),
                1,
                MantaSBTPallet::eth_sign(&alice_eth(), &post.proof, 1),
                evm_mint_type,
                Some(0),
                Some(0),
                Some(bvec![0])
            ),
            Error::<Test>::NotAllowlisted
        );
        // allowlist account
        assert_ok!(MantaSBTPallet::allowlist_evm_account(
            MockOrigin::signed(ALICE),
            evm_mint_type
        ));

        // wrong signature fails
        assert_noop!(
            MantaSBTPallet::mint_sbt_eth(
                MockOrigin::signed(ALICE),
                post.clone(),
                1,
                MantaSBTPallet::eth_sign(&alice_eth(), &[0; 128], 1),
                evm_mint_type,
                Some(0),
                Some(0),
                Some(bvec![0])
            ),
            Error::<Test>::BadSignature
        );
        assert_noop!(
            MantaSBTPallet::mint_sbt_eth(
                MockOrigin::signed(ALICE),
                post.clone(),
                1,
                MantaSBTPallet::eth_sign(&bob_eth(), &[0; 128], 1),
                evm_mint_type,
                Some(0),
                Some(0),
                Some(bvec![0])
            ),
            Error::<Test>::BadSignature
        );
        assert_noop!(
            MantaSBTPallet::mint_sbt_eth(
                MockOrigin::signed(ALICE),
                post.clone(),
                1,
                MantaSBTPallet::eth_sign(&bob_eth(), &post.proof, 2),
                evm_mint_type,
                Some(0),
                Some(0),
                Some(bvec![0])
            ),
            Error::<Test>::BadSignature
        );
        assert_noop!(
            MantaSBTPallet::mint_sbt_eth(
                MockOrigin::signed(ALICE),
                post.clone(),
                2,
                MantaSBTPallet::eth_sign(&bob_eth(), &post.proof, 1),
                evm_mint_type,
                Some(0),
                Some(0),
                Some(bvec![0])
            ),
            Error::<Test>::BadSignature
        );

        assert_ok!(MantaSBTPallet::mint_sbt_eth(
            MockOrigin::signed(ALICE),
            post.clone(),
            1,
            MantaSBTPallet::eth_sign(&alice_eth(), &post.proof, 1),
            evm_mint_type,
            Some(0),
            Some(0),
            Some(bvec![0])
        ));
        let sbt_metadata = SbtMetadata::<Test>::get(1).unwrap();
        assert_eq!(sbt_metadata.collection_id, Some(0));
        assert_eq!(sbt_metadata.item_id, Some(0));
        assert_eq!(sbt_metadata.extra, Some(bvec![0]));
        assert_eq!(sbt_metadata.mint_type, MintType::Bab);

        // Account is already minted
        assert_eq!(
            EvmAddressAllowlist::<Test>::get(evm_mint_type).unwrap(),
            MintStatus::AlreadyMinted
        );
        assert_noop!(
            MantaSBTPallet::mint_sbt_eth(
                MockOrigin::signed(ALICE),
                post.clone(),
                1,
                MantaSBTPallet::eth_sign(&alice_eth(), &post.proof, 1),
                evm_mint_type,
                Some(0),
                Some(0),
                Some(bvec![0])
            ),
            Error::<Test>::AlreadyMinted
        );
    })
}

#[test]
fn timestamp_range_fails() {
    let mut rng = OsRng;
    new_test_ext().execute_with(|| {
        assert_ok!(MantaSBTPallet::change_allowlist_account(
            MockOrigin::root(),
            Some(ALICE)
        ));
        assert_ok!(MantaSBTPallet::set_mint_chain_info(
            MockOrigin::root(),
            MintType::Bab,
            0,
            None
        ));
        Timestamp::set_timestamp(1);
        let evm_mint_type = EvmAddressType::Bab(MantaSBTPallet::eth_address(&alice_eth()));
        assert_ok!(MantaSBTPallet::allowlist_evm_account(
            MockOrigin::signed(ALICE),
            evm_mint_type
        ));
        assert_ok!(MantaSBTPallet::set_mint_chain_info(
            MockOrigin::root(),
            MintType::Bab,
            10,
            Some(20)
        ));

        let value = 1;
        let storage_id = match EvmAddressAllowlist::<Test>::get(evm_mint_type).unwrap() {
            MintStatus::Available(asset_id) => asset_id,
            MintStatus::AlreadyMinted => panic!("should not be minted"),
        };
        let id = field_from_id(storage_id);
        let post = Box::new(sample_to_private(id, value, &mut rng));

        // set timestamp too early
        Timestamp::set_timestamp(5);
        assert_noop!(
            MantaSBTPallet::mint_sbt_eth(
                MockOrigin::signed(ALICE),
                post.clone(),
                1,
                MantaSBTPallet::eth_sign(&alice_eth(), &post.proof, 0),
                evm_mint_type,
                Some(0),
                Some(0),
                Some(bvec![0])
            ),
            Error::<Test>::MintNotAvailable
        );

        // set timestamp too late
        Timestamp::set_timestamp(25);
        assert_noop!(
            MantaSBTPallet::mint_sbt_eth(
                MockOrigin::signed(ALICE),
                post.clone(),
                1,
                MantaSBTPallet::eth_sign(&alice_eth(), &post.proof, 1),
                evm_mint_type,
                Some(0),
                Some(0),
                Some(bvec![0])
            ),
            Error::<Test>::MintNotAvailable
        );

        // with None as end time works
        assert_ok!(MantaSBTPallet::set_mint_chain_info(
            MockOrigin::root(),
            MintType::Bab,
            10,
            None
        ));
        assert_ok!(MantaSBTPallet::mint_sbt_eth(
            MockOrigin::signed(ALICE),
            post.clone(),
            1,
            MantaSBTPallet::eth_sign(&alice_eth(), &post.proof, 1),
            evm_mint_type,
            Some(0),
            Some(0),
            Some(bvec![0])
        ));
    })
}

#[test]
fn set_mint_chain_info_works() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            MantaSBTPallet::set_mint_chain_info(MockOrigin::signed(ALICE), MintType::Bab, 0, None),
            DispatchError::BadOrigin
        );
        assert_noop!(
            MantaSBTPallet::set_mint_chain_info(MockOrigin::root(), MintType::Bab, 10, Some(5)),
            Error::<Test>::InvalidTimeRange
        );

        assert_ok!(MantaSBTPallet::set_mint_chain_info(
            MockOrigin::root(),
            MintType::Bab,
            0,
            None
        ));
        assert_eq!(
            MintChainInfos::<Test>::get(MintType::Bab).unwrap(),
            MintChainInfo::<Moment<Test>> {
                start_time: 0,
                end_time: None
            }
        );
    })
}
