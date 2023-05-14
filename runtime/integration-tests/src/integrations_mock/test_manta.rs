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

#![cfg(feature = "manta")]

use super::{mock::ExtBuilder, *};
use frame_support::{
    assert_ok,
    codec::Encode,
    traits::{PalletInfo, StorageInfo, StorageInfoTrait, StorePreimage},
    StorageHasher, Twox128,
};
use manta_primitives::{constants::time::DAYS, types::AccountId};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, Hash},
    DispatchError, ModuleError,
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
        manta_runtime::RuntimeEvent::Council(pallet_collective::Event::Executed {
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
        RuntimeEvent::Council(pallet_collective::Event::Executed {
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
        RuntimeEvent::Scheduler(pallet_scheduler::Event::Scheduled {
            when: time_of_enactment,
            index: referendum_index
        })
    );

    // After the enactment period the proposal is dispatched:
    run_to_block(time_of_enactment);
    assert_eq!(
        last_event(),
        RuntimeEvent::Scheduler(pallet_scheduler::Event::Dispatched {
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
        RuntimeEvent::Council(pallet_collective::Event::Executed {
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
    assert!(<manta_runtime::Runtime as pallet_democracy::Config>::InstantAllowed::get());
}

#[test]
fn sanity_check_governance_periods() {
    assert_eq!(LaunchPeriod::get(), 7 * DAYS);
    assert_eq!(VotingPeriod::get(), 7 * DAYS);
    assert_eq!(EnactmentPeriod::get(), DAYS);
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
            RuntimeEvent::Democracy(pallet_democracy::Event::Started {
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
        assert_eq!(1, Democracy::referendum_count());
        assert_eq!(
            last_event(),
            RuntimeEvent::Democracy(pallet_democracy::Event::Started {
                ref_index: referendum_index,
                threshold: pallet_democracy::VoteThreshold::SuperMajorityAgainst
            })
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
                balance: 100 * KMA
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
            RuntimeEvent::TechnicalCommittee(pallet_collective::Event::Executed {
                proposal_hash: tech_committee_motion_hash,
                result: Ok(())
            })
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
    assert!(<manta_runtime::Runtime as pallet_democracy::Config>::InstantAllowed::get());

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
fn verify_pallet_prefixes() {
    fn is_pallet_prefix<P: 'static>(name: &str) {
        // Compares the unhashed pallet prefix in the `StorageInstance` implementation by every
        // storage item in the pallet P. This pallet prefix is used in conjunction with the
        // item name to get the unique storage key: hash(PalletPrefix) + hash(StorageName)
        // https://github.com/paritytech/substrate/blob/master/frame/support/procedural/src/pallet/
        // expand/storage.rs#L389-L401
        assert_eq!(
            <manta_runtime::Runtime as frame_system::Config>::PalletInfo::name::<P>(),
            Some(name)
        );
    }

    is_pallet_prefix::<manta_runtime::System>("System");
    is_pallet_prefix::<manta_runtime::ParachainSystem>("ParachainSystem");
    is_pallet_prefix::<manta_runtime::Timestamp>("Timestamp");
    is_pallet_prefix::<manta_runtime::ParachainInfo>("ParachainInfo");

    is_pallet_prefix::<manta_runtime::Balances>("Balances");
    is_pallet_prefix::<manta_runtime::TransactionPayment>("TransactionPayment");

    is_pallet_prefix::<manta_runtime::ParachainStaking>("ParachainStaking");

    is_pallet_prefix::<manta_runtime::AuthorInherent>("AuthorInherent");
    is_pallet_prefix::<manta_runtime::AuraAuthorFilter>("AuraAuthorFilter");

    is_pallet_prefix::<manta_runtime::Authorship>("Authorship");
    is_pallet_prefix::<manta_runtime::CollatorSelection>("CollatorSelection");
    is_pallet_prefix::<manta_runtime::Session>("Session");
    is_pallet_prefix::<manta_runtime::Aura>("Aura");

    is_pallet_prefix::<manta_runtime::Treasury>("Treasury");

    is_pallet_prefix::<manta_runtime::Preimage>("Preimage");

    is_pallet_prefix::<manta_runtime::XcmpQueue>("XcmpQueue");
    is_pallet_prefix::<manta_runtime::PolkadotXcm>("PolkadotXcm");
    is_pallet_prefix::<manta_runtime::CumulusXcm>("CumulusXcm");
    is_pallet_prefix::<manta_runtime::DmpQueue>("DmpQueue");
    is_pallet_prefix::<manta_runtime::XTokens>("XTokens");

    is_pallet_prefix::<manta_runtime::Utility>("Utility");
    is_pallet_prefix::<manta_runtime::Multisig>("Multisig");

    is_pallet_prefix::<manta_runtime::Sudo>("Sudo");

    is_pallet_prefix::<manta_runtime::Assets>("Assets");
    is_pallet_prefix::<manta_runtime::AssetManager>("AssetManager");

    let prefix = |pallet_name, storage_name| {
        let mut res = [0u8; 32];
        res[0..16].copy_from_slice(&Twox128::hash(pallet_name));
        res[16..32].copy_from_slice(&Twox128::hash(storage_name));
        res.to_vec()
    };
    assert_eq!(
        <Timestamp as StorageInfoTrait>::storage_info(),
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
        <Balances as StorageInfoTrait>::storage_info(),
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
fn verify_manta_pallet_indices() {
    fn is_pallet_index<P: 'static>(index: usize) {
        assert_eq!(
            <manta_runtime::Runtime as frame_system::Config>::PalletInfo::index::<P>(),
            Some(index)
        );
    }

    is_pallet_index::<manta_runtime::System>(0);
    is_pallet_index::<manta_runtime::ParachainSystem>(1);
    is_pallet_index::<manta_runtime::Timestamp>(2);
    is_pallet_index::<manta_runtime::ParachainInfo>(3);

    is_pallet_index::<manta_runtime::Balances>(10);
    is_pallet_index::<manta_runtime::TransactionPayment>(11);

    is_pallet_index::<manta_runtime::ParachainStaking>(48);

    is_pallet_index::<manta_runtime::AuthorInherent>(60);
    is_pallet_index::<manta_runtime::AuraAuthorFilter>(63);

    is_pallet_index::<manta_runtime::Authorship>(20);
    is_pallet_index::<manta_runtime::CollatorSelection>(21);
    is_pallet_index::<manta_runtime::Session>(22);
    is_pallet_index::<manta_runtime::Aura>(23);

    is_pallet_index::<manta_runtime::Treasury>(26);

    is_pallet_index::<manta_runtime::Preimage>(28);

    is_pallet_index::<manta_runtime::XcmpQueue>(30);
    is_pallet_index::<manta_runtime::PolkadotXcm>(31);
    is_pallet_index::<manta_runtime::CumulusXcm>(32);
    is_pallet_index::<manta_runtime::DmpQueue>(33);
    is_pallet_index::<manta_runtime::XTokens>(34);

    is_pallet_index::<manta_runtime::Utility>(40);
    is_pallet_index::<manta_runtime::Multisig>(41);

    is_pallet_index::<manta_runtime::Sudo>(42);

    is_pallet_index::<manta_runtime::Assets>(45);
    is_pallet_index::<manta_runtime::AssetManager>(46);
}
