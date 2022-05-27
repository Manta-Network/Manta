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

//! Simulation Tests for XCM

mod xcm_mock;

use codec::Encode;
use frame_support::{assert_err, assert_ok, weights::constants::WEIGHT_PER_SECOND};
use manta_primitives::{
	assets::{AssetConfig, AssetLocation, AssetRegistrarMetadata},
	types::AssetId,
};
use xcm::{latest::prelude::*, v2::Response, VersionedMultiLocation, WrapVersion};
use xcm_executor::traits::WeightBounds;
use xcm_mock::{
	parachain::{ParachainAssetConfig, PALLET_ASSET_INDEX},
	*,
};
use xcm_simulator::TestExt;

use crate::xcm_mock::parachain::{
	AssetManager, ParaTokenPerSecond, XcmExecutorConfig as ParaXcmExecutorConfig,
};

// `reserved_transfer_asset` contains the following 4 instructions
//  1. ReserveAssetDeposited(assets.clone()),
//  2. ClearOrigin,
//  3. BuyExecution { fees, weight_limit: Limited(0) },
//  4. DepositAsset { assets: Wild(All), max_assets, beneficiary },
//  each instruction's weight is 1000, thus, the total weight is 4000
const RESERVE_TRANSFER_WEIGHT: u64 = 4000;

fn calculate_fee(units_per_seconds: u128, weight: u64) -> u128 {
	units_per_seconds * (weight as u128) / (WEIGHT_PER_SECOND as u128)
}

fn weight_of_xcm_msg_with_4_instructions() -> u64 {
	let mut msg = Xcm(vec![
		ReserveAssetDeposited(MultiAssets::from(vec![MultiAsset {
			id: Concrete(MultiLocation {
				parents: 1,
				interior: X1(Parachain(1)),
			}),
			fun: Fungible(10000000000000),
		}])),
		ClearOrigin,
		BuyExecution {
			fees: MultiAsset {
				id: Concrete(MultiLocation {
					parents: 1,
					interior: X1(Parachain(1)),
				}),
				fun: Fungible(10000000000000),
			},
			weight_limit: Limited(3999999999),
		},
		DepositAsset {
			assets: Wild(All),
			max_assets: 1,
			beneficiary: MultiLocation {
				parents: 0,
				interior: X1(AccountId32 {
					network: Any,
					id: [
						0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
						0, 0, 0, 0, 0, 0, 0,
					],
				}),
			},
		},
	]);

	<ParaXcmExecutorConfig as xcm_executor::Config>::Weigher::weight(&mut msg).unwrap()
}

// Helper function for forming buy execution message
fn buy_execution<C>(fees: impl Into<MultiAsset>) -> Instruction<C> {
	BuyExecution {
		fees: fees.into(),
		weight_limit: Unlimited,
	}
}

#[test]
fn dmp() {
	MockNet::reset();

	let remark = parachain::Call::System(
		frame_system::Call::<parachain::Runtime>::remark_with_event {
			remark: vec![1, 2, 3],
		},
	);
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::send_xcm(
			Here,
			Parachain(1),
			Xcm(vec![Transact {
				origin_type: OriginKind::SovereignAccount,
				require_weight_at_most: INITIAL_BALANCE as u64,
				call: remark.encode().into(),
			}]),
		));
	});

	ParaA::execute_with(|| {
		use parachain::{Event, System};
		assert!(System::events()
			.iter()
			.any(|r| matches!(r.event, Event::System(frame_system::Event::Remarked { .. }))));
	});
}

#[test]
fn ump() {
	MockNet::reset();

	let remark = relay_chain::Call::System(
		frame_system::Call::<relay_chain::Runtime>::remark_with_event {
			remark: vec![1, 2, 3],
		},
	);
	ParaA::execute_with(|| {
		assert_ok!(ParachainPalletXcm::send_xcm(
			Here,
			Parent,
			Xcm(vec![Transact {
				origin_type: OriginKind::SovereignAccount,
				require_weight_at_most: INITIAL_BALANCE as u64,
				call: remark.encode().into(),
			}]),
		));
	});

	Relay::execute_with(|| {
		use relay_chain::{Event, System};
		assert!(System::events()
			.iter()
			.any(|r| matches!(r.event, Event::System(frame_system::Event::Remarked { .. }))));
	});
}

#[test]
fn xcmp_transact_from_sibling_parachain_blocked_by_barrier() {
	MockNet::reset();

	let remark = parachain::Call::System(
		frame_system::Call::<parachain::Runtime>::remark_with_event {
			remark: vec![1, 2, 3],
		},
	);
	ParaA::execute_with(|| {
		assert_ok!(ParachainPalletXcm::send_xcm(
			Here,
			(Parent, Parachain(2)),
			Xcm(vec![Transact {
				origin_type: OriginKind::SovereignAccount,
				require_weight_at_most: INITIAL_BALANCE as u64,
				call: remark.encode().into(),
			}]),
		));
	});

	// The `AllowUnpaidExecutionFrom<ParentOrParentsExecutivePlurality>` barrier implementation
	// only allows Transact instructions sent by the relay chain's governance
	ParaB::execute_with(|| {
		use parachain::{Event, System};
		assert!(!System::events()
			.iter()
			.any(|r| matches!(r.event, Event::System(frame_system::Event::Remarked { .. }))));
	});
}

