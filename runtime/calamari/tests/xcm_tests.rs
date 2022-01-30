mod xcm_mock;

use codec::Encode;
use frame_support::assert_ok;
use xcm::latest::prelude::*;
use xcm::VersionedMultiLocation;
use xcm_mock::*;
use xcm_simulator::TestExt;
use manta_primitives::AssetLocation;

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
	let source_location = AssetLocation::Xcm(MultiLocation::parent());

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
			parachain::Balances::free_balance(&para_account_id(1)),
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
	let source_location = AssetLocation::Xcm(MultiLocation::parent());

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
			parachain::Balances::free_balance(&para_account_id(1)),
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
	 		withdraw_amount,
	 		Box::new(VersionedMultiLocation::V1(dest)),
	 		40000
	 	));
	});

	ParaA::execute_with(|| {
	 	// free execution, full amount received
	 	assert_eq!(parachain::Assets::balance(relay_asset_id, &ALICE.into()), 0);
	});

	Relay::execute_with(|| {
	 	// free execution,x	 full amount received
	 	assert!(RelayBalances::free_balance(&ALICE) > balance_before_sending);
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
