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
use common::{info_from_weight, last_event, mock::*, root_origin, BOND_AMOUNT, INITIAL_BALANCE};

pub use calamari_runtime::{
	currency::KMA,
	fee::{
		FEES_PERCENTAGE_TO_AUTHOR, FEES_PERCENTAGE_TO_TREASURY, TIPS_PERCENTAGE_TO_AUTHOR,
		TIPS_PERCENTAGE_TO_TREASURY,
	},
	Authorship, Balances, CalamariVesting, Council, Democracy, EnactmentPeriod, LaunchPeriod,
	NativeTokenExistentialDeposit, Origin, Period, PolkadotXcm, Runtime, Sudo, TechnicalCommittee,
	Timestamp, Treasury, Utility, VotingPeriod,
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
	constants::time::{DAYS, HOURS},
	helpers::{get_account_id_from_seed, get_collator_keys_from_seed},
	types::{AccountId, Header},
};

use pallet_transaction_payment::ChargeTransactionPayment;

use sp_consensus_aura::AURA_ENGINE_ID;
use sp_core::{sr25519, H256};
use sp_runtime::{
	generic::DigestItem,
	traits::{BlakeTwo256, Hash, Header as HeaderT, SignedExtension},
	DispatchError, Percent,
};

fn note_preimage(proposer: &AccountId, proposal_call: &Call) -> H256 {
	let preimage = proposal_call.encode();
	let preimage_hash = BlakeTwo256::hash(&preimage[..]);
	assert_ok!(Democracy::note_preimage(
		Origin::signed(proposer.clone()),
		preimage.clone()
	));
	preimage_hash
}

fn propose_council_motion(council_motion: &Call, proposer: &AccountId) -> H256 {
	let council_motion_len: u32 = council_motion.using_encoded(|p| p.len() as u32);
	assert_ok!(Council::propose(
		Origin::signed(proposer.clone()),
		1,
		Box::new(council_motion.clone()),
		council_motion_len
	));
	let council_motion_hash = BlakeTwo256::hash_of(&council_motion);
	council_motion_hash
}

fn start_governance_assertions(proposer: &AccountId) -> H256 {
	// Setup the preimage and preimage hash
	let preimage_hash = note_preimage(
		&proposer,
		&Call::System(frame_system::Call::remark { remark: vec![0] }),
	);

	// Setup the Council and Technical Committee
	assert_ok!(Council::set_members(
		root_origin(),
		vec![proposer.clone()],
		None,
		0
	));
	assert_ok!(TechnicalCommittee::set_members(
		root_origin(),
		vec![proposer.clone()],
		None,
		0
	));

	// Setup and propose the Council motion for external_propose_default routine
	// No voting required because there's only 1 seat.
	let council_motion = Call::Democracy(pallet_democracy::Call::external_propose_default {
		proposal_hash: preimage_hash,
	});
	let council_motion_hash = propose_council_motion(&council_motion, &proposer);

	assert_eq!(
		last_event(),
		calamari_runtime::Event::Council(pallet_collective::Event::Executed {
			proposal_hash: council_motion_hash,
			result: Ok(())
		})
	);

	preimage_hash
}

fn end_governance_assertions(referendum_index: u32, end_of_referendum: u32, enactment_period: u32) {
	let time_of_enactment = end_of_referendum + enactment_period;
	run_to_block(end_of_referendum - 1);
	assert_eq!(1, Democracy::referendum_count());

	// After the voting period the referendum ends and is scheduled for enactment:
	run_to_block(end_of_referendum);
	assert_eq!(
		last_event(),
		calamari_runtime::Event::Scheduler(pallet_scheduler::Event::Scheduled {
			when: time_of_enactment,
			index: referendum_index
		})
	);

	// After the enactment period the proposal is dispatched:
	run_to_block(time_of_enactment);
	assert_eq!(
		last_event(),
		calamari_runtime::Event::Scheduler(pallet_scheduler::Event::Dispatched {
			task: (time_of_enactment, referendum_index),
			id: Some(vec![100, 101, 109, 111, 99, 114, 97, 99, 0, 0, 0, 0]),
			result: Ok(())
		})
	);
}

fn assert_proposal_is_filtered(proposer: &AccountId, motion: &Call) {
	let council_motion_hash = propose_council_motion(&motion, &proposer);

	assert_eq!(
		last_event(),
		calamari_runtime::Event::Council(pallet_collective::Event::Executed {
			proposal_hash: council_motion_hash,
			result: Err(DispatchError::Module {
				index: 0,
				error: 5,
				message: None
			})
		})
	);
}

#[test]
fn fast_track_available() {
	assert!(<calamari_runtime::Runtime as pallet_democracy::Config>::InstantAllowed::get());
}