#[test]
fn reserve_transfer_relay_asset_from_relay_chain_to_parachain_a_then_back() {
	MockNet::reset();

	let relay_asset_id: AssetId =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::StartNonNativeAssetId::get();
	let relay_source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
	let units_per_sec = 0;
	let relay_asset_metadata = AssetRegistrarMetadata {
		name: b"Kusama".to_vec(),
		symbol: b"KSM".to_vec(),
		decimals: 12,
		min_balance: 1,
		evm_address: None,
		is_frozen: false,
		is_sufficient: true,
	};
	let weight_at_most = 40000;

	// Register relay chain asset in parachain A
	ParaA::execute_with(|| {
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			relay_source_location,
			relay_asset_metadata
		));
		// we don't charge anything
		assert_ok!(parachain::AssetManager::set_units_per_second(
			parachain::Origin::root(),
			relay_asset_id,
			units_per_sec
		));
	});

	let amount = 123;

	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(ALICE),
			Box::new(X1(Parachain(PARA_A_ID)).into().into()),
			Box::new(
				X1(AccountId32 {
					network: Any,
					id: ALICE.into()
				})
				.into()
				.into()
			),
			Box::new((Here, amount).into()),
			0,
		));
		assert_eq!(
			parachain::Balances::free_balance(&para_account_id(1)),
			INITIAL_BALANCE + amount
		);
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(
			pallet_assets::Pallet::<parachain::Runtime>::balance(relay_asset_id, &ALICE.into()),
			amount
		);
	});

	// Checking the balance of relay chain before sending token back
	let mut balance_before_sending = 0;
	Relay::execute_with(|| {
		balance_before_sending = RelayBalances::free_balance(&ALICE);
	});

	let dest = MultiLocation {
		parents: 1,
		interior: X1(AccountId32 {
			network: NetworkId::Any,
			id: ALICE.into(),
		}),
	};

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(parachain::XTokens::transfer(
			parachain::Origin::signed(ALICE.into()),
			parachain::CurrencyId::MantaCurrency(relay_asset_id),
			amount,
			Box::new(VersionedMultiLocation::V1(dest)),
			weight_at_most
		));
	});

	ParaA::execute_with(|| {
		// free execution, this will drain the parachain asset account
		assert_eq!(parachain::Assets::balance(relay_asset_id, &ALICE.into()), 0);
	});

	Relay::execute_with(|| {
		// free execution, full amount received
		assert_eq!(
			RelayBalances::free_balance(&ALICE),
			balance_before_sending + amount
		);
	});
}

#[test]
fn send_para_a_native_asset_to_para_b() {
	MockNet::reset();

	// We use an opinionated source location here:
	// Ideally, we could use `here()`, however, we always prefer to use the location from
	// `root` when possible.
	let a_source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
		1,
		X1(Parachain(PARA_A_ID)),
	)));
	let a_asset_id_on_a =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetId::get();
	let a_asset_id_on_b =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::StartNonNativeAssetId::get();

	let amount = 10000000000000u128;

	let weight = weight_of_xcm_msg_with_4_instructions();
	let units_per_sec = 0;
	let a_asset_metadata =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetMetadata::get();

	// Register ParaA native asset in ParaB
	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			parachain::Origin::root(),
			a_source_location.clone(),
			a_asset_metadata.clone()
		));
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(),
			a_asset_id_on_b,
			units_per_sec
		));
		assert_eq!(
			Some(a_asset_id_on_b),
			AssetManager::location_asset_id(a_source_location.clone())
		);
	});

	// Register ParaA native asset in ParaA
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(),
			a_asset_id_on_a,
			units_per_sec
		));
	});

	let dest = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(2),
			AccountId32 {
				network: NetworkId::Any,
				id: ALICE.into(),
			},
		),
	};

	// Transfer ParaA balance to B
	ParaA::execute_with(|| {
		assert_ok!(parachain::XTokens::transfer(
			parachain::Origin::signed(ALICE.into()),
			parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
			amount,
			Box::new(VersionedMultiLocation::V1(dest)),
			weight
		));
		assert_eq!(
			parachain::Balances::free_balance(&ALICE.into()),
			INITIAL_BALANCE - amount
		)
	});

	// Make sure B received the token
	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_eq!(
			parachain::Assets::balance(a_asset_id_on_b, &ALICE.into()),
			amount
		);
	});
}

#[test]
fn send_insufficient_asset_from_para_a_to_para_b() {
	MockNet::reset();

	let a_source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
		1,
		X1(Parachain(PARA_A_ID)),
	)));
	let a_asset_id_on_a =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetId::get();
	let a_asset_id_on_b =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::StartNonNativeAssetId::get();

	let amount = 8888u128;
	let units_per_second_at_b = 1_250_000u128;
	let weight = weight_of_xcm_msg_with_4_instructions();
	let fee_at_b = calculate_fee(units_per_second_at_b, weight);

	let mut a_asset_metadata_on_b =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetMetadata::get();
	a_asset_metadata_on_b.is_sufficient = false;

	// Register ParaA native asset in ParaA
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(),
			a_asset_id_on_a,
			0u128
		));
		assert_eq!(
			Some(a_asset_id_on_a),
			parachain::AssetManager::location_asset_id(a_source_location.clone())
		);
	});

	// Register ParaA native asset in ParaB
	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			parachain::Origin::root(),
			a_source_location.clone(),
			a_asset_metadata_on_b.clone()
		));
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(),
			a_asset_id_on_b,
			units_per_second_at_b
		));
		assert_eq!(
			Some(a_asset_id_on_b),
			parachain::AssetManager::location_asset_id(a_source_location.clone())
		);
	});

	let dest = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(PARA_B_ID),
			AccountId32 {
				network: NetworkId::Any,
				id: ALICE.into(),
			},
		),
	};

	assert!(amount >= fee_at_b);
	// Transfer ParaA balance to B
	ParaA::execute_with(|| {
		assert_ok!(parachain::XTokens::transfer(
			parachain::Origin::signed(ALICE.into()),
			parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
			amount,
			Box::new(VersionedMultiLocation::V1(dest.clone())),
			weight
		));
		assert_eq!(
			parachain::Balances::free_balance(&ALICE.into()),
			INITIAL_BALANCE - amount
		)
	});

	ParaB::execute_with(|| {
		// The total supply should not include the paid fee,
		// because the XcmFeesAccount had 0 providers with is_sufficient set to false,
		// so the mint_into() operation for the refund amount failed.
		assert_eq!(
			parachain::Assets::total_supply(a_asset_id_on_b),
			amount - fee_at_b
		);
		assert_eq!(
			parachain::Assets::balance(a_asset_id_on_b, &ALICE.into()),
			amount - fee_at_b
		);
	});

	// Setting the balance will in effect create the account
	// incrementing its providers counter to from 0 to 1
	ParaB::execute_with(|| {
		assert_ok!(pallet_balances::Pallet::<parachain::Runtime>::set_balance(
			parachain::Origin::root(),
			parachain::AssetManager::account_id(),
			1000000000000000,
			1000000000000000
		));
	});

	ParaA::execute_with(|| {
		assert_ok!(parachain::XTokens::transfer(
			parachain::Origin::signed(ALICE.into()),
			parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
			amount,
			Box::new(VersionedMultiLocation::V1(dest.clone())),
			weight
		));
		assert_eq!(
			parachain::Balances::free_balance(&ALICE.into()),
			INITIAL_BALANCE - amount * 2
		)
	});

	ParaB::execute_with(|| {
		// This time we expect the total supply to be the full amount
		// as the refund will be deposited to the XcmFeesAccount
		assert_eq!(
			parachain::Assets::total_supply(a_asset_id_on_b),
			(amount - fee_at_b) + amount
		);
		assert_eq!(
			parachain::Assets::balance(a_asset_id_on_b, &ALICE.into()),
			(amount - fee_at_b) * 2
		);
	});
}

