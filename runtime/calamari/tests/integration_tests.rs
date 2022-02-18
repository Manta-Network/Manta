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

//! Calamari Parachain Integration Tests.

mod common;
use common::*;

pub use calamari_runtime::{
	currency::KMA, Authorship, Balances, CalamariVesting, Council, Democracy,
	NativeTokenExistentialDeposit, Origin, PolkadotXcm, Runtime, TechnicalCommittee, Timestamp,
	Treasury,
};

use frame_support::{
	assert_err, assert_ok,
	codec::Encode,
	dispatch::Dispatchable,
	traits::{PalletInfo, StorageInfo, StorageInfoTrait, ValidatorSet},
	weights::constants::*,
	StorageHasher, Twox128,
};

use manta_primitives::{
	helpers::{get_account_id_from_seed, get_collator_keys_from_seed},
	AccountId, Header,
};

use pallet_transaction_payment::ChargeTransactionPayment;

use sp_consensus_aura::AURA_ENGINE_ID;
use sp_core::sr25519;
use sp_runtime::{
	generic::DigestItem,
	traits::{BlakeTwo256, Hash, Header as HeaderT, SignedExtension},
	Percent,
};

#[test]
fn fast_track_available() {
	assert!(<calamari_runtime::Runtime as pallet_democracy::Config>::InstantAllowed::get());
}

