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
    mock::{new_test_ext, Balances, MantaSBTPallet, RuntimeOrigin as MockOrigin, Test, Timestamp},
    AllowlistAccount, DispatchError, Error, EvmAccountAllowlist, EvmAddress, FreeReserveAccount,
    MintId, MintIdRegistry, MintStatus, ReservedIds, SbtMetadataV2, SignatureInfoOf, MANTA_MINT_ID,
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
use sp_core::{sr25519, Pair};
use sp_io::hashing::keccak_256;
use sp_runtime::AccountId32;

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

/// Initializes a test by funding accounts, reserving_sbt, set mintInfo for mintId=0.
#[inline]
fn initialize_test() {
    assert_ok!(Balances::set_balance(
        MockOrigin::root(),
        ALICE,
        1_000_000_000_000_000,
        0
    ));
    assert_ok!(MantaSBTPallet::reserve_sbt(MockOrigin::signed(ALICE), None));
}

/// Test that to_private mint is not available if mint info is not added.
#[test]
fn to_private_mint_not_available() {
    let mut rng = OsRng;
    new_test_ext().execute_with(|| {
        assert_ok!(Balances::set_balance(
            MockOrigin::root(),
            ALICE,
            1_000_000_000_000_000,
            0
        ));
        assert_ok!(MantaSBTPallet::reserve_sbt(MockOrigin::signed(ALICE), None));

        let value = 1;
        let id = field_from_id(ReservedIds::<Test>::get(ALICE).unwrap().0);
        let post = sample_to_private(id, value, &mut rng);

        assert_noop!(
            MantaSBTPallet::to_private(
                MockOrigin::signed(ALICE),
                Some(1),
                None,
                None,
                Box::new(post),
                bvec![0]
            ),
            Error::<Test>::MintNotAvailable,
        );
    });
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
            None,
            None,
            None,
            Box::new(post),
            bvec![0]
        ));
        assert_eq!(SbtMetadataV2::<Test>::get(1).unwrap().extra, Some(bvec![0]));
        assert_eq!(
            SbtMetadataV2::<Test>::get(1).unwrap().mint_id,
            MANTA_MINT_ID
        );

        assert_ok!(MantaSBTPallet::new_mint_info(
            MockOrigin::root(),
            0,
            None,
            bvec![],
            true
        ));
        let id = field_from_id(ReservedIds::<Test>::get(ALICE).unwrap().0);
        let post = sample_to_private(id, value, &mut rng);
        assert_ok!(MantaSBTPallet::to_private(
            MockOrigin::signed(ALICE),
            Some(1),
            None,
            None,
            Box::new(post),
            bvec![0]
        ));
        assert_eq!(SbtMetadataV2::<Test>::get(2).unwrap().extra, Some(bvec![0]));
        assert_eq!(SbtMetadataV2::<Test>::get(2).unwrap().mint_id, 1);
    });
}