#[test]
fn register_insufficient_with_zero_min_balance_should_fail() {
	MockNet::reset();

	let a_source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
		1,
		X1(Parachain(PARA_A_ID)),
	)));

	let mut a_asset_metadata_on_b =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetMetadata::get();
	a_asset_metadata_on_b.min_balance = 0;
	a_asset_metadata_on_b.is_sufficient = false;

	ParaB::execute_with(|| {
		assert_err!(
			AssetManager::register_asset(
				parachain::Origin::root(),
				a_source_location.clone(),
				a_asset_metadata_on_b.clone()
			),
			pallet_asset_manager::Error::<parachain::Runtime>::ErrorCreatingAsset
		);
	});
}

#[test]
fn send_para_a_custom_asset_to_para_b() {
	let asset_id: u32 =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::StartNonNativeAssetId::get();
	let amount = 321;
	let asset_metadata =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetMetadata::get();
	let units_per_sec = 0;
	let weight_at_most = 4000000000;
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
		1,
		X3(
			Parachain(PARA_A_ID),
			PalletInstance(PALLET_ASSET_INDEX),
			GeneralIndex(0),
		),
	)));

	// register a_currency in ParaA, ParaB
	ParaA::execute_with(|| {
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			source_location.clone(),
			asset_metadata.clone()
		));
		// we have to do this in order to mint asset to alice on A
		assert_ok!(parachain::Assets::force_asset_status(
			parachain::Origin::root(),
			asset_id,
			ALICE.into(),
			ALICE.into(),
			ALICE.into(),
			ALICE.into(),
			1,
			true,
			false,
		));
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(),
			asset_id,
			units_per_sec
		));
		assert_eq!(
			Some(asset_id),
			parachain::AssetManager::location_asset_id(source_location.clone())
		);
	});

	ParaB::execute_with(|| {
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			source_location.clone(),
			asset_metadata.clone()
		));
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(),
			asset_id,
			units_per_sec
		));
		assert_eq!(
			Some(asset_id),
			parachain::AssetManager::location_asset_id(source_location)
		);
	});

	let alice_on_b = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(PARA_B_ID),
			AccountId32 {
				network: NetworkId::Any,
				id: ALICE.into(),
			},
		),
	};

	ParaA::execute_with(|| {
		// Force customized asset balance for Alice
		assert_ok!(parachain::Assets::mint(
			parachain::Origin::signed(ALICE.into()),
			asset_id,
			ALICE.into(),
			INITIAL_BALANCE
		));
		assert_ok!(parachain::XTokens::transfer(
			parachain::Origin::signed(ALICE.into()),
			parachain::CurrencyId::MantaCurrency(asset_id),
			amount,
			Box::new(VersionedMultiLocation::V1(alice_on_b)),
			weight_at_most
		));
		assert_eq!(
			parachain::Assets::balance(asset_id, &ALICE.into()),
			INITIAL_BALANCE - amount
		)
	});

	// Make sure B received the token
	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_eq!(parachain::Assets::balance(asset_id, &ALICE.into()), amount);
	});
}

#[test]
fn send_para_a_native_asset_para_b_and_then_send_back() {
	MockNet::reset();

	// para a native asset location
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
		1,
		X1(Parachain(PARA_A_ID)),
	)));
	// a's currency id in para a, para b, and para c
	let a_asset_id_on_a =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetId::get();
	let a_asset_id_on_b =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::StartNonNativeAssetId::get();
	let amount = 10000000000000u128;
	let weight = weight_of_xcm_msg_with_4_instructions();
	let fee_on_b_when_send_back = calculate_fee(ParaTokenPerSecond::get().1, weight);
	assert!(fee_on_b_when_send_back < amount);
	let units_per_sec = 0;
	let asset_metadata =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetMetadata::get();

	// register a_currency in ParaA, ParaB
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(),
			a_asset_id_on_a,
			units_per_sec
		));
		assert_eq!(
			Some(a_asset_id_on_a),
			parachain::AssetManager::location_asset_id(source_location.clone())
		);
	});

	ParaB::execute_with(|| {
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			source_location.clone(),
			asset_metadata.clone()
		));
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(),
			a_asset_id_on_b,
			units_per_sec
		));
		assert_eq!(
			Some(a_asset_id_on_b),
			parachain::AssetManager::location_asset_id(source_location)
		);
	});

	let alice_on_b = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(PARA_B_ID),
			AccountId32 {
				network: NetworkId::Any,
				id: ALICE.into(),
			},
		),
	};

	ParaA::execute_with(|| {
		assert_ok!(parachain::XTokens::transfer(
			parachain::Origin::signed(ALICE.into()),
			parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
			amount,
			Box::new(VersionedMultiLocation::V1(alice_on_b)),
			weight
		));
		assert_eq!(
			parachain::Balances::free_balance(&ALICE.into()),
			INITIAL_BALANCE - amount
		)
	});

	// Make sure B received the token
	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_eq!(
			parachain::Assets::balance(a_asset_id_on_b, &ALICE.into()),
			amount
		);
	});

	let alice_on_a = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(PARA_A_ID),
			AccountId32 {
				network: NetworkId::Any,
				id: ALICE.into(),
			},
		),
	};

	// Send wrapped a back to a
	ParaB::execute_with(|| {
		assert_ok!(parachain::XTokens::transfer(
			parachain::Origin::signed(ALICE.into()),
			parachain::CurrencyId::MantaCurrency(a_asset_id_on_b),
			amount,
			Box::new(VersionedMultiLocation::V1(alice_on_a)),
			weight
		));
		assert_eq!(
			parachain::Assets::balance(a_asset_id_on_b, &ALICE.into()),
			0
		);
	});

	// make sure that a received the token
	ParaA::execute_with(|| {
		assert_eq!(
			parachain::Balances::free_balance(&ALICE.into()),
			INITIAL_BALANCE - fee_on_b_when_send_back
		)
	});
}