#[test]
fn democracy_external_propose_default_with_fast_track_works() {
	assert!(<calamari_runtime::Runtime as pallet_democracy::Config>::InstantAllowed::get());

	let alice = get_account_id_from_seed::<sr25519::Public>("Alice");

	ExtBuilder::default().build().execute_with(|| {
		set_parachain_inherent_data();

		// Setup the preimage and preimage hash
		let proposal_call = Call::System(frame_system::Call::remark { remark: vec![0] });
		let preimage = proposal_call.encode();
		let preimage_hash = BlakeTwo256::hash(&preimage[..]);
		assert_ok!(Democracy::note_preimage(
			Origin::signed(alice.clone()),
			preimage.clone()
		));

		// Setup the Council and Technical Committee
		assert_ok!(Council::set_members(
			root_origin(),
			vec![alice.clone()],
			None,
			0
		));
		assert_ok!(TechnicalCommittee::set_members(
			root_origin(),
			vec![alice.clone()],
			None,
			0
		));

		// Setup and propose the Council motion for external_propose_default routine
		// No voting required because there's only 1 seat.
		let council_motion = Call::Democracy(pallet_democracy::Call::external_propose_default {
			proposal_hash: preimage_hash,
		});
		let council_motion_len: u32 = council_motion.using_encoded(|p| p.len() as u32);
		assert_ok!(Council::propose(
			Origin::signed(alice.clone()),
			1,
			Box::new(council_motion.clone()),
			council_motion_len
		));
		// Make sure it was actually executed
		let council_motion_hash = BlakeTwo256::hash_of(&council_motion);
		assert_eq!(
			last_event(),
			calamari_runtime::Event::Council(pallet_collective::Event::Executed {
				proposal_hash: council_motion_hash,
				result: Ok(())
			})
		);

		// Setup and propose the Technical Committee motion for the fast_track routine
		// No voting required because there's only 1 seat.
		// Voting and delay periods of 5 blocks so this should be enacted on block 11
		let tech_committee_motion = Call::Democracy(pallet_democracy::Call::fast_track {
			proposal_hash: preimage_hash,
			voting_period: 5,
			delay: 5,
		});
		let tech_committee_motion_len: u32 =
			tech_committee_motion.using_encoded(|p| p.len() as u32);
		let tech_committee_motion_hash = BlakeTwo256::hash_of(&tech_committee_motion);
		assert_ok!(TechnicalCommittee::propose(
			Origin::signed(alice.clone()),
			1,
			Box::new(tech_committee_motion),
			tech_committee_motion_len
		));
		// Make sure the motion was actually executed
		assert_eq!(
			last_event(),
			calamari_runtime::Event::TechnicalCommittee(pallet_collective::Event::Executed {
				proposal_hash: tech_committee_motion_hash,
				result: Ok(())
			})
		);

		// Time to vote for the referendum with some amount
		assert_ok!(Democracy::vote(
			Origin::signed(alice.clone()),
			0,
			pallet_democracy::AccountVote::Standard {
				vote: pallet_democracy::Vote {
					aye: true,
					conviction: pallet_democracy::Conviction::None
				},
				balance: 10 * KMA
			}
		));

		run_to_block(6);
		// At block 6 the voting should be concluded and the original proposal scheduled
		assert_eq!(
			last_event(),
			calamari_runtime::Event::Scheduler(pallet_scheduler::Event::Scheduled {
				when: 11,
				index: 0
			})
		);

		run_to_block(11);
		// At block 11 the scheduler should have dispatched the proposal passed with referendum 0
		assert_eq!(
			last_event(),
			calamari_runtime::Event::Scheduler(pallet_scheduler::Event::Dispatched {
				task: (11, 0),
				id: Some(vec![100, 101, 109, 111, 99, 114, 97, 99, 0, 0, 0, 0]),
				result: Ok(())
			})
		);
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
			<calamari_runtime::Runtime as frame_system::Config>::PalletInfo::name::<P>(),
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
	is_pallet_prefix::<calamari_runtime::Authorship>("Authorship");
	is_pallet_prefix::<calamari_runtime::CollatorSelection>("CollatorSelection");
	is_pallet_prefix::<calamari_runtime::Session>("Session");
	is_pallet_prefix::<calamari_runtime::Aura>("Aura");
	is_pallet_prefix::<calamari_runtime::AuraExt>("AuraExt");
	is_pallet_prefix::<calamari_runtime::Treasury>("Treasury");
	is_pallet_prefix::<calamari_runtime::Scheduler>("Scheduler");
	is_pallet_prefix::<calamari_runtime::XcmpQueue>("XcmpQueue");
	is_pallet_prefix::<calamari_runtime::PolkadotXcm>("PolkadotXcm");
	is_pallet_prefix::<calamari_runtime::CumulusXcm>("CumulusXcm");
	is_pallet_prefix::<calamari_runtime::DmpQueue>("DmpQueue");
	is_pallet_prefix::<calamari_runtime::Utility>("Utility");
	is_pallet_prefix::<calamari_runtime::Multisig>("Multisig");
	is_pallet_prefix::<calamari_runtime::Sudo>("Sudo");
	is_pallet_prefix::<calamari_runtime::CalamariVesting>("CalamariVesting");

	let prefix = |pallet_name, storage_name| {
		let mut res = [0u8; 32];
		res[0..16].copy_from_slice(&Twox128::hash(pallet_name));
		res[16..32].copy_from_slice(&Twox128::hash(storage_name));
		res.to_vec()
	};
	assert_eq!(
		<calamari_runtime::Timestamp as StorageInfoTrait>::storage_info(),
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
		<calamari_runtime::Balances as StorageInfoTrait>::storage_info(),
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
				storage_name: b"Account".to_vec(),
				prefix: prefix(b"Balances", b"Account"),
				max_values: Some(300_000),
				max_size: Some(112),
			},
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"Locks".to_vec(),
				prefix: prefix(b"Balances", b"Locks"),
				max_values: Some(300_000),
				max_size: Some(1299),
			},
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"Reserves".to_vec(),
				prefix: prefix(b"Balances", b"Reserves"),
				max_values: None,
				max_size: Some(1249),
			},
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"StorageVersion".to_vec(),
				prefix: prefix(b"Balances", b"StorageVersion"),
				max_values: Some(1),
				max_size: Some(1),
			}
		]
	);
	assert_eq!(
		<calamari_runtime::Sudo as StorageInfoTrait>::storage_info(),
		vec![StorageInfo {
			pallet_name: b"Sudo".to_vec(),
			storage_name: b"Key".to_vec(),
			prefix: prefix(b"Sudo", b"Key"),
			max_values: Some(1),
			max_size: Some(32),
		}]
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
fn verify_pallet_indices() {
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
	is_pallet_index::<calamari_runtime::Authorship>(20);
	is_pallet_index::<calamari_runtime::CollatorSelection>(21);
	is_pallet_index::<calamari_runtime::Session>(22);
	is_pallet_index::<calamari_runtime::Aura>(23);
	is_pallet_index::<calamari_runtime::AuraExt>(24);
	is_pallet_index::<calamari_runtime::Treasury>(26);
	is_pallet_index::<calamari_runtime::Scheduler>(29);
	is_pallet_index::<calamari_runtime::XcmpQueue>(30);
	is_pallet_index::<calamari_runtime::PolkadotXcm>(31);
	is_pallet_index::<calamari_runtime::CumulusXcm>(32);
	is_pallet_index::<calamari_runtime::DmpQueue>(33);
	is_pallet_index::<calamari_runtime::Utility>(40);
	is_pallet_index::<calamari_runtime::Multisig>(41);
	is_pallet_index::<calamari_runtime::Sudo>(42);
	is_pallet_index::<calamari_runtime::CalamariVesting>(50);
}

#[test]
fn balances_operations_should_work() {
	let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
	let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
	let charlie = get_account_id_from_seed::<sr25519::Public>("Charlie");
	let dave = get_account_id_from_seed::<sr25519::Public>("Dave");
	let initial_balance = 1_000 * KMA;

	ExtBuilder::default()
		.with_balances(vec![
			(alice.clone(), initial_balance),
			(bob.clone(), initial_balance),
			(charlie.clone(), initial_balance),
			(dave.clone(), initial_balance),
		])
		.with_authorities(vec![(alice.clone(), get_collator_keys_from_seed("Alice"))])
		.with_collators(vec![alice.clone()])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			let transfer = 10 * KMA;

			// Basic transfer should work
			assert_ok!(Balances::transfer(
				Origin::signed(alice.clone()),
				sp_runtime::MultiAddress::Id(charlie.clone()),
				10 * KMA,
			));
			assert_eq!(
				Balances::free_balance(alice.clone()),
				initial_balance - transfer
			);
			assert_eq!(
				Balances::free_balance(charlie.clone()),
				initial_balance + transfer
			);

			// Force transfer some tokens from one account to another with Root
			assert_ok!(Balances::force_transfer(
				root_origin(),
				sp_runtime::MultiAddress::Id(charlie.clone()),
				sp_runtime::MultiAddress::Id(alice.clone()),
				10 * KMA,
			));
			assert_eq!(Balances::free_balance(alice.clone()), initial_balance);
			assert_eq!(Balances::free_balance(charlie.clone()), initial_balance);

			// Should not be able to trnasfer all with this call
			assert_err!(
				Balances::transfer_keep_alive(
					Origin::signed(alice.clone()),
					sp_runtime::MultiAddress::Id(charlie.clone()),
					initial_balance,
				),
				pallet_balances::Error::<Runtime>::KeepAlive
			);

			// Transfer all down to zero
			assert_ok!(Balances::transfer_all(
				Origin::signed(bob.clone()),
				sp_runtime::MultiAddress::Id(charlie.clone()),
				false
			));
			assert_eq!(Balances::free_balance(bob.clone()), 0);
			assert_eq!(Balances::free_balance(charlie.clone()), initial_balance * 2);

			// Transfer all but keep alive with ED
			assert_ok!(Balances::transfer_all(
				Origin::signed(dave.clone()),
				sp_runtime::MultiAddress::Id(alice.clone()),
				true
			));
			assert_eq!(
				Balances::free_balance(dave.clone()),
				NativeTokenExistentialDeposit::get()
			);

			// Even though keep alive is set to false alice cannot fall below the ED
			// because it has an outstanding consumer reference, from being a collator.
			assert_ok!(Balances::transfer_all(
				Origin::signed(alice.clone()),
				sp_runtime::MultiAddress::Id(charlie.clone()),
				false
			));
			assert_eq!(
				Balances::free_balance(alice.clone()),
				NativeTokenExistentialDeposit::get()
			);
		});
}