#[test]
fn sanity_check_governance_periods() {
	assert_eq!(LaunchPeriod::get(), 7 * DAYS);
	assert_eq!(VotingPeriod::get(), 7 * DAYS);
	assert_eq!(EnactmentPeriod::get(), 1 * DAYS);
}

#[test]
fn slow_governance_works() {
	let alice = get_account_id_from_seed::<sr25519::Public>("Alice");

	ExtBuilder::default().build().execute_with(|| {
		let _preimage_hash = start_governance_assertions(&alice);

		let start_of_referendum = LaunchPeriod::get();
		let referendum_index = 0;

		run_to_block(start_of_referendum - 1);
		assert_eq!(0, Democracy::referendum_count());

		// 7 days in the external proposal queue before the referendum starts.
		run_to_block(start_of_referendum);
		assert_eq!(
			last_event(),
			calamari_runtime::Event::Democracy(pallet_democracy::Event::Started {
				ref_index: referendum_index,
				threshold: pallet_democracy::VoteThreshold::SuperMajorityAgainst
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

		end_governance_assertions(
			referendum_index,
			start_of_referendum + VotingPeriod::get(),
			EnactmentPeriod::get(),
		);
	});
}

#[test]
fn fast_track_governance_works() {
	let alice = get_account_id_from_seed::<sr25519::Public>("Alice");

	ExtBuilder::default().build().execute_with(|| {
		let preimage_hash = start_governance_assertions(&alice);

		let voting_period = 5;
		let enactment_period = 5;
		let referendum_index = 0;

		// Setup and propose the Technical Committee motion for the fast_track routine
		// No voting required because there's only 1 seat.
		// Voting and delay periods of 5 blocks so this should be enacted on block 11
		let tech_committee_motion = Call::Democracy(pallet_democracy::Call::fast_track {
			proposal_hash: preimage_hash,
			voting_period: voting_period,
			delay: enactment_period,
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
			referendum_index,
			pallet_democracy::AccountVote::Standard {
				vote: pallet_democracy::Vote {
					aye: true,
					conviction: pallet_democracy::Conviction::None
				},
				balance: 10 * KMA
			}
		));

		// No launch period because of the fast track.
		end_governance_assertions(
			referendum_index,
			System::block_number() + voting_period,
			enactment_period,
		);
	});
}

#[test]
fn governance_filters_work() {
	assert!(<calamari_runtime::Runtime as pallet_democracy::Config>::InstantAllowed::get());

	let alice = get_account_id_from_seed::<sr25519::Public>("Alice");

	ExtBuilder::default().build().execute_with(|| {
		// Setup the preimage and preimage hash
		let preimage_hash = note_preimage(
			&alice,
			&Call::System(frame_system::Call::remark { remark: vec![0] }),
		);

		// Setup the Council
		assert_ok!(Council::set_members(
			root_origin(),
			vec![alice.clone()],
			None,
			0
		));

		// Public proposals should be filtered out.
		assert_proposal_is_filtered(
			&alice,
			&Call::Democracy(pallet_democracy::Call::propose {
				proposal_hash: preimage_hash,
				value: 100 * KMA,
			}),
		);

		// External proposals other than external_proposal_default should be filtered out.
		assert_proposal_is_filtered(
			&alice,
			&Call::Democracy(pallet_democracy::Call::external_propose {
				proposal_hash: preimage_hash,
			}),
		);

		// External proposals other than external_proposal_default should be filtered out.
		assert_proposal_is_filtered(
			&alice,
			&Call::Democracy(pallet_democracy::Call::external_propose_majority {
				proposal_hash: preimage_hash,
			}),
		);
	});
}

#[test]
fn balances_operations_should_work() {
	let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
	let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
	let charlie = get_account_id_from_seed::<sr25519::Public>("Charlie");
	let dave = get_account_id_from_seed::<sr25519::Public>("Dave");

	ExtBuilder::default()
		.with_balances(vec![
			(alice.clone(), INITIAL_BALANCE),
			(bob.clone(), INITIAL_BALANCE),
			(charlie.clone(), INITIAL_BALANCE),
			(dave.clone(), INITIAL_BALANCE),
		])
		.with_authorities(vec![(alice.clone(), get_collator_keys_from_seed("Alice"))])
		.with_collators(vec![alice.clone()], 0)
		.build()
		.execute_with(|| {
			let transfer_amount = 10 * KMA;

			// Basic transfer should work
			assert_ok!(Balances::transfer(
				Origin::signed(alice.clone()),
				sp_runtime::MultiAddress::Id(charlie.clone()),
				transfer_amount,
			));
			assert_eq!(
				Balances::free_balance(alice.clone()),
				INITIAL_BALANCE - transfer_amount
			);
			assert_eq!(
				Balances::free_balance(charlie.clone()),
				INITIAL_BALANCE + transfer_amount
			);

			// Force transfer some tokens from one account to another with Root
			assert_ok!(Balances::force_transfer(
				root_origin(),
				sp_runtime::MultiAddress::Id(charlie.clone()),
				sp_runtime::MultiAddress::Id(alice.clone()),
				transfer_amount,
			));
			assert_eq!(Balances::free_balance(alice.clone()), INITIAL_BALANCE);
			assert_eq!(Balances::free_balance(charlie.clone()), INITIAL_BALANCE);

			// Should not be able to trnasfer all with this call
			assert_err!(
				Balances::transfer_keep_alive(
					Origin::signed(alice.clone()),
					sp_runtime::MultiAddress::Id(charlie.clone()),
					INITIAL_BALANCE,
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
			assert_eq!(Balances::free_balance(charlie.clone()), INITIAL_BALANCE * 2);

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
fn sanity_check_fees_and_tips_splits() {
	assert_eq!(100, TIPS_PERCENTAGE_TO_AUTHOR + TIPS_PERCENTAGE_TO_TREASURY);
	assert_eq!(100, FEES_PERCENTAGE_TO_AUTHOR + FEES_PERCENTAGE_TO_TREASURY);
}

#[test]
fn reward_fees_to_block_author_and_treasury() {
	let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
	let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
	let charlie = get_account_id_from_seed::<sr25519::Public>("Charlie");
	let desired_candidates = 0;

	ExtBuilder::default()
		.with_balances(vec![
			(alice.clone(), INITIAL_BALANCE),
			(bob.clone(), INITIAL_BALANCE),
			(charlie.clone(), INITIAL_BALANCE),
		])
		.with_authorities(vec![(alice.clone(), get_collator_keys_from_seed("Alice"))])
		.with_collators(vec![alice.clone()], desired_candidates)
		.build()
		.execute_with(|| {
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

			let author_received_reward = Balances::free_balance(alice) - INITIAL_BALANCE;
			println!("The rewarded_amount is: {:?}", author_received_reward);

			let author_percent = Percent::from_percent(FEES_PERCENTAGE_TO_AUTHOR);
			let expected_fee =
				TransactionPayment::compute_actual_fee(len as u32, &info, &post_info, 0);
			assert_eq!(author_received_reward, author_percent * expected_fee);

			let treasury_percent = Percent::from_percent(FEES_PERCENTAGE_TO_TREASURY);
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
fn sanity_check_session_period() {
	assert_eq!(Period::get(), 6 * HOURS);
}

fn advance_session_assertions(session_index: &mut u32, advance_by: u32) {
	*session_index += advance_by;

	run_to_block(*session_index * Period::get() - 1);
	assert_eq!(Session::session_index(), *session_index - 1);

	run_to_block(*session_index * Period::get());
	assert_eq!(Session::session_index(), *session_index);
}

#[test]
fn session_and_collator_selection_work() {
	let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
	let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
	let alice_aura = get_collator_keys_from_seed("Alice");
	let bob_aura = get_collator_keys_from_seed("Bob");
	let desired_candidates = 1;

	ExtBuilder::default()
		.with_collators(vec![alice.clone()], desired_candidates)
		.with_balances(vec![
			(alice.clone(), INITIAL_BALANCE),
			(bob.clone(), INITIAL_BALANCE),
		])
		.build()
		.execute_with(|| {
			// Alice is in the invulnerables set, so not part of the candidates set.
			assert_eq!(CollatorSelection::candidates(), vec![]);

			// Create and bond session keys to Bob's account.
			let keys = calamari_runtime::opaque::SessionKeys {
				aura: bob_aura.clone(),
			};
			assert_ok!(Session::set_keys(Origin::signed(bob.clone()), keys, vec![]));

			assert_ok!(CollatorSelection::register_candidate(
				root_origin(),
				bob.clone()
			));
			let candidate = manta_collator_selection::CandidateInfo {
				who: bob.clone(),
				deposit: BOND_AMOUNT,
			};

			// Bob is a candidate but only Alice is queued as a collator in this session.
			assert_eq!(CollatorSelection::candidates(), vec![candidate.clone()]);
			assert_eq!(
				Session::queued_keys(),
				vec![(
					alice.clone(),
					calamari_runtime::opaque::SessionKeys {
						aura: alice_aura.clone(),
					}
				)]
			);
			assert_eq!(Session::validators(), vec![alice.clone()]);

			let mut session_index = 0;
			assert_eq!(Session::session_index(), session_index);

			advance_session_assertions(&mut session_index, 1);

			// After 1 sessions both Alice and Bob are queued for collators
			// But only Alice is actually a collator for now.
			assert_eq!(
				Session::queued_keys(),
				vec![
					(
						alice.clone(),
						calamari_runtime::opaque::SessionKeys {
							aura: alice_aura.clone(),
						}
					),
					(
						bob.clone(),
						calamari_runtime::opaque::SessionKeys {
							aura: bob_aura.clone(),
						}
					)
				]
			);
			assert_eq!(Session::validators(), vec![alice.clone()]);

			advance_session_assertions(&mut session_index, 1);

			// Bob has finally been included as a collator
			// Note that we started from block 1, so the delay period of 1 full session
			// required us to wait until session index 2.
			assert_eq!(Session::validators(), vec![alice.clone(), bob.clone()]);

			// Once Bob decides to leave he will be removed from the candidates set immediately.
			assert_ok!(CollatorSelection::leave_intent(Origin::signed(bob.clone())));
			assert_eq!(CollatorSelection::candidates(), vec![]);

			// But will not leave as a validator until after one full session.
			assert_eq!(Session::validators(), vec![alice.clone(), bob.clone()]);

			advance_session_assertions(&mut session_index, 1);

			assert_eq!(Session::validators(), vec![alice.clone(), bob.clone()]);

			advance_session_assertions(&mut session_index, 1);

			// Bob was removed as a collator.
			assert_eq!(Session::validators(), vec![alice.clone()]);
		});
}

#[test]
fn batched_registration_of_collator_candidates_works() {
	let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
	let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
	let charlie = get_account_id_from_seed::<sr25519::Public>("Charlie");
	let alice_aura = get_collator_keys_from_seed("Alice");
	let bob_aura = get_collator_keys_from_seed("Bob");
	let charlie_aura = get_collator_keys_from_seed("Charlie");
	let desired_candidates = 2;

	ExtBuilder::default()
		.with_balances(vec![
			(alice.clone(), INITIAL_BALANCE),
			(bob.clone(), INITIAL_BALANCE),
			(charlie.clone(), INITIAL_BALANCE),
		])
		.with_collators(vec![alice.clone()], desired_candidates)
		.build()
		.execute_with(|| {
			let keys = calamari_runtime::opaque::SessionKeys {
				aura: bob_aura.clone(),
			};
			assert_ok!(Session::set_keys(Origin::signed(bob.clone()), keys, vec![]));

			let keys = calamari_runtime::opaque::SessionKeys {
				aura: charlie_aura.clone(),
			};
			assert_ok!(Session::set_keys(
				Origin::signed(charlie.clone()),
				keys,
				vec![]
			));

			assert_eq!(CollatorSelection::candidates(), vec![]);

			assert_ok!(Utility::batch(
				root_origin(),
				vec![
					Call::CollatorSelection(manta_collator_selection::Call::register_candidate {
						new_candidate: bob.clone(),
					}),
					Call::CollatorSelection(manta_collator_selection::Call::register_candidate {
						new_candidate: charlie.clone(),
					}),
				],
			));

			let candidate_bob = manta_collator_selection::CandidateInfo {
				who: bob.clone(),
				deposit: BOND_AMOUNT,
			};
			let candidate_charlie = manta_collator_selection::CandidateInfo {
				who: charlie.clone(),
				deposit: BOND_AMOUNT,
			};
			assert_eq!(
				CollatorSelection::candidates(),
				vec![candidate_bob, candidate_charlie]
			);
			assert_eq!(
				Session::queued_keys(),
				vec![(
					alice.clone(),
					calamari_runtime::opaque::SessionKeys {
						aura: alice_aura.clone(),
					}
				),]
			);
			assert_eq!(Session::validators(), vec![alice.clone()]);

			let mut session_index = 1;
			run_to_block(session_index * Period::get());

			let all_collator_pairs = vec![
				(
					alice.clone(),
					calamari_runtime::opaque::SessionKeys {
						aura: alice_aura.clone(),
					},
				),
				(
					bob.clone(),
					calamari_runtime::opaque::SessionKeys {
						aura: bob_aura.clone(),
					},
				),
				(
					charlie.clone(),
					calamari_runtime::opaque::SessionKeys {
						aura: charlie_aura.clone(),
					},
				),
			];

			assert_eq!(Session::queued_keys(), all_collator_pairs);
			assert_eq!(Session::validators(), vec![alice.clone()]);

			session_index += 1;
			run_to_block(session_index * Period::get());

			assert_eq!(Session::queued_keys(), all_collator_pairs);
			assert_eq!(
				Session::validators(),
				vec![alice.clone(), bob.clone(), charlie.clone()]
			);
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
			// Timestamp expects milliseconds, so multiply by 1_000 to convert from seconds.
			let now = schedule[period].1 * 1_000 + 1;
			Timestamp::set_timestamp(now);
			assert_ok!(CalamariVesting::vest(Origin::signed(bob.clone())));
			vested += schedule[period].0 * unvested;
			assert_eq!(Balances::usable_balance(&bob), vested);
		}
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
