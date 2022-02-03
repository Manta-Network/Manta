mod xcm_mock;

use codec::Encode;
use frame_support::assert_ok;
use manta_primitives::AssetLocation;
use xcm::{latest::prelude::*, VersionedMultiLocation};
use xcm_mock::{parachain::PALLET_BALANCES_INDEX, *};
use xcm_simulator::TestExt;

use crate::xcm_mock::parachain::AssetManager;

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

// TO BE FIXED: This test is known to fail.
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
fn xcmp() {
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

	ParaB::execute_with(|| {
		use parachain::{Event, System};
		assert!(System::events()
			.iter()
			.any(|r| matches!(r.event, Event::System(frame_system::Event::Remarked { .. }))));
	});
}

#[test]
fn reserve_transfer_relaychain_to_parachain_a() {
	MockNet::reset();

	let relay_asset_id: parachain::AssetId = 0;
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));

	let asset_metadata = parachain::AssetRegistarMetadata {
		name: b"Kusama".to_vec(),
		symbol: b"KSM".to_vec(),
		decimals: 12,
		min_balance: 1u128,
		evm_address: None,
		is_frozen: false,
		is_sufficient: true,
	};

	// Register relay chain asset in parachain A
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
			0u128));
	});

	let withdraw_amount = 123;

	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(ALICE),
			Box::new(X1(Parachain(1)).into().into()),
			Box::new(
				X1(AccountId32 {
					network: Any,
					id: ALICE.into()
				})
				.into()
				.into()
			),
			Box::new((Here, withdraw_amount).into()),
			0,
		));
		assert_eq!(
			relay_chain::Balances::free_balance(&para_account_id(1)),
			INITIAL_BALANCE + withdraw_amount
		);
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(
			pallet_assets::Pallet::<parachain::Runtime>::balance(relay_asset_id, &ALICE.into()),
			withdraw_amount
		);
	});
}

#[test]
fn reserve_transfer_relaychain_to_parachain_a_then_back() {
	MockNet::reset();

	let relay_asset_id: parachain::AssetId = 0;
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));

	let asset_metadata = parachain::AssetRegistarMetadata {
		name: b"Kusama".to_vec(),
		symbol: b"KSM".to_vec(),
		decimals: 12,
		min_balance: 1u128,
		evm_address: None,
		is_frozen: false,
		is_sufficient: true,
	};

	// Register relay chain asset in parachain A
	ParaA::execute_with(|| {
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			source_location,
			asset_metadata
		));
		// we don't charge anything
		assert_ok!(parachain::AssetManager::set_units_per_second(
			parachain::Origin::root(), 
			relay_asset_id, 
			0u128));
	});

	let amount = 123;

	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(ALICE),
			Box::new(X1(Parachain(1)).into().into()),
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
			40000
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
fn send_para_a_asset_to_para_b() {
	MockNet::reset();

	// ParaA balance location
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
		1,
		X2(
			Parachain(1),
			PalletInstance(parachain::PALLET_BALANCES_INDEX),
		),
	)));
	let a_currency_id = 0u32;
	let amount = 100u128;

	let asset_metadata = parachain::AssetRegistarMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
		evm_address: None,
		min_balance: 1,
		is_frozen: false,
		is_sufficient: false,
	};

	// Register ParaA native asset in ParaB
	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			parachain::Origin::root(),
			source_location.clone(),
			asset_metadata.clone()
		));
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(), 
			a_currency_id, 
			0u128));
		assert_eq!(
			Some(a_currency_id),
			AssetManager::location_asset_id(source_location.clone())
		);
	});

	// Register ParaA native asset in ParaA
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			parachain::Origin::root(),
			// This need to be changed starting from v0.9.16
			// need to use something like MultiLocation { parents: 0, interior: here} instead
			source_location.clone(),
			asset_metadata
		));
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(), 
			a_currency_id, 
			0u128));
		assert_eq!(
			Some(a_currency_id),
			AssetManager::location_asset_id(source_location)
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
			parachain::CurrencyId::MantaCurrency(a_currency_id),
			amount,
			Box::new(VersionedMultiLocation::V1(dest)),
			800000
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
			parachain::Assets::balance(a_currency_id, &ALICE.into()),
			amount
		);
	});
}

#[test]
fn send_para_a_asset_para_b_and_then_send_back() {
	MockNet::reset();

	// para a asset location
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
		1,
		X2(Parachain(1), PalletInstance(PALLET_BALANCES_INDEX)),
	)));
	// a's currency id in para a, para b, and para c
	let a_currency_id = 0u32;
	let amount = 321u128;

	let asset_metadata = parachain::AssetRegistarMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
		evm_address: None,
		min_balance: 1,
		is_frozen: false,
		is_sufficient: false,
	};

	// register a_currency in ParaA, ParaB and ParaC
	ParaA::execute_with(|| {
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			// we need to change this on/after v0.9.16
			source_location.clone(),
			asset_metadata.clone()
		));
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(), 
			a_currency_id, 
			0u128));
		assert_eq!(
			Some(a_currency_id),
			parachain::AssetManager::location_asset_id(source_location.clone())
		);
	});

	ParaB::execute_with(||{
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			source_location.clone(),
			asset_metadata.clone()
		));
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(), 
			a_currency_id, 
			0u128));
		assert_eq!(
			Some(a_currency_id),
			parachain::AssetManager::location_asset_id(source_location.clone())
		);
	});

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

	ParaA::execute_with(||{
		assert_ok!(parachain::XTokens::transfer(
			parachain::Origin::signed(ALICE.into()),
			parachain::CurrencyId::MantaCurrency(a_currency_id),
			amount,
			Box::new(VersionedMultiLocation::V1(alice_on_b)),
			800000
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
			parachain::Assets::balance(a_currency_id, &ALICE.into()),
			amount
		);
	});

	let alice_on_a = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(1),
			AccountId32 {
				network: NetworkId::Any,
				id: ALICE.into(),
			},
		),
	};

	// Send wrapped a back to a
	ParaB::execute_with(||{
		assert_ok!(parachain::XTokens::transfer(
			parachain::Origin::signed(ALICE.into()),
			parachain::CurrencyId::MantaCurrency(a_currency_id),
			amount,
			Box::new(VersionedMultiLocation::V1(alice_on_a)),
			800000
		));
		assert_eq!(
			parachain::Assets::balance(a_currency_id, &ALICE.into()),
			0
		);
	});

	// make sure that a received the token
	ParaA::execute_with(||{
		assert_eq!(
			parachain::Balances::free_balance(&ALICE.into()),
			INITIAL_BALANCE
		)
	});

}