fn seal_header(mut header: Header, author: AccountId) -> Header {
	{
		let digest = header.digest_mut();
		digest
			.logs
			.push(DigestItem::PreRuntime(AURA_ENGINE_ID, author.encode()));
		digest
			.logs
			.push(DigestItem::Seal(AURA_ENGINE_ID, author.encode()));
	}

	header
}

#[test]
fn reward_fees_to_block_author_and_treasury() {
	let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
	let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
	let charlie = get_account_id_from_seed::<sr25519::Public>("Charlie");
	let initial_balance = 1_000_000_000_000 * KMA;

	ExtBuilder::default()
		.with_balances(vec![
			(alice.clone(), initial_balance),
			(bob.clone(), initial_balance),
			(charlie.clone(), initial_balance),
		])
		.with_authorities(vec![(alice.clone(), get_collator_keys_from_seed("Alice"))])
		.with_collators(vec![alice.clone()])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();

			let author = alice.clone();
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
			calamari_runtime::System::initialize(&1, &Default::default(), header.digest());
			assert_eq!(Authorship::author().unwrap(), author);

			let call = Call::Balances(pallet_balances::Call::transfer {
				dest: sp_runtime::MultiAddress::Id(charlie),
				value: 10 * KMA,
			});

			let len = 10;
			let info = info_from_weight(100);
			let maybe_pre = ChargeTransactionPayment::<Runtime>::from(0)
				.pre_dispatch(&bob, &call, &info, len)
				.unwrap();

			let res = call.clone().dispatch(Origin::signed(bob));

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

			let author_received_reward = Balances::free_balance(alice) - initial_balance;
			println!("The rewarded_amount is: {:?}", author_received_reward);

			let author_percent = Percent::from_percent(60);
			let expected_fee =
				TransactionPayment::compute_actual_fee(len as u32, &info, &post_info, 0);
			assert_eq!(author_received_reward, author_percent * expected_fee);

			// Treasury gets 40% of fee
			let treasury_percent = Percent::from_percent(40);
			assert_eq!(
				Balances::free_balance(Treasury::account_id()),
				treasury_percent * expected_fee
			);
		});
}