#[test]
fn to_private_relay_signature_works() {
    let mut rng = OsRng;
    new_test_ext().execute_with(|| {
        let account_pair = sr25519::Pair::from_string(
            "endorse doctor arch helmet master dragon wild favorite property mercy vault maze",
            None,
        )
        .unwrap();
        let account_pair_2 = sr25519::Pair::from_string(
            "bottom drive obey lake curtain smoke basket hold race lonely fit walk",
            None,
        )
        .unwrap();
        let public_account: AccountId32 = account_pair.public().into();

        assert_ok!(Balances::set_balance(
            MockOrigin::root(),
            ALICE,
            1_000_000_000_000_000,
            0
        ));
        assert_ok!(MantaSBTPallet::reserve_sbt(
            MockOrigin::signed(ALICE),
            Some(public_account.clone())
        ));

        let value = 1;
        let id = field_from_id(ReservedIds::<Test>::get(public_account.clone()).unwrap().0);
        let post = sample_to_private(id, value, &mut rng);

        let msg_hash = keccak_256(&MantaSBTPallet::eip712_signable_message(&post.proof, 0));
        let wrap_msg: Vec<u8> = MantaSBTPallet::wrap_msg_with_bytes(msg_hash);

        // with bytes wrapped
        let signature = account_pair.sign(wrap_msg.as_ref());
        // without bytes wrapped
        let signature1 = account_pair.sign(msg_hash.as_ref());
        // wrong account
        let signature_2 = account_pair_2.sign(msg_hash.as_ref());

        let signature_info = SignatureInfoOf::<Test> {
            sig: signature.into(),
            pub_key: account_pair.public().into(),
        };
        let bad_signature_info1 = SignatureInfoOf::<Test> {
            sig: signature1.into(),
            pub_key: account_pair.public().into(),
        };
        // incorrect signature
        let bad_signature_info2 = SignatureInfoOf::<Test> {
            sig: signature_2.into(),
            pub_key: account_pair.public().into(),
        };

        // bad signature fails
        assert_noop!(
            MantaSBTPallet::to_private(
                MockOrigin::signed(ALICE),
                None,
                None,
                Some(bad_signature_info1),
                Box::new(post.clone()),
                bvec![0]
            ),
            Error::<Test>::BadSignature
        );
        // bad signature fails
        assert_noop!(
            MantaSBTPallet::to_private(
                MockOrigin::signed(ALICE),
                None,
                None,
                Some(bad_signature_info2),
                Box::new(post.clone()),
                bvec![0]
            ),
            Error::<Test>::BadSignature
        );

        // have alice relay `account_pair`
        assert_ok!(MantaSBTPallet::to_private(
            MockOrigin::signed(ALICE),
            None,
            None,
            Some(signature_info),
            Box::new(post),
            bvec![0]
        ));
        assert_eq!(SbtMetadataV2::<Test>::get(1).unwrap().extra, Some(bvec![0]));
        assert_eq!(
            SbtMetadataV2::<Test>::get(1).unwrap().mint_id,
            MANTA_MINT_ID
        );

        let id = field_from_id(ReservedIds::<Test>::get(public_account).unwrap().0);
        let post = sample_to_private(id, value, &mut rng);

        // chain id set to 1.
        let chain_id: u64 = 1;
        let msg_hash = keccak_256(&MantaSBTPallet::eip712_signable_message(
            &post.proof,
            chain_id,
        ));
        let wrap_msg: Vec<u8> = MantaSBTPallet::wrap_msg_with_bytes(msg_hash);

        // with bytes wrapped
        let signature = account_pair.sign(wrap_msg.as_ref());
        let signature_info = SignatureInfoOf::<Test> {
            sig: signature.into(),
            pub_key: account_pair.public().into(),
        };
        // bad signature cause of mismatch of chainId
        assert_noop!(
            MantaSBTPallet::to_private(
                MockOrigin::signed(ALICE),
                None,
                None,
                Some(signature_info.clone()),
                Box::new(post.clone()),
                bvec![0]
            ),
            Error::<Test>::BadSignature
        );
        let mint_id: u32 = 1;
        assert_noop!(
            MantaSBTPallet::to_private(
                MockOrigin::signed(ALICE),
                Some(mint_id),
                Some(chain_id),
                Some(signature_info.clone()),
                Box::new(post.clone()),
                bvec![0]
            ),
            Error::<Test>::MintNotAvailable
        );
        // new mintInfo with mintId = 1
        assert_ok!(MantaSBTPallet::new_mint_info(
            MockOrigin::root(),
            0,
            None,
            bvec![],
            true
        ));
        assert_ok!(MantaSBTPallet::to_private(
            MockOrigin::signed(ALICE),
            Some(mint_id),
            Some(chain_id),
            Some(signature_info),
            Box::new(post),
            bvec![0]
        ));
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
                None,
                None,
                None,
                Box::new(post),
                bvec![0]
            ));
        }
    });
}

/// Cannot call reserve_sbt if account already has AssetIds reserved
#[test]
fn overwrite_asset_id_fails() {
    new_test_ext().execute_with(|| {
        initialize_test();
        // second reserve_id fails
        assert_noop!(
            MantaSBTPallet::reserve_sbt(MockOrigin::signed(ALICE), None),
            Error::<Test>::AssetIdsAlreadyReserved
        );
    });
}

