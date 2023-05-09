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

//! Calamari Parachain Integration Tests.

#![cfg(test)]
#![allow(clippy::identity_op)] // keep e.g. 1 * DAYS for legibility

use super::{mock::*, *};

pub use calamari_runtime::{
    assets_config::{CalamariAssetConfig, CalamariConcreteFungibleLedger},
    currency::KMA,
    fee::{FEES_PERCENTAGE_TO_AUTHOR, FEES_PERCENTAGE_TO_TREASURY},
    xcm_config::{XcmExecutorConfig, XcmFeesAccount},
    AssetManager, Assets, Authorship, Balances, CalamariVesting, Council, DefaultBlocksPerRound,
    Democracy, EnactmentPeriod, LaunchPeriod, LeaveDelayRounds, NativeTokenExistentialDeposit,
    ParachainStaking, Period, PolkadotXcm, Preimage, Runtime, RuntimeEvent as Event, RuntimeOrigin,
    TechnicalCommittee, Timestamp, TransactionPause, Treasury, Utility, VotingPeriod,
};

use calamari_runtime::opaque::SessionKeys;
use frame_support::{
    assert_err, assert_noop, assert_ok,
    codec::Encode,
    dispatch::Dispatchable,
    traits::{
        tokens::ExistenceRequirement, Get, PalletInfo, StorageInfo, StorageInfoTrait, StorePreimage,
    },
    StorageHasher, Twox128,
};
use runtime_common::test_helpers::{
    self_reserve_xcm_message_receiver_side, self_reserve_xcm_message_sender_side,
    to_reserve_xcm_message_receiver_side, to_reserve_xcm_message_sender_side,
    ADVERTISED_DEST_WEIGHT,
};

use manta_primitives::{
    assets::{
        AssetConfig, AssetLocation, AssetRegistryMetadata, AssetStorageMetadata, FungibleLedger,
        FungibleLedgerError,
    },
    constants::time::{DAYS, HOURS},
    types::{AccountId, Balance, CalamariAssetId, Header},
};
use session_key_primitives::util::unchecked_account_id;
use xcm::{
    opaque::latest::{
        Junction::{PalletInstance, Parachain},
        Junctions::X2,
        MultiLocation,
    },
    VersionedMultiLocation,
};
use xcm_executor::traits::WeightBounds;

use nimbus_primitives::NIMBUS_ENGINE_ID;
use pallet_transaction_payment::ChargeTransactionPayment;
use sp_core::{sr25519, H256};
use sp_runtime::{
    generic::DigestItem,
    traits::{BadOrigin, BlakeTwo256, Hash, Header as HeaderT, SignedExtension},
    DispatchError, ModuleError, Percent,
};

fn note_preimage(proposer: &AccountId, proposal_call: &RuntimeCall) -> H256 {
    let preimage = proposal_call.encode();
    let preimage_hash = BlakeTwo256::hash(&preimage[..]);
    assert_ok!(Preimage::note_preimage(
        RuntimeOrigin::signed(proposer.clone()),
        preimage
    ));
    preimage_hash
}

fn propose_council_motion(council_motion: &RuntimeCall, proposer: &AccountId) -> H256 {
    let council_motion_len: u32 = council_motion.using_encoded(|p| p.len() as u32);
    assert_ok!(Council::propose(
        RuntimeOrigin::signed(proposer.clone()),
        1,
        Box::new(council_motion.clone()),
        council_motion_len
    ));

    BlakeTwo256::hash_of(&council_motion)
}

fn start_supermajority_against_governance_assertions(proposer: &AccountId) -> H256 {
    // Setup the preimage and preimage hash
    let runtime_call = RuntimeCall::System(frame_system::Call::remark { remark: vec![0] });
    let preimage_hash = note_preimage(proposer, &runtime_call);
    let proposal = Preimage::bound(runtime_call).unwrap();

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
    let council_motion =
        RuntimeCall::Democracy(pallet_democracy::Call::external_propose_default { proposal });
    let council_motion_hash = propose_council_motion(&council_motion, proposer);

    assert_eq!(
        last_event(),
        calamari_runtime::RuntimeEvent::Council(pallet_collective::Event::Executed {
            proposal_hash: council_motion_hash,
            result: Ok(())
        })
    );

    preimage_hash
}