#[test]
fn send_para_a_native_asset_from_para_b_to_para_c() {
	MockNet::reset();

	// para a asset location
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
		1,
		X1(Parachain(PARA_A_ID)),
	)));
	let a_asset_id_on_a =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetId::get();
	let a_asset_id_on_b =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::StartNonNativeAssetId::get();
	let a_asset_id_on_c =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::StartNonNativeAssetId::get();
	let amount = 10000000000000u128;
	let weight = weight_of_xcm_msg_with_4_instructions();
	let fee_at_reserve = calculate_fee(ParaTokenPerSecond::get().1, weight);
	assert!(amount >= fee_at_reserve * 2 as u128);

	let mut a_asset_metadata_insufficient =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetMetadata::get();
	a_asset_metadata_insufficient.is_sufficient = false;

	// register a_currency in ParaA, ParaB and ParaC
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(),
			a_asset_id_on_a,
			0u128
		));
		assert_eq!(
			Some(a_asset_id_on_a),
			parachain::AssetManager::location_asset_id(source_location.clone())
		);
	});

	ParaB::execute_with(|| {
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			source_location.clone(),
			a_asset_metadata_insufficient.clone()
		));
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(),
			a_asset_id_on_b,
			0u128
		));
		assert_eq!(
			Some(a_asset_id_on_b),
			parachain::AssetManager::location_asset_id(source_location.clone())
		);
	});

	ParaC::execute_with(|| {
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			source_location.clone(),
			a_asset_metadata_insufficient.clone()
		));
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(),
			a_asset_id_on_c,
			0u128
		));
		assert_eq!(
			Some(a_asset_id_on_c),
			parachain::AssetManager::location_asset_id(source_location.clone())
		);
	});

	// A send B some token
	let alice_on_b = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(PARA_B_ID),
			AccountId32 {
				network: NetworkId::Any,
				id: ALICE.into(),
			},
		),
	};

	ParaA::execute_with(|| {
		assert_ok!(parachain::XTokens::transfer(
			parachain::Origin::signed(ALICE.into()),
			parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
			amount,
			Box::new(VersionedMultiLocation::V1(alice_on_b.clone())),
			weight
		));
		assert_eq!(
			parachain::Balances::free_balance(&ALICE.into()),
			INITIAL_BALANCE - amount
		)
	});

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_eq!(
			parachain::Assets::balance(a_asset_id_on_b, &ALICE.into()),
			amount
		);
	});

	// B send C para A asset
	let alice_on_c = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(PARA_C_ID),
			AccountId32 {
				network: NetworkId::Any,
				id: ALICE.into(),
			},
		),
	};

	ParaB::execute_with(|| {
		assert_ok!(parachain::XTokens::transfer(
			parachain::Origin::signed(ALICE.into()),
			parachain::CurrencyId::MantaCurrency(a_asset_id_on_b),
			amount,
			Box::new(VersionedMultiLocation::V1(alice_on_c)),
			weight,
		));
		assert_eq!(
			parachain::Assets::balance(a_asset_id_on_b, &ALICE.into()),
			0
		);
	});

	// Make sure C received the token
	ParaC::execute_with(|| {
		// free execution, full amount received
		assert_eq!(
			parachain::Assets::balance(a_asset_id_on_c, &ALICE.into()),
			amount - fee_at_reserve
		);
	});
}

#[test]
fn receive_relay_asset_with_trader_on_parachain() {
	MockNet::reset();

	let relay_asset_id: AssetId =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::StartNonNativeAssetId::get();
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
	let asset_metadata = AssetRegistrarMetadata {
		name: b"Kusama".to_vec(),
		symbol: b"KSM".to_vec(),
		decimals: 12,
		min_balance: 1u128,
		evm_address: None,
		is_frozen: false,
		is_sufficient: true,
	};
	let amount = 666u128;
	// We charge 10^9 as units per second on ParaA
	let units_per_second = 1_000_000_000u128;
	let fee = calculate_fee(units_per_second, RESERVE_TRANSFER_WEIGHT);
	assert!(fee > 0);
	let weight_at_most = 0;

	ParaA::execute_with(|| {
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			source_location,
			asset_metadata
		));
		assert_ok!(parachain::AssetManager::set_units_per_second(
			parachain::Origin::root(),
			relay_asset_id,
			units_per_second
		));
	});

	let dest: MultiLocation = AccountId32 {
		network: Any,
		id: ALICE.into(),
	}
	.into();

	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(ALICE),
			Box::new(X1(Parachain(1)).into().into()),
			Box::new(VersionedMultiLocation::V1(dest).clone().into()),
			Box::new((Here, amount).into()),
			weight_at_most,
		));
		assert_eq!(
			relay_chain::Balances::free_balance(&para_account_id(1)),
			INITIAL_BALANCE + amount
		);
	});

	ParaA::execute_with(|| {
		// ALICE gets amount - fee
		assert_eq!(
			parachain::Assets::balance(relay_asset_id, &ALICE.into()),
			amount - fee
		);
		// Fee sink gets fee
		assert_eq!(
			parachain::Assets::balance(relay_asset_id, AssetManager::account_id()),
			fee
		);
	});
}

#[test]
fn send_para_a_asset_to_para_b_with_trader_and_fee() {
	MockNet::reset();

	// para a balance location
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
		1,
		X1(Parachain(PARA_A_ID)),
	)));
	let a_asset_id_on_a: u32 =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetId::get();
	let a_asset_id_on_b: u32 =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::StartNonNativeAssetId::get();
	let amount = 10000000000000u128;
	let units_per_second = 1_250_000u128;
	let weight = weight_of_xcm_msg_with_4_instructions();
	let fee = calculate_fee(units_per_second, weight);

	let asset_metadata =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetMetadata::get();

	// Register ParaA native asset in ParaA
	ParaA::execute_with(|| {
		assert_eq!(
			Some(a_asset_id_on_a),
			AssetManager::location_asset_id(source_location.clone())
		);
	});

	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			parachain::Origin::root(),
			source_location.clone(),
			asset_metadata.clone()
		));
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(),
			a_asset_id_on_b,
			units_per_second
		));
		assert_eq!(
			Some(a_asset_id_on_b),
			AssetManager::location_asset_id(source_location.clone())
		);
	});

	let dest = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(PARA_B_ID),
			AccountId32 {
				network: NetworkId::Any,
				id: ALICE.into(),
			},
		),
	};

	// Transfer ParaA balance to B
	ParaA::execute_with(|| {
		assert_ok!(parachain::XTokens::transfer_with_fee(
			parachain::Origin::signed(ALICE.into()),
			parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
			amount,
			fee,
			Box::new(VersionedMultiLocation::V1(dest)),
			weight,
		));
		assert_eq!(
			parachain::Balances::free_balance(&ALICE.into()),
			INITIAL_BALANCE - amount - fee
		)
	});

	ParaB::execute_with(|| {
		assert_eq!(
			parachain::Assets::balance(a_asset_id_on_b, &ALICE.into()),
			amount
		);
	});
}