#[test]
fn allowlist_account_reserves_works() {
    new_test_ext().execute_with(|| {
        initialize_test();
        // reserving AssetId was not free
        assert_eq!(Balances::free_balance(&ALICE), 999999999999000);

        assert_ok!(MantaSBTPallet::change_free_reserve_account(
            MockOrigin::root(),
            Some(ALICE)
        ));
        assert_ok!(MantaSBTPallet::reserve_sbt(
            MockOrigin::signed(ALICE),
            Some(BOB)
        ));
        assert!(ReservedIds::<Test>::get(BOB).is_some());
        // balance remains the same as allowlist account is free
        assert_eq!(Balances::free_balance(&ALICE), 999999999999000);

        // allowlist account cannot overwrite reserved ids
        assert_noop!(
            MantaSBTPallet::reserve_sbt(MockOrigin::signed(ALICE), None),
            Error::<Test>::AssetIdsAlreadyReserved
        );
        assert_noop!(
            MantaSBTPallet::reserve_sbt(MockOrigin::signed(ALICE), Some(ALICE)),
            Error::<Test>::AssetIdsAlreadyReserved
        );
        assert_noop!(
            MantaSBTPallet::reserve_sbt(MockOrigin::signed(ALICE), Some(BOB)),
            Error::<Test>::AssetIdsAlreadyReserved
        );
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
                    None,
                    None,
                    None,
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
            MantaSBTPallet::to_private(
                MockOrigin::signed(ALICE),
                None,
                None,
                None,
                Box::new(post),
                bvec![0]
            ),
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
            None,
            None,
            None,
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
            MantaSBTPallet::to_private(
                MockOrigin::signed(ALICE),
                None,
                None,
                None,
                Box::new(post),
                bvec![]
            ),
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
            None,
            None,
            None,
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
            MantaSBTPallet::to_private(
                MockOrigin::signed(ALICE),
                None,
                None,
                None,
                Box::new(post),
                bvec![]
            ),
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
                None,
                None,
                None,
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
                None,
                None,
                None,
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
fn change_free_reserve_account_works() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            MantaSBTPallet::change_free_reserve_account(MockOrigin::signed(ALICE), Some(ALICE)),
            DispatchError::BadOrigin
        );
        assert_eq!(FreeReserveAccount::<Test>::get(), None);

        assert_ok!(MantaSBTPallet::change_free_reserve_account(
            MockOrigin::root(),
            Some(ALICE)
        ));
        assert_eq!(FreeReserveAccount::<Test>::get().unwrap(), ALICE);
        assert_ok!(MantaSBTPallet::change_free_reserve_account(
            MockOrigin::root(),
            None
        ));
        assert_eq!(FreeReserveAccount::<Test>::get(), None);
    })
}

#[test]
fn allowlist_account_works() {
    new_test_ext().execute_with(|| {
        let bab_id: MintId = 1;
        assert_noop!(
            MantaSBTPallet::allowlist_evm_account(
                MockOrigin::signed(ALICE),
                bab_id,
                EvmAddress::default(),
            ),
            Error::<Test>::MintNotAvailable,
        );
        assert_ok!(MantaSBTPallet::new_mint_info(
            MockOrigin::root(),
            5,
            None,
            bvec![],
            false
        ));
        Timestamp::set_timestamp(2);
        assert_noop!(
            MantaSBTPallet::allowlist_evm_account(
                MockOrigin::signed(ALICE),
                bab_id,
                EvmAddress::default()
            ),
            Error::<Test>::MintNotAvailable,
        );

        Timestamp::set_timestamp(10);
        assert_noop!(
            MantaSBTPallet::allowlist_evm_account(
                MockOrigin::signed(ALICE),
                bab_id,
                EvmAddress::default()
            ),
            Error::<Test>::NotAllowlistAccount,
        );
        assert_eq!(
            EvmAccountAllowlist::<Test>::get(1, EvmAddress::default()),
            None
        );

        assert_ok!(MantaSBTPallet::change_allowlist_account(
            MockOrigin::root(),
            Some(ALICE)
        ));
        assert_ok!(MantaSBTPallet::allowlist_evm_account(
            MockOrigin::signed(ALICE),
            bab_id,
            EvmAddress::default(),
        ));
        assert_eq!(
            EvmAccountAllowlist::<Test>::get(bab_id, EvmAddress::default()).unwrap(),
            MintStatus::Available(1)
        );

        assert_noop!(
            MantaSBTPallet::allowlist_evm_account(
                MockOrigin::signed(ALICE),
                bab_id,
                EvmAddress::default(),
            ),
            Error::<Test>::AlreadyInAllowlist,
        );

        assert_noop!(
            MantaSBTPallet::allowlist_evm_account(
                MockOrigin::signed(BOB),
                bab_id,
                EvmAddress::default()
            ),
            Error::<Test>::NotAllowlistAccount,
        );

        assert_noop!(
            MantaSBTPallet::remove_allowlist_evm_account(
                MockOrigin::signed(ALICE),
                bab_id,
                EvmAddress::default(),
            ),
            DispatchError::BadOrigin
        );

        assert_ok!(MantaSBTPallet::remove_allowlist_evm_account(
            MockOrigin::root(),
            bab_id,
            EvmAddress::default(),
        ),);
        assert!(!EvmAccountAllowlist::<Test>::contains_key(
            bab_id,
            EvmAddress::default()
        ));
    })
}