fn start_majority_carries_governance_assertions(proposer: &AccountId) -> H256 {
    let democracy_proposal = RuntimeCall::System(frame_system::Call::remark { remark: vec![0] });

    // Setup the preimage and preimage hash
    let preimage_hash = note_preimage(
        proposer,
        &RuntimeCall::System(frame_system::Call::remark { remark: vec![0] }),
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

    let bounded_democracy_proposal = Preimage::bound(democracy_proposal).unwrap();
    // Setup and propose the Council motion for external_propose_default routine
    // No voting required because there's only 1 seat.
    let council_motion =
        RuntimeCall::Democracy(pallet_democracy::Call::external_propose_majority {
            proposal: bounded_democracy_proposal,
        });
    let council_motion_hash = propose_council_motion(&council_motion, proposer);

    assert_eq!(
        last_event(),
        calamari_runtime::RuntimeEvent::Council(pallet_collective::Event::Executed {
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
        calamari_runtime::RuntimeEvent::Scheduler(pallet_scheduler::Event::Scheduled {
            when: time_of_enactment,
            index: referendum_index
        })
    );

    // After the enactment period the proposal is dispatched:
    run_to_block(time_of_enactment);
    assert_eq!(
        last_event(),
        calamari_runtime::RuntimeEvent::Scheduler(pallet_scheduler::Event::Dispatched {
            task: (time_of_enactment, referendum_index),
            id: Some([
                100, 101, 109, 111, 99, 114, 97, 99, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0
            ]),
            result: Ok(())
        })
    );
}

fn assert_proposal_is_filtered(proposer: &AccountId, motion: &RuntimeCall) {
    let council_motion_hash = propose_council_motion(motion, proposer);

    assert_eq!(
        last_event(),
        calamari_runtime::RuntimeEvent::Council(pallet_collective::Event::Executed {
            proposal_hash: council_motion_hash,
            result: Err(DispatchError::Module(ModuleError {
                index: 0,
                error: [5, 0, 0, 0],
                message: None
            }))
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
fn ensure_block_per_round_and_leave_delays_equal_7days() {
    // NOTE: If you change one, change the other as well
    type LeaveCandidatesDelay =
        <calamari_runtime::Runtime as pallet_parachain_staking::Config>::LeaveCandidatesDelay;
    type LeaveDelegatorsDelay =
        <calamari_runtime::Runtime as pallet_parachain_staking::Config>::LeaveDelegatorsDelay;
    type CandidateBondLessDelay =
        <calamari_runtime::Runtime as pallet_parachain_staking::Config>::CandidateBondLessDelay;
    type DelegationBondLessDelay =
        <calamari_runtime::Runtime as pallet_parachain_staking::Config>::DelegationBondLessDelay;
    assert_eq!(
        7 * DAYS,
        DefaultBlocksPerRound::get() * LeaveDelayRounds::get()
    );
    assert_eq!(
        7 * DAYS,
        DefaultBlocksPerRound::get() * LeaveCandidatesDelay::get()
    );
    assert_eq!(
        7 * DAYS,
        DefaultBlocksPerRound::get() * LeaveDelegatorsDelay::get()
    );
    assert_eq!(
        7 * DAYS,
        DefaultBlocksPerRound::get() * CandidateBondLessDelay::get()
    );
    assert_eq!(
        7 * DAYS,
        DefaultBlocksPerRound::get() * DelegationBondLessDelay::get()
    );
}

#[test]
fn slow_majority_carries_governance_works() {
    ExtBuilder::default().build().execute_with(|| {
        let _preimage_hash = start_majority_carries_governance_assertions(&ALICE);

        let start_of_referendum = LaunchPeriod::get();
        let referendum_index = 0;

        run_to_block(start_of_referendum - 1);
        assert_eq!(0, Democracy::referendum_count());

        // 7 days in the external proposal queue before the referendum starts.
        run_to_block(start_of_referendum);
        assert_eq!(
            last_event(),
            calamari_runtime::RuntimeEvent::Democracy(pallet_democracy::Event::Started {
                ref_index: referendum_index,
                threshold: pallet_democracy::VoteThreshold::SimpleMajority
            })
        );
        // Time to vote for the referendum with some amount
        assert_ok!(Democracy::vote(
            RuntimeOrigin::signed(ALICE.clone()),
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
fn slow_supermajority_against_governance_works() {
    ExtBuilder::default().build().execute_with(|| {
        let _preimage_hash = start_supermajority_against_governance_assertions(&ALICE);

        let start_of_referendum = LaunchPeriod::get();
        let referendum_index = 0;

        run_to_block(start_of_referendum - 1);
        assert_eq!(0, Democracy::referendum_count());

        // 7 days in the external proposal queue before the referendum starts.
        run_to_block(start_of_referendum);
        assert_eq!(
            last_event(),
            calamari_runtime::RuntimeEvent::Democracy(pallet_democracy::Event::Started {
                ref_index: referendum_index,
                threshold: pallet_democracy::VoteThreshold::SuperMajorityAgainst
            })
        );
        // Time to vote for the referendum with some amount
        assert_ok!(Democracy::vote(
            RuntimeOrigin::signed(ALICE.clone()),
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
    ExtBuilder::default().build().execute_with(|| {
        let preimage_hash = start_supermajority_against_governance_assertions(&ALICE);

        let voting_period = 5;
        let enactment_period = 5;
        let referendum_index = 0;

        // Setup and propose the Technical Committee motion for the fast_track routine
        // No voting required because there's only 1 seat.
        // Voting and delay periods of 5 blocks so this should be enacted on block 11
        let tech_committee_motion = RuntimeCall::Democracy(pallet_democracy::Call::fast_track {
            proposal_hash: preimage_hash,
            voting_period,
            delay: enactment_period,
        });
        let tech_committee_motion_len: u32 =
            tech_committee_motion.using_encoded(|p| p.len() as u32);
        let tech_committee_motion_hash = BlakeTwo256::hash_of(&tech_committee_motion);
        assert_ok!(TechnicalCommittee::propose(
            RuntimeOrigin::signed(ALICE.clone()),
            1,
            Box::new(tech_committee_motion),
            tech_committee_motion_len
        ));
        // Make sure the motion was actually executed
        assert_eq!(
            last_event(),
            calamari_runtime::RuntimeEvent::TechnicalCommittee(
                pallet_collective::Event::Executed {
                    proposal_hash: tech_committee_motion_hash,
                    result: Ok(())
                }
            )
        );

        // Time to vote for the referendum with some amount
        assert_ok!(Democracy::vote(
            RuntimeOrigin::signed(ALICE.clone()),
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

    ExtBuilder::default().build().execute_with(|| {
        // Setup the preimage and preimage hash
        let runtime_call = RuntimeCall::System(frame_system::Call::remark { remark: vec![0] });
        let proposal = Preimage::bound(runtime_call).unwrap();

        // Setup the Council
        assert_ok!(Council::set_members(
            root_origin(),
            vec![ALICE.clone()],
            None,
            0
        ));

        // Public proposals should be filtered out.
        assert_proposal_is_filtered(
            &ALICE,
            &RuntimeCall::Democracy(pallet_democracy::Call::propose {
                proposal,
                value: 100 * KMA,
            }),
        );
    });
}

#[test]
fn balances_operations_should_work() {
    ExtBuilder::default()
        .with_balances(vec![
            (ALICE.clone(), INITIAL_BALANCE),
            (BOB.clone(), INITIAL_BALANCE),
            (CHARLIE.clone(), INITIAL_BALANCE),
            (DAVE.clone(), INITIAL_BALANCE),
        ])
        .build()
        .execute_with(|| {
            let transfer_amount = 10 * KMA;

            // Basic transfer should work
            assert_ok!(Balances::transfer(
                RuntimeOrigin::signed(ALICE.clone()),
                sp_runtime::MultiAddress::Id(CHARLIE.clone()),
                transfer_amount,
            ));
            assert_eq!(
                Balances::free_balance(ALICE.clone()),
                INITIAL_BALANCE - transfer_amount
            );
            assert_eq!(
                Balances::free_balance(CHARLIE.clone()),
                INITIAL_BALANCE + transfer_amount
            );

            // Force transfer some tokens from one account to another with Root
            assert_ok!(Balances::force_transfer(
                root_origin(),
                sp_runtime::MultiAddress::Id(CHARLIE.clone()),
                sp_runtime::MultiAddress::Id(ALICE.clone()),
                transfer_amount,
            ));
            assert_eq!(Balances::free_balance(ALICE.clone()), INITIAL_BALANCE);
            assert_eq!(Balances::free_balance(CHARLIE.clone()), INITIAL_BALANCE);

            // Should not be able to transfer all with this call
            assert_err!(
                Balances::transfer_keep_alive(
                    RuntimeOrigin::signed(ALICE.clone()),
                    sp_runtime::MultiAddress::Id(CHARLIE.clone()),
                    INITIAL_BALANCE,
                ),
                pallet_balances::Error::<Runtime>::KeepAlive
            );

            // Transfer all down to zero
            assert_ok!(Balances::transfer_all(
                RuntimeOrigin::signed(BOB.clone()),
                sp_runtime::MultiAddress::Id(CHARLIE.clone()),
                false
            ));
            assert_eq!(Balances::free_balance(BOB.clone()), 0);
            assert_eq!(Balances::free_balance(CHARLIE.clone()), INITIAL_BALANCE * 2);

            // Transfer all but keep alive with ED
            assert_ok!(Balances::transfer_all(
                RuntimeOrigin::signed(DAVE.clone()),
                sp_runtime::MultiAddress::Id(ALICE.clone()),
                true
            ));
            assert_eq!(
                Balances::free_balance(DAVE.clone()),
                NativeTokenExistentialDeposit::get()
            );

            // Even though keep alive is set to false alice cannot fall below the ED
            // because it has an outstanding consumer reference, from being a collator.
            assert_ok!(Balances::transfer_all(
                RuntimeOrigin::signed(ALICE.clone()),
                sp_runtime::MultiAddress::Id(CHARLIE.clone()),
                false
            ));
            assert_eq!(
                Balances::free_balance(ALICE.clone()),
                NativeTokenExistentialDeposit::get()
            );
        });
}

fn seal_header(mut header: Header, author: AccountId) -> Header {
    {
        let digest = header.digest_mut();
        digest
            .logs
            .push(DigestItem::PreRuntime(NIMBUS_ENGINE_ID, author.encode()));
        digest
            .logs
            .push(DigestItem::Seal(NIMBUS_ENGINE_ID, author.encode()));
    }

    header
}

#[test]
fn reward_fees_to_block_author_and_treasury() {
    ExtBuilder::default()
        .with_balances(vec![
            (ALICE.clone(), INITIAL_BALANCE),
            (BOB.clone(), INITIAL_BALANCE),
            (CHARLIE.clone(), INITIAL_BALANCE),
        ])
        .with_authorities(vec![(
            ALICE.clone(),
            SessionKeys::from_seed_unchecked("Alice"),
        )])
        .build()
        .execute_with(|| {
            let author = ALICE.clone();
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

            let call = RuntimeCall::Balances(pallet_balances::Call::transfer {
                dest: sp_runtime::MultiAddress::Id(CHARLIE.clone()),
                value: 10 * KMA,
            });

            let len = 10;
            let info = info_from_weight(Weight::from_ref_time(100));
            let maybe_pre = ChargeTransactionPayment::<Runtime>::from(0)
                .pre_dispatch(&BOB, &call, &info, len)
                .unwrap();

            let res = call.dispatch(RuntimeOrigin::signed(BOB.clone()));

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

            let author_received_reward = Balances::free_balance(ALICE.clone()) - INITIAL_BALANCE;
            println!("The rewarded_amount is: {author_received_reward:?}");

            // Fees split: 40% burned, 40% to treasury, 10% to author.
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
fn sanity_check_round_duration() {
    assert_eq!(DefaultBlocksPerRound::get(), 6 * HOURS);
}

#[test]
fn collator_can_join_with_min_bond() {
    ExtBuilder::default()
        .with_collators(vec![(ALICE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR)])
        .with_balances(vec![
            (ALICE.clone(), INITIAL_BALANCE),
            (BOB.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
        ])
        .build()
        .execute_with(|| {
            // Create and bond session keys to Bob's account.
            assert_ok!(Session::set_keys(
                RuntimeOrigin::signed(BOB.clone()),
                BOB_SESSION_KEYS.clone(),
                vec![]
            ));
            assert!(<Session as frame_support::traits::ValidatorRegistration<AccountId>>::is_registered(&BOB));

            assert_ok!(ParachainStaking::join_candidates(
                RuntimeOrigin::signed(BOB.clone()),
                <calamari_runtime::Runtime as pallet_parachain_staking::Config>::MinCandidateStk::get(),
                3u32
            ));

            // BOB is now a candidate
            assert!(ParachainStaking::candidate_pool().contains(&pallet_parachain_staking::Bond{ owner: BOB.clone(), amount: MIN_BOND_TO_BE_CONSIDERED_COLLATOR }));

            // After one round
            run_to_block(<calamari_runtime::Runtime as pallet_parachain_staking::Config>::DefaultBlocksPerRound::get() + 1);

            // BOB becomes part of the selected candidates set
            assert!(ParachainStaking::selected_candidates().contains(&BOB));
        });
}

#[test]
fn collator_cant_join_below_standard_bond() {
    ExtBuilder::default()
        .with_balances(vec![
            (ALICE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
            (BOB.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
            (CHARLIE.clone(), EARLY_COLLATOR_MINIMUM_STAKE + 100),
        ])
        .with_collators(vec![(ALICE.clone(), 50)])
        .build()
        .execute_with(|| {
            assert_noop!(
                ParachainStaking::join_candidates(
                    RuntimeOrigin::signed(BOB.clone()),
                    MIN_BOND_TO_BE_CONSIDERED_COLLATOR - 1,
                    6u32
                ),
                pallet_parachain_staking::Error::<Runtime>::CandidateBondBelowMin
            );
        });
}

#[test]
fn collator_with_large_stake_but_too_low_self_bond_not_selected_for_block_production() {
    ExtBuilder::default()
        .with_balances(vec![
            (ALICE.clone(), EARLY_COLLATOR_MINIMUM_STAKE + 100),
            (BOB.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
            (CHARLIE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
            (DAVE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
            (EVE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
            (FERDIE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
            (USER.clone(), 400_000_000 * KMA),
        ])
        .with_invulnerables(vec![])
        .with_authorities(vec![
            (ALICE.clone(), ALICE_SESSION_KEYS.clone()),
            (BOB.clone(), BOB_SESSION_KEYS.clone()),
            (CHARLIE.clone(), CHARLIE_SESSION_KEYS.clone()),
            (DAVE.clone(), DAVE_SESSION_KEYS.clone()),
            (EVE.clone(), EVE_SESSION_KEYS.clone()),
            (FERDIE.clone(), FERDIE_SESSION_KEYS.clone()),
        ])
        .build()
        .execute_with(|| {
            initialize_collators_through_whitelist(vec![
                ALICE.clone(),
                BOB.clone(),
                CHARLIE.clone(),
                DAVE.clone(),
                EVE.clone(),
                FERDIE.clone(),
            ]);
            // Increase self-bond for everyone but ALICE
            for collator in vec![
                BOB.clone(),
                CHARLIE.clone(),
                DAVE.clone(),
                EVE.clone(),
                FERDIE.clone(),
            ] {
                assert_ok!(ParachainStaking::candidate_bond_more(
                    RuntimeOrigin::signed(collator.clone()),
                    MIN_BOND_TO_BE_CONSIDERED_COLLATOR - EARLY_COLLATOR_MINIMUM_STAKE
                ));
            }

            // Delegate a large amount of tokens to EVE and ALICE
            for collator in vec![EVE.clone(), ALICE.clone()] {
                assert_ok!(ParachainStaking::delegate(
                    RuntimeOrigin::signed(USER.clone()),
                    collator,
                    100_000_000 * KMA,
                    50,
                    50
                ));
            }

            // Ensure ALICE is not selected despite having a large total stake through delegation
            // NOTE: Must use 6 or more collators because 5 is the minimum on calamari
            assert!(!ParachainStaking::compute_top_candidates().contains(&ALICE));
            assert!(ParachainStaking::compute_top_candidates().contains(&BOB));
            assert!(ParachainStaking::compute_top_candidates().contains(&CHARLIE));
            assert!(ParachainStaking::compute_top_candidates().contains(&DAVE));
            assert!(ParachainStaking::compute_top_candidates().contains(&EVE));
            assert!(ParachainStaking::compute_top_candidates().contains(&FERDIE));
        });
}

#[test]
fn collator_can_leave_if_below_standard_bond() {
    ExtBuilder::default()
        .with_balances(vec![
            (ALICE.clone(), EARLY_COLLATOR_MINIMUM_STAKE + 100),
            (BOB.clone(), EARLY_COLLATOR_MINIMUM_STAKE + 100),
            (CHARLIE.clone(), EARLY_COLLATOR_MINIMUM_STAKE + 100),
            (DAVE.clone(), EARLY_COLLATOR_MINIMUM_STAKE + 100),
            (EVE.clone(), EARLY_COLLATOR_MINIMUM_STAKE + 100),
            (FERDIE.clone(), EARLY_COLLATOR_MINIMUM_STAKE + 100),
        ])
        .with_invulnerables(vec![])
        .with_authorities(vec![
            (ALICE.clone(), ALICE_SESSION_KEYS.clone()),
            (BOB.clone(), BOB_SESSION_KEYS.clone()),
            (CHARLIE.clone(), CHARLIE_SESSION_KEYS.clone()),
            (DAVE.clone(), DAVE_SESSION_KEYS.clone()),
            (EVE.clone(), EVE_SESSION_KEYS.clone()),
            (FERDIE.clone(), FERDIE_SESSION_KEYS.clone()),
        ])
        .build()
        .execute_with(|| {
            initialize_collators_through_whitelist(vec![
                ALICE.clone(),
                BOB.clone(),
                CHARLIE.clone(),
                DAVE.clone(),
                EVE.clone(),
                FERDIE.clone(),
            ]);
            // Attempt to leave as whitelist collator
            assert_ok!(ParachainStaking::schedule_leave_candidates(
                RuntimeOrigin::signed(FERDIE.clone()),
                6
            ));
        });
}

#[test]
fn collator_with_400k_not_selected_for_block_production() {
    ExtBuilder::default()
        .with_balances(vec![
            (ALICE.clone(), EARLY_COLLATOR_MINIMUM_STAKE + 100),
            (BOB.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
            (CHARLIE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
            (DAVE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
            (EVE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
            (FERDIE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
        ])
        .with_invulnerables(vec![])
        .with_authorities(vec![
            (ALICE.clone(), ALICE_SESSION_KEYS.clone()),
            (BOB.clone(), BOB_SESSION_KEYS.clone()),
            (CHARLIE.clone(), CHARLIE_SESSION_KEYS.clone()),
            (DAVE.clone(), DAVE_SESSION_KEYS.clone()),
            (EVE.clone(), EVE_SESSION_KEYS.clone()),
            (FERDIE.clone(), FERDIE_SESSION_KEYS.clone()),
        ])
        .build()
        .execute_with(|| {
            initialize_collators_through_whitelist(vec![
                ALICE.clone(),
                BOB.clone(),
                CHARLIE.clone(),
                DAVE.clone(),
                EVE.clone(),
                FERDIE.clone(),
            ]);
            // Increase bond for everyone but FERDIE
            for collator in vec![
                BOB.clone(),
                CHARLIE.clone(),
                DAVE.clone(),
                EVE.clone(),
                FERDIE.clone(),
            ] {
                assert_ok!(ParachainStaking::candidate_bond_more(
                    RuntimeOrigin::signed(collator.clone()),
                    MIN_BOND_TO_BE_CONSIDERED_COLLATOR - EARLY_COLLATOR_MINIMUM_STAKE
                ));
            }

            // Ensure CHARLIE and later are not selected
            // NOTE: Must use 6 or more collators because 5 is the minimum on calamari
            assert!(!ParachainStaking::compute_top_candidates().contains(&ALICE));
            assert!(ParachainStaking::compute_top_candidates().contains(&BOB));
            assert!(ParachainStaking::compute_top_candidates().contains(&CHARLIE));
            assert!(ParachainStaking::compute_top_candidates().contains(&DAVE));
            assert!(ParachainStaking::compute_top_candidates().contains(&EVE));
            assert!(ParachainStaking::compute_top_candidates().contains(&FERDIE));
        });
}

#[test]
fn calamari_vesting_works() {
    ExtBuilder::default().build().execute_with(|| {
        let unvested = 100 * KMA;
        assert_ok!(CalamariVesting::vested_transfer(
            RuntimeOrigin::signed(ALICE.clone()),
            sp_runtime::MultiAddress::Id(BOB.clone()),
            unvested
        ));

        assert_eq!(Balances::free_balance(BOB.clone()), 100 * KMA);
        assert_eq!(Balances::usable_balance(BOB.clone()), 0);

        let schedule = calamari_vesting::Pallet::<Runtime>::vesting_schedule();
        let mut vested = 0;

        for period in 0..schedule.len() {
            // Timestamp expects milliseconds, so multiply by 1_000 to convert from seconds.
            let now = schedule[period].1 * 1_000 + 1;
            Timestamp::set_timestamp(now);
            assert_ok!(CalamariVesting::vest(RuntimeOrigin::signed(BOB.clone())));
            vested += schedule[period].0 * unvested;
            assert_eq!(Balances::usable_balance(BOB.clone()), vested);
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
    is_pallet_prefix::<calamari_runtime::Treasury>("Treasury");
    is_pallet_prefix::<calamari_runtime::Scheduler>("Scheduler");
    is_pallet_prefix::<calamari_runtime::XcmpQueue>("XcmpQueue");
    is_pallet_prefix::<calamari_runtime::PolkadotXcm>("PolkadotXcm");
    is_pallet_prefix::<calamari_runtime::CumulusXcm>("CumulusXcm");
    is_pallet_prefix::<calamari_runtime::DmpQueue>("DmpQueue");
    is_pallet_prefix::<calamari_runtime::Utility>("Utility");
    is_pallet_prefix::<calamari_runtime::Multisig>("Multisig");
    is_pallet_prefix::<calamari_runtime::CalamariVesting>("CalamariVesting");
    is_pallet_prefix::<calamari_runtime::AuthorInherent>("AuthorInherent");
    is_pallet_prefix::<calamari_runtime::AuraAuthorFilter>("AuraAuthorFilter");
    is_pallet_prefix::<calamari_runtime::ParachainStaking>("ParachainStaking");

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
                storage_name: b"InactiveIssuance".to_vec(),
                prefix: prefix(b"Balances", b"InactiveIssuance"),
                max_values: Some(1),
                max_size: Some(16),
            },
            StorageInfo {
                pallet_name: b"Balances".to_vec(),
                storage_name: b"Account".to_vec(),
                prefix: prefix(b"Balances", b"Account"),
                max_values: None,
                max_size: Some(112),
            },
            StorageInfo {
                pallet_name: b"Balances".to_vec(),
                storage_name: b"Locks".to_vec(),
                prefix: prefix(b"Balances", b"Locks"),
                max_values: None,
                max_size: Some(1299),
            },
            StorageInfo {
                pallet_name: b"Balances".to_vec(),
                storage_name: b"Reserves".to_vec(),
                prefix: prefix(b"Balances", b"Reserves"),
                max_values: None,
                max_size: Some(1249),
            },
        ]
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
    is_pallet_index::<calamari_runtime::Treasury>(26);
    is_pallet_index::<calamari_runtime::Preimage>(28);
    is_pallet_index::<calamari_runtime::Scheduler>(29);
    is_pallet_index::<calamari_runtime::XcmpQueue>(30);
    is_pallet_index::<calamari_runtime::PolkadotXcm>(31);
    is_pallet_index::<calamari_runtime::CumulusXcm>(32);
    is_pallet_index::<calamari_runtime::DmpQueue>(33);
    is_pallet_index::<calamari_runtime::XTokens>(34);
    is_pallet_index::<calamari_runtime::Utility>(40);
    is_pallet_index::<calamari_runtime::Multisig>(41);
    is_pallet_index::<calamari_runtime::Assets>(45);
    is_pallet_index::<calamari_runtime::AssetManager>(46);
    is_pallet_index::<calamari_runtime::ParachainStaking>(48);
    is_pallet_index::<calamari_runtime::CalamariVesting>(50);
    is_pallet_index::<calamari_runtime::AuthorInherent>(60);
    is_pallet_index::<calamari_runtime::AuraAuthorFilter>(63);

    // Check removed pallets.
    ExtBuilder::default().build().execute_with(|| {
        use frame_support::metadata::{v14::META_RESERVED, RuntimeMetadata};

        let runtime_metadata = calamari_runtime::Runtime::metadata();
        assert_eq!(runtime_metadata.0, META_RESERVED);
        if let RuntimeMetadata::V14(v14) = runtime_metadata.1 {
            // Ensure sudo=42 has been removed, no one is taking this index.
            assert!(v14.pallets.iter().any(|pallet| pallet.index != 42));
            // AuraExt
            assert!(v14.pallets.iter().any(|pallet| pallet.index != 24));
        }
    });
}

#[test]
fn concrete_fungible_ledger_transfers_work() {
    ExtBuilder::default()
        .with_balances(vec![
            (ALICE.clone(), INITIAL_BALANCE),
            (BOB.clone(), INITIAL_BALANCE),
            (CHARLIE.clone(), INITIAL_BALANCE),
        ])
        .build()
        .execute_with(|| {
            let transfer_amount = 10 * KMA;
            let mut current_balance_alice = INITIAL_BALANCE;
            let mut current_balance_charlie = INITIAL_BALANCE;

            // Transfer tests for native assets:

            // Try to transfer more than available
            assert_err!(
                CalamariConcreteFungibleLedger::transfer(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                    &ALICE.clone(),
                    &CHARLIE.clone(),
                    INITIAL_BALANCE + 1,
                    ExistenceRequirement::KeepAlive
                ),
                FungibleLedgerError::InvalidTransfer(DispatchError::Module(ModuleError {
                    index: <calamari_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
                        Balances,
                    >()
                    .unwrap() as u8,
                    error: [2, 0, 0, 0],
                    message: Some("InsufficientBalance")
                }))
            );
            assert_eq!(Balances::free_balance(ALICE.clone()), current_balance_alice);
            assert_eq!(
                Balances::free_balance(CHARLIE.clone()),
                current_balance_charlie
            );

            // Try to transfer and go below existential deposit
            assert_err!(
                CalamariConcreteFungibleLedger::transfer(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                    &ALICE.clone(),
                    &CHARLIE.clone(),
                    INITIAL_BALANCE,
                    ExistenceRequirement::KeepAlive
                ),
                FungibleLedgerError::InvalidTransfer(DispatchError::Module(ModuleError {
                    index: <calamari_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
                        Balances,
                    >()
                    .unwrap() as u8,
                    error: [4, 0, 0, 0],
                    message: Some("KeepAlive")
                }))
            );
            assert_eq!(Balances::free_balance(ALICE.clone()), current_balance_alice);
            assert_eq!(
                Balances::free_balance(CHARLIE.clone()),
                current_balance_charlie
            );

            // A normal transfer should work
            assert_ok!(CalamariConcreteFungibleLedger::transfer(
                <CalamariAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                &ALICE.clone(),
                &CHARLIE.clone(),
                transfer_amount,
                ExistenceRequirement::KeepAlive
            ));
            current_balance_alice -= transfer_amount;
            current_balance_charlie += transfer_amount;
            assert_eq!(Balances::free_balance(ALICE.clone()), current_balance_alice);
            assert_eq!(
                Balances::free_balance(CHARLIE.clone()),
                current_balance_charlie
            );

            // Should not be able to create new account with lower than ED balance
            let new_account = unchecked_account_id::<sr25519::Public>("NewAccount");
            assert_err!(
                CalamariConcreteFungibleLedger::transfer(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                    &ALICE.clone(),
                    &new_account,
                    NativeTokenExistentialDeposit::get() - 1,
                    ExistenceRequirement::KeepAlive
                ),
                FungibleLedgerError::InvalidTransfer(DispatchError::Module(ModuleError {
                    index: <calamari_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
                        Balances,
                    >()
                    .unwrap() as u8,
                    error: [3, 0, 0, 0],
                    message: Some("ExistentialDeposit")
                }))
            );

            // Should be able to create new account with enough balance
            assert_ok!(CalamariConcreteFungibleLedger::transfer(
                <CalamariAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                &ALICE.clone(),
                &new_account,
                NativeTokenExistentialDeposit::get(),
                ExistenceRequirement::KeepAlive
            ));
            current_balance_alice -= NativeTokenExistentialDeposit::get();
            assert_eq!(Balances::free_balance(ALICE.clone()), current_balance_alice);
            assert_eq!(
                Balances::free_balance(new_account),
                NativeTokenExistentialDeposit::get()
            );

            // Transfer all of your balance without dropping below ED should work
            assert_ok!(CalamariConcreteFungibleLedger::transfer(
                <CalamariAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                &BOB.clone(),
                &ALICE.clone(),
                INITIAL_BALANCE - NativeTokenExistentialDeposit::get(),
                ExistenceRequirement::KeepAlive
            ));
            current_balance_alice += INITIAL_BALANCE - NativeTokenExistentialDeposit::get();
            assert_eq!(Balances::free_balance(ALICE.clone()), current_balance_alice);
            assert_eq!(
                Balances::free_balance(BOB.clone()),
                NativeTokenExistentialDeposit::get()
            );

            // Transfer the ED should work if AllowDeath is selected
            assert_ok!(CalamariConcreteFungibleLedger::transfer(
                <CalamariAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                &BOB.clone(),
                &ALICE.clone(),
                NativeTokenExistentialDeposit::get(),
                ExistenceRequirement::AllowDeath
            ));
            current_balance_alice += NativeTokenExistentialDeposit::get();
            assert_eq!(Balances::free_balance(ALICE.clone()), current_balance_alice);
            assert_eq!(Balances::free_balance(BOB.clone()), 0);
            assert!(!frame_system::Account::<Runtime>::contains_key(BOB.clone()));

            // Transfer tests for non-native assets:

            let min_balance = 10u128;
            let asset_metadata = AssetRegistryMetadata {
                metadata: AssetStorageMetadata {
                    name: b"Kusama".to_vec(),
                    symbol: b"KSM".to_vec(),
                    decimals: 12,
                    is_frozen: false,
                },
                min_balance,
                is_sufficient: true,
            };
            let source_location =
                AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
            assert_ok!(AssetManager::register_asset(
                root_origin(),
                source_location,
                asset_metadata
            ),);

            // Register and mint for testing.
            let amount = Balance::MAX;
            assert_ok!(CalamariConcreteFungibleLedger::deposit_minting(
                <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                &ALICE.clone(),
                amount,
            ),);
            assert_eq!(
                Assets::balance(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    ALICE.clone()
                ),
                amount
            );

            // Transferring and falling below ED of the asset should not work with KeepAlive.
            assert_err!(
                CalamariConcreteFungibleLedger::transfer(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    &ALICE.clone(),
                    &BOB.clone(),
                    amount,
                    ExistenceRequirement::KeepAlive
                ),
                FungibleLedgerError::InvalidTransfer(DispatchError::Module(ModuleError {
                    index: <calamari_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
                        Assets,
                    >()
                    .unwrap() as u8,
                    error: [0, 0, 0, 0],
                    message: Some("BalanceLow")
                }))
            );
            assert_eq!(
                Assets::balance(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    ALICE.clone()
                ),
                amount
            );

            assert_err!(
                CalamariConcreteFungibleLedger::transfer(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    &ALICE.clone(),
                    &BOB.clone(),
                    min_balance - 1,
                    ExistenceRequirement::KeepAlive
                ),
                FungibleLedgerError::InvalidTransfer(DispatchError::Token(
                    sp_runtime::TokenError::BelowMinimum
                ))
            );
            assert_eq!(
                Assets::balance(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    ALICE.clone()
                ),
                amount
            );

            // Transferring normal amounts should work.
            assert_ok!(CalamariConcreteFungibleLedger::transfer(
                <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                &ALICE.clone(),
                &BOB.clone(),
                transfer_amount,
                ExistenceRequirement::KeepAlive
            ),);
            assert_eq!(
                Assets::balance(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    ALICE.clone()
                ),
                u128::MAX - transfer_amount
            );
            assert_eq!(
                Assets::balance(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    BOB.clone()
                ),
                transfer_amount
            );

            // Transferring all of the balance of an account should work.
            assert_ok!(CalamariConcreteFungibleLedger::transfer(
                <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                &BOB.clone(),
                &ALICE.clone(),
                transfer_amount,
                ExistenceRequirement::AllowDeath
            ),);
            assert_eq!(
                Assets::balance(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    BOB.clone()
                ),
                0
            );

            // Transferring unregistered asset ID should not work.
            assert_err!(
                CalamariConcreteFungibleLedger::transfer(
                    CalamariAssetId::MAX,
                    &ALICE.clone(),
                    &CHARLIE.clone(),
                    transfer_amount,
                    ExistenceRequirement::KeepAlive
                ),
                FungibleLedgerError::InvalidTransfer(DispatchError::Module(ModuleError {
                    index: <calamari_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
                        Assets,
                    >()
                    .unwrap() as u8,
                    error: [3, 0, 0, 0],
                    message: Some("Unknown")
                }))
            );
        });
}

#[test]
fn concrete_fungible_ledger_can_deposit_and_mint_works() {
    ExtBuilder::default()
        .with_balances(vec![(ALICE.clone(), INITIAL_BALANCE)])
        .build()
        .execute_with(|| {
            // Native asset tests:

            let new_account = unchecked_account_id::<sr25519::Public>("NewAccount");
            assert_err!(
                CalamariConcreteFungibleLedger::can_deposit(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                    &new_account,
                    NativeTokenExistentialDeposit::get() - 1,
                    true,
                ),
                FungibleLedgerError::BelowMinimum
            );

            // Non-native asset tests:

            let min_balance = 10u128;
            let asset_metadata = AssetRegistryMetadata {
                metadata: AssetStorageMetadata {
                    name: b"Kusama".to_vec(),
                    symbol: b"KSM".to_vec(),
                    decimals: 12,
                    is_frozen: false,
                },
                min_balance,
                is_sufficient: true,
            };
            let source_location =
                AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
            assert_ok!(AssetManager::register_asset(
                root_origin(),
                source_location,
                asset_metadata
            ),);

            assert_err!(
                CalamariConcreteFungibleLedger::can_deposit(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    &ALICE.clone(),
                    0,
                    true,
                ),
                FungibleLedgerError::BelowMinimum
            );
            assert_err!(
                CalamariConcreteFungibleLedger::can_deposit(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get() + 1,
                    &ALICE.clone(),
                    11,
                    true,
                ),
                FungibleLedgerError::UnknownAsset
            );
            assert_ok!(CalamariConcreteFungibleLedger::deposit_minting(
                <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                &ALICE.clone(),
                u128::MAX,
            ),);
            assert_eq!(
                Assets::balance(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    ALICE.clone()
                ),
                u128::MAX
            );
            assert_err!(
                CalamariConcreteFungibleLedger::can_deposit(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    &ALICE.clone(),
                    1,
                    true,
                ),
                FungibleLedgerError::Overflow
            );

            let asset_metadata = AssetRegistryMetadata {
                metadata: AssetStorageMetadata {
                    name: b"Rococo".to_vec(),
                    symbol: b"Roc".to_vec(),
                    decimals: 12,
                    is_frozen: false,
                },
                min_balance,
                is_sufficient: false,
            };

            let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
                1,
                X2(Parachain(1), PalletInstance(1)),
            )));
            assert_ok!(AssetManager::register_asset(
                root_origin(),
                source_location,
                asset_metadata
            ),);
            assert_err!(
                CalamariConcreteFungibleLedger::can_deposit(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get() + 1,
                    &XcmFeesAccount::get(),
                    11,
                    true,
                ),
                FungibleLedgerError::CannotCreate
            );
        });
}

// `can_withdraw` uses `reducible_amount` implementation in order to use the `keep_alive` argument.
// Unfortunately that function does not return the reason for failure cases like `can_withdraw`.
// The errors that would've been returned if `can_withdraw` was used instead of `reducible_amount`
// are included as comments on top of each case for more clarity.
#[test]
fn concrete_fungible_ledger_can_withdraw_works() {
    ExtBuilder::default()
        .with_balances(vec![(CHARLIE.clone(), INITIAL_BALANCE)])
        .build()
        .execute_with(|| {
            let existential_deposit = NativeTokenExistentialDeposit::get();

            // Native asset tests:

            assert_err!(
                CalamariConcreteFungibleLedger::can_withdraw(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                    &CHARLIE.clone(),
                    &(INITIAL_BALANCE + 1),
                    ExistenceRequirement::KeepAlive
                ),
                // Underflow
                FungibleLedgerError::CannotWithdrawMoreThan(INITIAL_BALANCE - existential_deposit)
            );

            assert_err!(
                CalamariConcreteFungibleLedger::can_withdraw(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                    &CHARLIE.clone(),
                    &INITIAL_BALANCE,
                    ExistenceRequirement::KeepAlive
                ),
                // WouldDie
                FungibleLedgerError::CannotWithdrawMoreThan(INITIAL_BALANCE - existential_deposit)
            );

            assert_ok!(CalamariConcreteFungibleLedger::can_withdraw(
                <CalamariAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                &CHARLIE.clone(),
                &INITIAL_BALANCE,
                ExistenceRequirement::AllowDeath
            ),);

            assert_err!(
                CalamariConcreteFungibleLedger::can_withdraw(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                    &BOB.clone(),
                    &INITIAL_BALANCE,
                    ExistenceRequirement::KeepAlive
                ),
                // NoFunds
                FungibleLedgerError::CannotWithdrawMoreThan(0)
            );

            // Non-native asset tests:

            let min_balance = 10u128;
            let asset_metadata = AssetRegistryMetadata {
                metadata: AssetStorageMetadata {
                    name: b"Kusama".to_vec(),
                    symbol: b"KSM".to_vec(),
                    decimals: 12,
                    is_frozen: false,
                },
                min_balance,
                is_sufficient: true,
            };
            let source_location =
                AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
            assert_ok!(AssetManager::register_asset(
                root_origin(),
                source_location,
                asset_metadata
            ),);

            assert_ok!(CalamariConcreteFungibleLedger::deposit_minting(
                <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                &ALICE.clone(),
                INITIAL_BALANCE,
            ),);
            assert_eq!(
                Assets::balance(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    ALICE.clone()
                ),
                INITIAL_BALANCE
            );

            assert_err!(
                CalamariConcreteFungibleLedger::can_withdraw(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    &ALICE.clone(),
                    &(INITIAL_BALANCE + 1),
                    ExistenceRequirement::AllowDeath
                ),
                // Underflow
                FungibleLedgerError::CannotWithdrawMoreThan(INITIAL_BALANCE)
            );

            assert_ok!(CalamariConcreteFungibleLedger::can_withdraw(
                <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                &ALICE.clone(),
                &INITIAL_BALANCE,
                ExistenceRequirement::AllowDeath
            ),);

            assert_err!(
                CalamariConcreteFungibleLedger::can_withdraw(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    &BOB.clone(),
                    &10u128,
                    ExistenceRequirement::AllowDeath
                ),
                // NoFunds
                FungibleLedgerError::CannotWithdrawMoreThan(0)
            );

            assert_ok!(CalamariConcreteFungibleLedger::deposit_minting(
                <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                &BOB.clone(),
                INITIAL_BALANCE,
            ),);
            assert_err!(
                CalamariConcreteFungibleLedger::can_withdraw(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    &ALICE.clone(),
                    &INITIAL_BALANCE,
                    ExistenceRequirement::KeepAlive
                ),
                FungibleLedgerError::CannotWithdrawMoreThan(INITIAL_BALANCE - min_balance)
            );

            assert_ok!(Assets::freeze(
                RuntimeOrigin::signed(AssetManager::account_id()),
                <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                sp_runtime::MultiAddress::Id(ALICE.clone()),
            ));
            assert_err!(
                CalamariConcreteFungibleLedger::can_withdraw(
                    <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    &ALICE.clone(),
                    &10u128,
                    ExistenceRequirement::AllowDeath
                ),
                // Frozen
                FungibleLedgerError::CannotWithdrawMoreThan(0)
            );
        });
}

#[test]
fn test_receiver_side_weights() {
    let weight = <XcmExecutorConfig as xcm_executor::Config>::Weigher::weight(
        &mut self_reserve_xcm_message_receiver_side::<RuntimeCall>(),
    )
    .unwrap();
    assert!(weight <= ADVERTISED_DEST_WEIGHT);

    let weight = <XcmExecutorConfig as xcm_executor::Config>::Weigher::weight(
        &mut to_reserve_xcm_message_receiver_side::<RuntimeCall>(),
    )
    .unwrap();
    assert!(weight <= ADVERTISED_DEST_WEIGHT);
}

#[test]
fn test_sender_side_xcm_weights() {
    let mut msg = self_reserve_xcm_message_sender_side::<RuntimeCall>();
    let weight = <XcmExecutorConfig as xcm_executor::Config>::Weigher::weight(&mut msg).unwrap();
    assert!(weight < ADVERTISED_DEST_WEIGHT);

    let mut msg = to_reserve_xcm_message_sender_side::<RuntimeCall>();
    let weight = <XcmExecutorConfig as xcm_executor::Config>::Weigher::weight(&mut msg).unwrap();
    assert!(weight < ADVERTISED_DEST_WEIGHT);
}

const PALLET_NAME: &[u8] = b"Utility";
const FUNCTION_NAME: &[u8] = b"batch";

fn pause_transaction_storage_event_works(pause: bool) {
    if pause {
        System::assert_last_event(Event::TransactionPause(
            pallet_tx_pause::Event::<Runtime>::TransactionPaused(
                PALLET_NAME.to_vec(),
                FUNCTION_NAME.to_vec(),
            ),
        ));
        assert_eq!(
            TransactionPause::paused_transactions((PALLET_NAME.to_vec(), FUNCTION_NAME.to_vec(),)),
            Some(())
        );
    } else {
        System::assert_has_event(Event::TransactionPause(
            pallet_tx_pause::Event::<Runtime>::TransactionUnpaused(
                PALLET_NAME.to_vec(),
                FUNCTION_NAME.to_vec(),
            ),
        ));
        assert_eq!(
            TransactionPause::paused_transactions((PALLET_NAME.to_vec(), FUNCTION_NAME.to_vec(),)),
            None
        );
    }
}

fn pause_transactions_storage_event_works(pause: bool, pallet: bool) {
    let function_names: Vec<Vec<u8>> = vec![b"batch".to_vec(), b"batch_all".to_vec()];

    if pause {
        if pallet {
            System::assert_last_event(Event::TransactionPause(
                pallet_tx_pause::Event::<Runtime>::PalletPaused(PALLET_NAME.to_vec()),
            ));
        } else {
            for function_name in function_names.clone() {
                System::assert_has_event(Event::TransactionPause(
                    pallet_tx_pause::Event::<Runtime>::TransactionPaused(
                        PALLET_NAME.to_vec(),
                        function_name,
                    ),
                ));
            }
        }
        for function_name in function_names {
            assert_eq!(
                TransactionPause::paused_transactions((PALLET_NAME.to_vec(), function_name)),
                Some(())
            );
        }
    } else {
        if pallet {
            System::assert_last_event(Event::TransactionPause(
                pallet_tx_pause::Event::<Runtime>::PalletUnpaused(PALLET_NAME.to_vec()),
            ));
        } else {
            for function_name in function_names.clone() {
                System::assert_has_event(Event::TransactionPause(
                    pallet_tx_pause::Event::<Runtime>::TransactionUnpaused(
                        PALLET_NAME.to_vec(),
                        function_name,
                    ),
                ));
            }
        }
        for function_name in function_names {
            assert_eq!(
                TransactionPause::paused_transactions((PALLET_NAME.to_vec(), function_name)),
                None
            );
        }
    }
}

#[test]
fn tx_pause_works() {
    let alice = unchecked_account_id::<sr25519::Public>("Alice");
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            TransactionPause::pause_transaction(
                RuntimeOrigin::signed(alice),
                b"Balances".to_vec(),
                b"transfer".to_vec()
            ),
            BadOrigin
        );
        assert_noop!(
            TransactionPause::pause_pallets(root_origin(), vec![b"Balances".to_vec()]),
            pallet_tx_pause::Error::<Runtime>::CannotPause
        );
        assert_noop!(
            TransactionPause::pause_pallets(root_origin(), vec![b"TransactionPause".to_vec()]),
            pallet_tx_pause::Error::<Runtime>::CannotPause
        );

        let function_names: Vec<Vec<u8>> = vec![b"batch".to_vec(), b"batch_all".to_vec()];

        // pause transaction
        assert_ok!(TransactionPause::pause_transaction(
            root_origin(),
            PALLET_NAME.to_vec(),
            FUNCTION_NAME.to_vec(),
        ));
        pause_transaction_storage_event_works(true);

        // unpause transaction
        assert_ok!(TransactionPause::unpause_transaction(
            root_origin(),
            PALLET_NAME.to_vec(),
            FUNCTION_NAME.to_vec(),
        ));
        pause_transaction_storage_event_works(false);

        // pause transactions
        System::reset_events();
        assert_ok!(TransactionPause::pause_transactions(
            root_origin(),
            vec![(PALLET_NAME.to_vec(), function_names.clone())]
        ));
        pause_transactions_storage_event_works(true, false);

        // unpause transactions
        assert_ok!(TransactionPause::unpause_transactions(
            root_origin(),
            vec![(PALLET_NAME.to_vec(), function_names)]
        ));
        pause_transactions_storage_event_works(false, false);

        // pause pallet
        assert_ok!(TransactionPause::pause_pallets(
            root_origin(),
            vec![PALLET_NAME.to_vec()]
        ));
        pause_transactions_storage_event_works(true, true);

        // unpause pallet
        assert_ok!(TransactionPause::unpause_pallets(
            root_origin(),
            vec![PALLET_NAME.to_vec()]
        ));
        pause_transactions_storage_event_works(false, true);
    });
}