#[test]
fn send_para_a_asset_from_para_b_to_para_c_with_trader() {
	MockNet::reset();

	// para a balance location
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
		1,
		X1(Parachain(PARA_A_ID)),
	)));
	// let source_location =
	// 	<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetLocation::get();
	let a_asset_id_on_a: u32 =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetId::get();
	let a_asset_id_on_b: u32 =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::StartNonNativeAssetId::get();
	let a_asset_id_on_c: u32 =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::StartNonNativeAssetId::get();
	let mut amount = 10000000000000u128;
	let units_per_second = 1_250_000u128;
	let weight = weight_of_xcm_msg_with_4_instructions();
	let fee_at_a = calculate_fee(ParaTokenPerSecond::get().1, weight);
	let fee_at_b = calculate_fee(units_per_second, weight);
	let fee_at_c = calculate_fee(units_per_second, weight);
	let asset_metadata =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetMetadata::get();

	// register a_currency in ParaA, ParaB and ParaC

	// we don't charge any fee in A
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(),
			a_asset_id_on_a,
			0u128
		));
		assert_eq!(
			Some(a_asset_id_on_a),
			parachain::AssetManager::location_asset_id(source_location.clone())
		);
	});

	// We set units_per_seconds on ParaB to 1_250_000,
	ParaB::execute_with(|| {
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			source_location.clone(),
			asset_metadata.clone()
		));
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(),
			a_asset_id_on_b,
			units_per_second
		));
		assert_eq!(
			Some(a_asset_id_on_b),
			parachain::AssetManager::location_asset_id(source_location.clone())
		);
	});

	ParaC::execute_with(|| {
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			source_location.clone(),
			asset_metadata.clone()
		));
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(),
			a_asset_id_on_c,
			units_per_second
		));
		assert_eq!(
			Some(a_asset_id_on_c),
			parachain::AssetManager::location_asset_id(source_location.clone())
		);
	});

	// A send B some token
	let alice_on_b = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(2),
			AccountId32 {
				network: NetworkId::Any,
				id: ALICE.into(),
			},
		),
	};

	assert!(amount >= fee_at_b);
	ParaA::execute_with(|| {
		assert_ok!(parachain::XTokens::transfer(
			parachain::Origin::signed(ALICE.into()),
			parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
			amount,
			Box::new(VersionedMultiLocation::V1(alice_on_b.clone())),
			weight
		));
		assert_eq!(
			parachain::Balances::free_balance(&ALICE.into()),
			INITIAL_BALANCE - amount
		)
	});

	ParaB::execute_with(|| {
		assert_eq!(parachain::Assets::total_supply(a_asset_id_on_b), amount);
		amount = amount - fee_at_b;
		assert_eq!(
			parachain::Assets::balance(a_asset_id_on_b, &ALICE.into()),
			amount
		);
	});

	// B send C para A asset
	let alice_on_c = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(3),
			AccountId32 {
				network: NetworkId::Any,
				id: ALICE.into(),
			},
		),
	};

	assert!(amount >= fee_at_a + fee_at_c);
	ParaB::execute_with(|| {
		assert_ok!(parachain::XTokens::transfer(
			parachain::Origin::signed(ALICE.into()),
			parachain::CurrencyId::MantaCurrency(a_asset_id_on_b),
			amount,
			Box::new(VersionedMultiLocation::V1(alice_on_c)),
			weight
		));
		assert_eq!(
			parachain::Assets::balance(a_asset_id_on_b, &ALICE.into()),
			0
		);
	});

	// Make sure C received the token
	ParaC::execute_with(|| {
		amount = amount - fee_at_a - fee_at_c;
		assert_eq!(
			parachain::Assets::balance(a_asset_id_on_c, &ALICE.into()),
			amount
		);
	});
}

#[test]
fn receive_relay_asset_on_parachain_with_insufficient_fee_payment_should_fail() {
	MockNet::reset();

	let relay_asset_id: AssetId =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::StartNonNativeAssetId::get();
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
	let asset_metadata = AssetRegistrarMetadata {
		name: b"Kusama".to_vec(),
		symbol: b"KSM".to_vec(),
		decimals: 12,
		min_balance: 1u128,
		evm_address: None,
		is_frozen: false,
		is_sufficient: true,
	};
	let amount = 20u128;
	// We charge 2 x 10^10 as units per second on ParaA
	let units_per_second = 20_000_000_000u128;
	let fee = calculate_fee(units_per_second, RESERVE_TRANSFER_WEIGHT);
	assert!(fee > amount);
	let weight_at_most = 0;

	ParaA::execute_with(|| {
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			source_location,
			asset_metadata
		));
		assert_ok!(parachain::AssetManager::set_units_per_second(
			parachain::Origin::root(),
			relay_asset_id,
			units_per_second
		));
	});

	let dest: MultiLocation = AccountId32 {
		network: Any,
		id: ALICE.into(),
	}
	.into();

	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(ALICE),
			Box::new(X1(Parachain(1)).into().into()),
			Box::new(VersionedMultiLocation::V1(dest).clone().into()),
			Box::new((Here, amount).into()),
			weight_at_most,
		));
		assert_eq!(
			relay_chain::Balances::free_balance(&para_account_id(1)),
			INITIAL_BALANCE + amount
		);
	});

	ParaA::execute_with(|| {
		// ALICE gets nothing
		assert_eq!(parachain::Assets::balance(relay_asset_id, &ALICE.into()), 0);
		// Asset manager gets nothing, all balance stuck
		assert_eq!(
			parachain::Assets::balance(relay_asset_id, AssetManager::account_id()),
			0
		);
	});
}