#[test]
fn send_para_a_asset_from_para_b_to_para_c() {
	MockNet::reset();

	// source location of para a asset
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
		1,
		X2(Parachain(1), PalletInstance(PALLET_BALANCES_INDEX)),
	)));
	let a_currency_id = 0u32;
	let amount = 888u128;

	let asset_metadata = parachain::AssetRegistarMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
		evm_address: None,
		min_balance: 1,
		is_frozen: false,
		is_sufficient: false,
	};

	// register a_currency in ParaA, ParaB and ParaC
	ParaA::execute_with(|| {
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			// we need to change this on/after v0.9.16
			source_location.clone(),
			asset_metadata.clone()
		));
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(), 
			a_currency_id, 
			0u128));
		assert_eq!(
			Some(a_currency_id),
			parachain::AssetManager::location_asset_id(source_location.clone())
		);
	});

	ParaB::execute_with(||{
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			source_location.clone(),
			asset_metadata.clone()
		));
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(), 
			a_currency_id, 
			0u128));
		assert_eq!(
			Some(a_currency_id),
			parachain::AssetManager::location_asset_id(source_location.clone())
		);
	});

	ParaC::execute_with(||{
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			source_location.clone(),
			asset_metadata.clone()
		));
		assert_ok!(AssetManager::set_units_per_second(
			parachain::Origin::root(), 
			a_currency_id, 
			0u128));
		assert_eq!(
			Some(a_currency_id),
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

	ParaA::execute_with(||{
		assert_ok!(parachain::XTokens::transfer(
			parachain::Origin::signed(ALICE.into()),
			parachain::CurrencyId::MantaCurrency(a_currency_id),
			amount,
			Box::new(VersionedMultiLocation::V1(alice_on_b.clone())),
			800000
		));
		assert_eq!(
			parachain::Balances::free_balance(&ALICE.into()),
			INITIAL_BALANCE - amount
		)
	});

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_eq!(
			parachain::Assets::balance(a_currency_id, &ALICE.into()),
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

	ParaB::execute_with(||{
		assert_ok!(parachain::XTokens::transfer(
			parachain::Origin::signed(ALICE.into()),
			parachain::CurrencyId::MantaCurrency(a_currency_id),
			amount,
			Box::new(VersionedMultiLocation::V1(alice_on_c)),
			800000
		));
		assert_eq!(
			parachain::Assets::balance(a_currency_id, &ALICE.into()),
			0
		);
	});

	// Make sure C received the token
	ParaC::execute_with(|| {
		// free execution, full amount received
		assert_eq!(
			parachain::Assets::balance(a_currency_id, &ALICE.into()),
			amount
		);
	});	
}


#[test]
fn receive_relay_asset_with_trader(){
	MockNet::reset();

	let relay_asset_id: parachain::AssetId = 0;
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
	let asset_metadata = parachain::AssetRegistarMetadata {
		name: b"Kusama".to_vec(),
		symbol: b"KSM".to_vec(),
		decimals: 12,
		min_balance: 1u128,
		evm_address: None,
		is_frozen: false,
		is_sufficient: true,
	};
	let amount = 666u128;

	ParaA::execute_with(||{
		assert_ok!(parachain::AssetManager::register_asset(
			parachain::Origin::root(),
			source_location,
			asset_metadata
		));
		// We charge 10^9 as units per second
		assert_ok!(parachain::AssetManager::set_units_per_second(
			parachain::Origin::root(), 
			relay_asset_id, 
			1000_000_000u128));
	});

	let dest: MultiLocation = AccountId32 {
		network: Any,
		id: ALICE.into()
	}.into();

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
	
	// `reserved_transfer_asset` contains the following 4 instructions
	//  1. ReserveAssetDeposited(assets.clone()),
	//  2. ClearOrigin,
	//  3. BuyExecution { fees, weight_limit: Limited(0) },
	//  4. DepositAsset { assets: Wild(All), max_assets, beneficiary },
	//  each instruction's weight is 1000, thus, the total charge is:
	//  10^9 * 1^3 * 4 / 10^12 (WEIGHT_PER_SECOND defined by frame_support) = 4
	ParaA::execute_with(||{
		assert_eq!(
			parachain::Assets::balance(relay_asset_id, &ALICE.into()),
			amount - 4u128
		);
		assert_eq!(
			parachain::Assets::balance(relay_asset_id, AssetManager::account_id()),
			4u128
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