#[test]
fn root_can_change_default_xcm_vers() {
	ExtBuilder::default().build().execute_with(|| {
		// Root sets the defaultXcm
		assert_ok!(PolkadotXcm::force_default_xcm_version(
			root_origin(),
			Some(2)
		));
	})
}

#[test]
fn session_and_collator_selection_work() {
	let initial_balance = 1_000 * KMA;
	let bob = get_account_id_from_seed::<sr25519::Public>("Bob");

	ExtBuilder::default()
		.with_balances(vec![(bob.clone(), initial_balance)])
		.build()
		.execute_with(|| {
			let keys = calamari_runtime::opaque::SessionKeys {
				aura: get_collator_keys_from_seed("Bob"),
			};
			assert_ok!(Session::set_keys(Origin::signed(bob.clone()), keys, vec![]));
			// Alice is invulnerable by default so not part of the candidates set.
			assert_eq!(CollatorSelection::candidates(), vec![]);
			assert_ok!(CollatorSelection::register_candidate(
				root_origin(),
				bob.clone()
			));
			let candidate = manta_collator_selection::CandidateInfo {
				who: bob.clone(),
				deposit: 1_000 * KMA,
			};
			// Bob is a candidate but only Alice is queued as a collator in this session.
			assert_eq!(CollatorSelection::candidates(), vec![candidate.clone()]);
			assert_eq!(Session::queued_keys().len(), 1);
			assert_eq!(Session::validators().len(), 1);

			// Session length is 6 hours
			assert_eq!(Session::session_index(), 0);
			// Advance session
			run_to_block(1800);
			// Session index should be increased
			assert_eq!(Session::session_index(), 1);

			// After 1 sessions both Alice and Bob are queued for collators
			// But only Alice is actually a collator for now.
			assert_eq!(Session::queued_keys().len(), 2);
			assert_eq!(Session::validators().len(), 1);

			run_to_block(3599);
			// Session index should not be advanced until the end of a session
			assert_eq!(Session::session_index(), 1);
			// Advance session
			run_to_block(3600);
			assert_eq!(Session::session_index(), 2);

			// Bob has finally been included as a collator
			assert_eq!(Session::validators().len(), 2);

			// Once Bob decides to leave he will be removed from the candidates set immediately.
			assert_ok!(CollatorSelection::leave_intent(Origin::signed(bob)));
			assert_eq!(CollatorSelection::candidates(), vec![]);
			// But will not leave as a validator until after one full session.
			assert_eq!(Session::validators().len(), 2);

			// Advance session.
			run_to_block(5400);
			assert_eq!(Session::validators().len(), 2);

			// Advance session.
			run_to_block(7200);
			// Bob was removed as a collator.
			assert_eq!(Session::validators().len(), 1);
		});
}

#[test]
fn sanity_check_weight_per_time_constants_are_as_expected() {
	// These values comes from Substrate, we want to make sure that if it
	// ever changes we don't accidently break Polkadot
	assert_eq!(WEIGHT_PER_SECOND, 1_000_000_000_000);
	assert_eq!(WEIGHT_PER_MILLIS, WEIGHT_PER_SECOND / 1000);
	assert_eq!(WEIGHT_PER_MICROS, WEIGHT_PER_MILLIS / 1000);
	assert_eq!(WEIGHT_PER_NANOS, WEIGHT_PER_MICROS / 1000);
}

#[test]
fn calamari_vesting_works() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
		let bob = get_account_id_from_seed::<sr25519::Public>("Bob");

		let unvested = 100 * KMA;
		assert_ok!(CalamariVesting::vested_transfer(
			Origin::signed(alice),
			sp_runtime::MultiAddress::Id(bob.clone()),
			unvested
		));

		assert_eq!(Balances::free_balance(&bob), 100 * KMA);
		assert_eq!(Balances::usable_balance(&bob), 0);

		let schedule = calamari_vesting::Pallet::<Runtime>::vesting_schedule();
		let mut vested = 0;

		for period in 0..schedule.len() {
			let now = schedule[period].1 * 1000 + 1;
			Timestamp::set_timestamp(now);
			let _res = CalamariVesting::vest(Origin::signed(bob.clone()));
			vested += schedule[period].0 * unvested;
			assert_eq!(Balances::usable_balance(&bob), vested);
		}
	});
}