#[test]
fn receive_relay_should_fail_without_specifying_units_per_second() {
	MockNet::reset();

	let relay_asset_id: AssetId = 0;
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
	let asset_metadata = AssetRegistrarMetadata {
		name: b"Kusama".to_vec(),
		symbol: b"KSM".to_vec(),
		decimals: 12,
		min_balance: 1u128,
		evm_address: None,
		is_frozen: false,
		is_sufficient: true,
	};
	let amount = 333u128;

	ParaA::execute_with(|| {
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			source_location,
			asset_metadata
		));
	});

	let dest: MultiLocation = AccountId32 {
		network: Any,
		id: ALICE.into(),
	}
	.into();

	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(ALICE),
			Box::new(X1(Parachain(1)).into().into()),
			Box::new(VersionedMultiLocation::V1(dest).clone().into()),
			Box::new((Here, amount).into()),
			0,
		));
		assert_eq!(
			relay_chain::Balances::free_balance(&para_account_id(1)),
			INITIAL_BALANCE + amount
		);
	});

	ParaA::execute_with(|| {
		// ALICE gets nothing
		assert_eq!(parachain::Assets::balance(relay_asset_id, &ALICE.into()), 0);
		// Asset manager gets nothing, all balance stuck
		assert_eq!(
			parachain::Assets::balance(relay_asset_id, AssetManager::account_id()),
			0
		);
	});
}

#[test]
fn send_para_a_asset_to_para_b_with_insufficient_fee() {
	MockNet::reset();

	// para a balance location
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
		1,
		X1(Parachain(PARA_A_ID)),
	)));
	let a_asset_id_on_a =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetId::get();
	let a_asset_id_on_b =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::StartNonNativeAssetId::get();
	let amount = 15u128;
	let units_per_second = 20_000_000u128;
	let weight = 800_000u64;
	let fee = calculate_fee(units_per_second, weight);
	assert!(fee > amount);

	let asset_metadata =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetMetadata::get();

	// Register ParaA native asset in ParaA
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(),
			a_asset_id_on_a,
			0u128
		));
		assert_eq!(
			Some(a_asset_id_on_a),
			AssetManager::location_asset_id(source_location.clone())
		);
	});

	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			parachain::Origin::root(),
			source_location.clone(),
			asset_metadata.clone()
		));
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(),
			a_asset_id_on_b,
			units_per_second
		));
		assert_eq!(
			Some(a_asset_id_on_b),
			AssetManager::location_asset_id(source_location.clone())
		);
	});

	let dest = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(2),
			AccountId32 {
				network: NetworkId::Any,
				id: ALICE.into(),
			},
		),
	};

	// Transfer ParaA balance to B
	ParaA::execute_with(|| {
		assert_ok!(parachain::XTokens::transfer(
			parachain::Origin::signed(ALICE.into()),
			parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
			amount,
			Box::new(VersionedMultiLocation::V1(dest)),
			weight,
		));
		assert_eq!(
			parachain::Balances::free_balance(&ALICE.into()),
			INITIAL_BALANCE - amount
		)
	});

	// Alice on B should receive nothing since the fee is insufficient
	ParaB::execute_with(|| {
		assert_eq!(
			parachain::Assets::balance(a_asset_id_on_b, &ALICE.into()),
			0
		);
	});
}

#[test]
fn send_para_a_asset_to_para_b_without_specifying_units_per_second() {
	MockNet::reset();

	// para a balance location
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
		1,
		X1(Parachain(PARA_A_ID)),
	)));
	let a_asset_id_on_a =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetId::get();
	let a_asset_id_on_b =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::StartNonNativeAssetId::get();
	let amount = 567u128;
	let weight = 800_000u64;

	let asset_metadata =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetMetadata::get();

	// Register ParaA native asset in ParaA
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(),
			a_asset_id_on_a,
			0u128
		));
	});

	// We don't specify units_per_second on B
	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			parachain::Origin::root(),
			source_location.clone(),
			asset_metadata.clone()
		));
		assert_eq!(
			Some(a_asset_id_on_b),
			AssetManager::location_asset_id(source_location.clone())
		);
	});

	let dest = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(PARA_B_ID),
			AccountId32 {
				network: NetworkId::Any,
				id: ALICE.into(),
			},
		),
	};

	// Transfer ParaA balance to B
	ParaA::execute_with(|| {
		assert_ok!(parachain::XTokens::transfer(
			parachain::Origin::signed(ALICE.into()),
			parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
			amount,
			Box::new(VersionedMultiLocation::V1(dest)),
			weight,
		));
		assert_eq!(
			parachain::Balances::free_balance(&ALICE.into()),
			INITIAL_BALANCE - amount
		)
	});

	// Alice on B should receive nothing since we didn't specify the unit per second
	ParaB::execute_with(|| {
		assert_eq!(
			parachain::Assets::balance(a_asset_id_on_b, &ALICE.into()),
			0
		);
	});
}

#[test]
fn receive_insufficient_relay_asset_on_parachain() {
	MockNet::reset();

	let new_account = [5u8; 32];
	let relay_asset_id =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::StartNonNativeAssetId::get();
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
	let asset_metadata = AssetRegistrarMetadata {
		name: b"Kusama".to_vec(),
		symbol: b"KSM".to_vec(),
		decimals: 12,
		min_balance: 1u128,
		evm_address: None,
		is_frozen: false,
		is_sufficient: false,
	};
	let amount = 123u128;
	let units_per_sec = 0u128;

	// register relay asset in parachain A
	ParaA::execute_with(|| {
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			source_location,
			asset_metadata
		));
		// we don't charge anything during test
		assert_ok!(parachain::AssetManager::set_units_per_second(
			parachain::Origin::root(),
			relay_asset_id,
			units_per_sec
		));
	});

	let dest: MultiLocation = AccountId32 {
		network: Any,
		id: new_account.into(),
	}
	.into();

	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(ALICE),
			Box::new(X1(Parachain(1)).into().into()),
			Box::new(VersionedMultiLocation::V1(dest.clone()).clone().into()),
			Box::new((Here, amount).into()),
			0,
		));
		assert_eq!(
			relay_chain::Balances::free_balance(&para_account_id(1)),
			INITIAL_BALANCE + amount
		);
	});

	// parachain should not have received assets
	ParaA::execute_with(|| {
		assert_eq!(
			parachain::Assets::balance(relay_asset_id, &new_account.into()),
			0
		);
	});

	let fresh_account_amount = 100;
	// Send native token to fresh_account
	ParaA::execute_with(|| {
		assert_ok!(parachain::Balances::transfer(
			parachain::Origin::signed(ALICE.into()),
			new_account.into(),
			fresh_account_amount
		));
	});

	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(ALICE),
			Box::new(X1(Parachain(1)).into().into()),
			Box::new(VersionedMultiLocation::V1(dest).clone().into()),
			Box::new((Here, amount).into()),
			0,
		));
		assert_eq!(
			relay_chain::Balances::free_balance(&para_account_id(1)),
			INITIAL_BALANCE + amount + amount
		);
	});

	// parachain should not have received assets
	ParaA::execute_with(|| {
		assert_eq!(
			parachain::Balances::free_balance(&new_account.into()),
			fresh_account_amount
		);
	});
}