#[test]
fn mint_sbt_eth_works() {
    let mut rng = OsRng;
    new_test_ext().execute_with(|| {
        let bab_id = 1;
        assert_ok!(MantaSBTPallet::change_allowlist_account(
            MockOrigin::root(),
            Some(ALICE)
        ));
        let alice_eth_account = MantaSBTPallet::eth_address(&alice_eth());
        Timestamp::set_timestamp(10);
        assert_ok!(MantaSBTPallet::new_mint_info(
            MockOrigin::root(),
            0,
            None,
            bvec![],
            false
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
                bab_id,
                Some(0),
                Some(0),
                Some(bvec![0])
            ),
            Error::<Test>::NotAllowlisted
        );
        // allowlist account
        assert_ok!(MantaSBTPallet::allowlist_evm_account(
            MockOrigin::signed(ALICE),
            bab_id,
            alice_eth_account
        ));

        // wrong signature fails
        assert_noop!(
            MantaSBTPallet::mint_sbt_eth(
                MockOrigin::signed(ALICE),
                post.clone(),
                1,
                MantaSBTPallet::eth_sign(&alice_eth(), &[0; 128], 1),
                bab_id,
                Some(0),
                Some(0),
                Some(bvec![0])
            ),
            Error::<Test>::NotAllowlisted
        );
        assert_noop!(
            MantaSBTPallet::mint_sbt_eth(
                MockOrigin::signed(ALICE),
                post.clone(),
                1,
                MantaSBTPallet::eth_sign(&bob_eth(), &[0; 128], 1),
                bab_id,
                Some(0),
                Some(0),
                Some(bvec![0])
            ),
            Error::<Test>::NotAllowlisted
        );
        assert_noop!(
            MantaSBTPallet::mint_sbt_eth(
                MockOrigin::signed(ALICE),
                post.clone(),
                1,
                MantaSBTPallet::eth_sign(&bob_eth(), &post.proof, 2),
                bab_id,
                Some(0),
                Some(0),
                Some(bvec![0])
            ),
            Error::<Test>::NotAllowlisted
        );
        assert_noop!(
            MantaSBTPallet::mint_sbt_eth(
                MockOrigin::signed(ALICE),
                post.clone(),
                2,
                MantaSBTPallet::eth_sign(&bob_eth(), &post.proof, 1),
                bab_id,
                Some(0),
                Some(0),
                Some(bvec![0])
            ),
            Error::<Test>::NotAllowlisted
        );

        assert_ok!(MantaSBTPallet::mint_sbt_eth(
            MockOrigin::signed(ALICE),
            post.clone(),
            1,
            MantaSBTPallet::eth_sign(&alice_eth(), &post.proof, 1),
            bab_id,
            Some(0),
            Some(0),
            Some(bvec![0])
        ));
        let sbt_metadata = SbtMetadataV2::<Test>::get(1).unwrap();
        assert_eq!(sbt_metadata.collection_id, Some(0));
        assert_eq!(sbt_metadata.item_id, Some(0));
        assert_eq!(sbt_metadata.extra, Some(bvec![0]));
        assert_eq!(sbt_metadata.mint_id, bab_id);

        // Account is already minted
        assert_eq!(
            EvmAccountAllowlist::<Test>::get(bab_id, alice_eth_account).unwrap(),
            MintStatus::AlreadyMinted
        );
        assert_noop!(
            MantaSBTPallet::mint_sbt_eth(
                MockOrigin::signed(ALICE),
                post.clone(),
                1,
                MantaSBTPallet::eth_sign(&alice_eth(), &post.proof, 1),
                bab_id,
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
        let bab_id = 1;
        assert_ok!(MantaSBTPallet::change_allowlist_account(
            MockOrigin::root(),
            Some(ALICE)
        ));
        assert_ok!(MantaSBTPallet::new_mint_info(
            MockOrigin::root(),
            0,
            None,
            bvec![],
            true
        ));
        Timestamp::set_timestamp(1);
        let alice_eth_account = MantaSBTPallet::eth_address(&alice_eth());
        assert_ok!(MantaSBTPallet::allowlist_evm_account(
            MockOrigin::signed(ALICE),
            bab_id,
            alice_eth_account
        ));
        assert_ok!(MantaSBTPallet::update_mint_info(
            MockOrigin::root(),
            bab_id,
            10,
            Some(20),
            bvec![],
            false
        ));

        let value = 1;
        let storage_id = match EvmAccountAllowlist::<Test>::get(bab_id, alice_eth_account).unwrap()
        {
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
                bab_id,
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
                bab_id,
                Some(0),
                Some(0),
                Some(bvec![0])
            ),
            Error::<Test>::MintNotAvailable
        );

        // with None as end time works
        assert_ok!(MantaSBTPallet::update_mint_info(
            MockOrigin::root(),
            bab_id,
            10,
            None,
            bvec![],
            false
        ));
        assert_ok!(MantaSBTPallet::mint_sbt_eth(
            MockOrigin::signed(ALICE),
            post.clone(),
            1,
            MantaSBTPallet::eth_sign(&alice_eth(), &post.proof, 1),
            bab_id,
            Some(0),
            Some(0),
            Some(bvec![0])
        ));
    })
}

#[test]
fn new_mint_info_works() {
    new_test_ext().execute_with(|| {
        let bab_id = 1;
        assert_noop!(
            MantaSBTPallet::new_mint_info(MockOrigin::signed(ALICE), 0, None, bvec![], true),
            DispatchError::BadOrigin
        );
        assert_noop!(
            MantaSBTPallet::new_mint_info(MockOrigin::root(), 10, Some(5), bvec![], true),
            Error::<Test>::InvalidTimeRange
        );

        assert_ok!(MantaSBTPallet::new_mint_info(
            MockOrigin::root(),
            0,
            None,
            bvec![],
            true,
        ));
        let registered_mint = MintIdRegistry::<Test>::get(bab_id).unwrap();
        assert_eq!(registered_mint.start_time, 0);
        assert_eq!(registered_mint.end_time, None);
        assert_eq!(registered_mint.mint_name, vec![]);
    })
}

#[test]
fn update_mint_info_works() {
    new_test_ext().execute_with(|| {
        let bab_id = 1;
        assert_noop!(
            MantaSBTPallet::update_mint_info(
                MockOrigin::root(),
                bab_id,
                10,
                Some(20),
                bvec![],
                true
            ),
            Error::<Test>::InvalidMintId
        );
        assert_noop!(
            MantaSBTPallet::update_mint_info(
                MockOrigin::signed(ALICE),
                bab_id,
                10,
                Some(20),
                bvec![],
                true
            ),
            DispatchError::BadOrigin
        );

        assert_ok!(MantaSBTPallet::new_mint_info(
            MockOrigin::root(),
            0,
            None,
            bvec![],
            true
        ));
        assert_noop!(
            MantaSBTPallet::update_mint_info(
                MockOrigin::root(),
                bab_id,
                10,
                Some(5),
                bvec![],
                true
            ),
            Error::<Test>::InvalidTimeRange
        );

        assert_ok!(MantaSBTPallet::update_mint_info(
            MockOrigin::root(),
            bab_id,
            10,
            Some(20),
            bvec![],
            true,
        ));
    })
}