#[test]
fn receive_sufficient_relay_asset_on_parachain() {
	MockNet::reset();

	let new_account = [5u8; 32];
	let relay_asset_id =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::StartNonNativeAssetId::get();
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
	let asset_metadata = AssetRegistrarMetadata {
		name: b"Kusama".to_vec(),
		symbol: b"KSM".to_vec(),
		decimals: 12,
		min_balance: 1u128,
		evm_address: None,
		is_frozen: false,
		is_sufficient: true,
	};
	let amount = 123u128;
	let units_per_sec = 0;

	// register relay asset in parachain A
	ParaA::execute_with(|| {
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			source_location,
			asset_metadata
		));
		// we don't charge anything during test
		assert_ok!(parachain::AssetManager::set_units_per_second(
			parachain::Origin::root(),
			relay_asset_id,
			units_per_sec
		));
	});

	let dest: MultiLocation = AccountId32 {
		network: Any,
		id: new_account.into(),
	}
	.into();

	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(ALICE),
			Box::new(X1(Parachain(1)).into().into()),
			Box::new(VersionedMultiLocation::V1(dest.clone()).clone().into()),
			Box::new((Here, amount).into()),
			0,
		));
		assert_eq!(
			relay_chain::Balances::free_balance(&para_account_id(1)),
			INITIAL_BALANCE + amount
		);
	});

	// parachain should have received assets
	ParaA::execute_with(|| {
		assert_eq!(
			parachain::Assets::balance(relay_asset_id, &new_account.into()),
			amount
		);
	});
}

/// Scenario:
/// A parachain transfers funds on the relay chain to another parachain account.
///
/// Asserts that the parachain accounts are updated as expected.
#[test]
fn withdraw_and_deposit() {
	MockNet::reset();

	let send_amount = 10;

	ParaA::execute_with(|| {
		let message = Xcm(vec![
			WithdrawAsset((Here, send_amount).into()),
			buy_execution((Here, send_amount)),
			DepositAsset {
				assets: All.into(),
				max_assets: 1,
				beneficiary: Parachain(2).into(),
			},
		]);
		// Send withdraw and deposit
		assert_ok!(ParachainPalletXcm::send_xcm(Here, Parent, message.clone()));
	});

	Relay::execute_with(|| {
		assert_eq!(
			relay_chain::Balances::free_balance(para_account_id(1)),
			INITIAL_BALANCE - send_amount
		);
		assert_eq!(
			relay_chain::Balances::free_balance(para_account_id(2)),
			send_amount
		);
	});
}

/// Scenario:
/// A parachain wants to be notified that a transfer worked correctly.
/// It sends a `QueryHolding` after the deposit to get notified on success.
///
/// Asserts that the balances are updated correctly and the expected XCM is sent.
#[test]
fn query_holding() {
	MockNet::reset();

	let send_amount = 10;
	let query_id_set = 1234;

	// Send a message which fully succeeds on the relay chain
	ParaA::execute_with(|| {
		let message = Xcm(vec![
			WithdrawAsset((Here, send_amount).into()),
			buy_execution((Here, send_amount)),
			DepositAsset {
				assets: All.into(),
				max_assets: 1,
				beneficiary: Parachain(2).into(),
			},
			QueryHolding {
				query_id: query_id_set,
				dest: Parachain(1).into(),
				assets: All.into(),
				max_response_weight: 1_000_000_000,
			},
		]);
		// Send withdraw and deposit with query holding
		assert_ok!(ParachainPalletXcm::send_xcm(Here, Parent, message.clone(),));
	});

	// Check that transfer was executed
	Relay::execute_with(|| {
		// Withdraw executed
		assert_eq!(
			relay_chain::Balances::free_balance(para_account_id(1)),
			INITIAL_BALANCE - send_amount
		);
		// Deposit executed
		assert_eq!(
			relay_chain::Balances::free_balance(para_account_id(2)),
			send_amount
		);
	});

	// Check that QueryResponse message was received
	ParaA::execute_with(|| {
		assert_eq!(
			parachain::MsgQueue::received_dmp(),
			vec![Xcm(vec![QueryResponse {
				query_id: query_id_set,
				response: Response::Assets(MultiAssets::new()),
				max_weight: 1_000_000_000,
			}])],
		);
	});
}

#[test]
fn test_versioning_on_runtime_upgrade_with_relay() {
	MockNet::reset();

	let relay_asset_id_on_a =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::StartNonNativeAssetId::get();
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
	let asset_metadata = AssetRegistrarMetadata {
		name: b"Kusama".to_vec(),
		symbol: b"KSM".to_vec(),
		decimals: 12,
		min_balance: 1u128,
		evm_address: None,
		is_frozen: false,
		is_sufficient: true,
	};

	// register relay asset in parachain A (XCM version 1)
	ParaA::execute_with(|| {
		parachain::XcmVersioner::set_version(1);
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			source_location,
			asset_metadata
		));
		// we don't charge anything during test
		assert_ok!(parachain::AssetManager::set_units_per_second(
			parachain::Origin::root(),
			relay_asset_id_on_a,
			0u128
		));
	});

	let response = Response::Version(2);

	// This is irrelevant, nothing will be done with this message,
	// but we need to pass a message as an argument to trigger the storage change
	let mock_message: Xcm<()> = Xcm(vec![QueryResponse {
		query_id: 0,
		response,
		max_weight: 0,
	}]);

	let dest: MultiLocation = AccountId32 {
		network: Any,
		id: ALICE.into(),
	}
	.into();

	Relay::execute_with(|| {
		// This sets the default version, for not known destinations
		assert_ok!(RelayChainPalletXcm::force_default_xcm_version(
			relay_chain::Origin::root(),
			Some(2)
		));

		// Wrap version, which sets VersionedStorage
		// This is necessary because the mock router does not use wrap_version, but
		// this is not necessary in prod.
		// more specifically, this will trigger `note_unknown_version` to put the
		// version to `VersionDiscoveryQueue` on relay-chain's pallet-xcm
		assert_ok!(<RelayChainPalletXcm as WrapVersion>::wrap_version(
			&Parachain(PARA_A_ID).into(),
			mock_message
		));

		// Transfer assets. Since it is an unknown destination, it will query for version
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(ALICE),
			Box::new(Parachain(PARA_A_ID).into().into()),
			Box::new(VersionedMultiLocation::V1(dest).clone().into()),
			Box::new((Here, 123).into()),
			0,
		));

		// Let's advance the relay. This should trigger the subscription message
		relay_chain::relay_roll_to(2);

		// queries should have been updated
		assert!(RelayChainPalletXcm::query(0).is_some());
	});

	let expected_supported_version: relay_chain::Event =
		pallet_xcm::Event::SupportedVersionChanged(
			MultiLocation {
				parents: 0,
				interior: X1(Parachain(PARA_A_ID)),
			},
			1,
		)
		.into();

	Relay::execute_with(|| {
		// Assert that the events vector contains the version change
		assert!(relay_chain::relay_events().contains(&expected_supported_version));
	});

	let expected_version_notified: parachain::Event = pallet_xcm::Event::VersionChangeNotified(
		MultiLocation {
			parents: 1,
			interior: Here,
		},
		2,
	)
	.into();

	// ParaA changes version to 2, and calls on_runtime_upgrade. This should notify the targets
	// of the new version change
	ParaA::execute_with(|| {
		// Set version
		parachain::XcmVersioner::set_version(2);
		// Do runtime upgrade
		parachain::on_runtime_upgrade();
		// Initialize block, to call on_initialize and notify targets
		parachain::para_roll_to(2);
		// Expect the event in the parachain
		assert!(parachain::para_events().contains(&expected_version_notified));
	});

	// This event should have been seen in the relay
	let expected_supported_version_2: relay_chain::Event =
		pallet_xcm::Event::SupportedVersionChanged(
			MultiLocation {
				parents: 0,
				interior: X1(Parachain(PARA_A_ID)),
			},
			2,
		)
		.into();

	Relay::execute_with(|| {
		// Assert that the events vector contains the new version change
		assert!(relay_chain::relay_events().contains(&expected_supported_version_2));
	});
}

#[test]
fn test_automatic_versioning_on_runtime_upgrade_with_para_b() {
	MockNet::reset();

	// para a balance location
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
		1,
		X1(Parachain(PARA_A_ID)),
	)));
	let a_asset_id_on_a =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetId::get();
	let a_asset_id_on_b =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::StartNonNativeAssetId::get();
	let asset_metadata =
		<ParachainAssetConfig as AssetConfig<parachain::Runtime>>::NativeAssetMetadata::get();
	let response = Response::Version(2);
	let amount = 10000000000000u128;
	let weight = weight_of_xcm_msg_with_4_instructions();
	let units_per_sec = 0;

	// This is irrelevant, nothing will be done with this message,
	// but we need to pass a message as an argument to trigger the storage change
	let mock_message: Xcm<()> = Xcm(vec![QueryResponse {
		query_id: 0,
		response,
		max_weight: 0,
	}]);

	ParaA::execute_with(|| {
		// advertised version
		parachain::XcmVersioner::set_version(2);
		assert_eq!(
			Some(a_asset_id_on_a),
			AssetManager::location_asset_id(source_location.clone())
		);
	});

	ParaB::execute_with(|| {
		// Let's try with v0
		parachain::XcmVersioner::set_version(0);

		assert_ok!(AssetManager::register_asset(
			parachain::Origin::root(),
			source_location,
			asset_metadata
		));
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(),
			a_asset_id_on_b,
			units_per_sec
		));
	});

	ParaA::execute_with(|| {
		// This sets the default version, for not known destinations
		assert_ok!(ParachainPalletXcm::force_default_xcm_version(
			parachain::Origin::root(),
			Some(2)
		));
		// Wrap version, which sets VersionedStorage
		assert_ok!(<ParachainPalletXcm as WrapVersion>::wrap_version(
			&MultiLocation::new(1, X1(Parachain(2))).into(),
			mock_message
		));

		parachain::para_roll_to(2);

		// queries should have been updated
		assert!(ParachainPalletXcm::query(0).is_some());
	});

	let expected_supported_version: parachain::Event = pallet_xcm::Event::SupportedVersionChanged(
		MultiLocation {
			parents: 1,
			interior: X1(Parachain(PARA_B_ID)),
		},
		0,
	)
	.into();

	ParaA::execute_with(|| {
		// Assert that the events vector contains the version change
		assert!(parachain::para_events().contains(&expected_supported_version));
	});

	// Let's ensure talking in v0 works
	let dest = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(PARA_B_ID),
			AccountId32 {
				network: NetworkId::Any,
				id: ALICE.into(),
			},
		),
	};

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(parachain::XTokens::transfer(
			parachain::Origin::signed(ALICE.into()),
			parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
			amount,
			Box::new(VersionedMultiLocation::V1(dest)),
			weight
		));
		// free execution, full amount received
		assert_eq!(
			parachain::Balances::free_balance(&ALICE.into()),
			INITIAL_BALANCE - amount
		);
	});

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_eq!(
			parachain::Assets::balance(a_asset_id_on_b, &ALICE.into()),
			amount
		);
	});

	let expected_version_notified: parachain::Event = pallet_xcm::Event::VersionChangeNotified(
		MultiLocation {
			parents: 1,
			interior: X1(Parachain(PARA_A_ID)),
		},
		2,
	)
	.into();

	// ParaB changes version to 2, and calls on_runtime_upgrade. This should notify the targets
	// of the new version change
	ParaB::execute_with(|| {
		// Set version
		parachain::XcmVersioner::set_version(2);
		// Do runtime upgrade
		parachain::on_runtime_upgrade();
		// Initialize block, to call on_initialize and notify targets
		parachain::para_roll_to(2);
		// Expect the event in the parachain
		assert!(parachain::para_events().contains(&expected_version_notified));
	});

	// This event should have been seen in para A
	let expected_supported_version_2: parachain::Event =
		pallet_xcm::Event::SupportedVersionChanged(
			MultiLocation {
				parents: 1,
				interior: X1(Parachain(PARA_B_ID)),
			},
			2,
		)
		.into();

	// Para A should have received the version change
	ParaA::execute_with(|| {
		// Assert that the events vector contains the new version change
		assert!(parachain::para_events().contains(&expected_supported_version_2));
	});
}
