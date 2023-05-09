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

use crate as collator_selection;
use crate::{
    mock::*, BlocksPerCollatorThisSession, CandidateInfo, Error, EvictionBaseline,
    EvictionTolerance,
};
use frame_support::{
    assert_noop, assert_ok,
    traits::{Currency, GenesisBuild, OnInitialize, ReservableCurrency},
};
use pallet_balances::Error as BalancesError;
use sp_arithmetic::Percent;
use sp_runtime::{testing::UintAuthorityId, traits::BadOrigin};

const ALICE: u64 = 1;
const BOB: u64 = 2;
const CHAD: u64 = 3;
const DAVE: u64 = 4; // NOTE: As defined in mock, dave will author all blocks
const EVE: u64 = 5;

fn candidate_ids() -> Vec<u64> {
    CollatorSelection::candidates()
        .iter()
        .map(|c| c.who)
        .collect::<Vec<_>>()
}
fn set_all_validator_perf_to(n: u32) {
    for v in Session::validators() {
        BlocksPerCollatorThisSession::<Test>::insert(v, n);
    }
}
fn setup_3_candidates() {
    assert_ok!(CollatorSelection::set_desired_candidates(
        RuntimeOrigin::signed(RootAccount::get()),
        3
    ));
    assert_ok!(CollatorSelection::register_as_candidate(
        RuntimeOrigin::signed(CHAD)
    ));
    assert_ok!(Session::set_keys(
        RuntimeOrigin::signed(CHAD),
        UintAuthorityId(CHAD).into(),
        vec![]
    ));
    assert_ok!(CollatorSelection::register_as_candidate(
        RuntimeOrigin::signed(DAVE)
    ));
    assert_ok!(Session::set_keys(
        RuntimeOrigin::signed(DAVE),
        UintAuthorityId(DAVE).into(),
        vec![]
    ));
    assert_ok!(CollatorSelection::register_as_candidate(
        RuntimeOrigin::signed(EVE)
    ));
    assert_ok!(Session::set_keys(
        RuntimeOrigin::signed(EVE),
        UintAuthorityId(EVE).into(),
        vec![]
    ));
}

#[test]
fn basic_setup_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(CollatorSelection::desired_candidates(), 2);
        assert_eq!(CollatorSelection::candidacy_bond(), 10);

        assert!(CollatorSelection::candidates().is_empty());
        assert_eq!(CollatorSelection::invulnerables(), vec![1, 2]);
    });
}

#[test]
fn it_should_set_invulnerables() {
    new_test_ext().execute_with(|| {
        let new_set = vec![1, 2, 3, 4];
        assert_ok!(CollatorSelection::set_invulnerables(
            RuntimeOrigin::signed(RootAccount::get()),
            new_set.clone()
        ));
        assert_eq!(CollatorSelection::invulnerables(), new_set);

        // cannot set with non-root.
        assert_noop!(
            CollatorSelection::set_invulnerables(RuntimeOrigin::signed(1), new_set),
            BadOrigin
        );
    });
}

#[test]
fn set_desired_candidates_works() {
    new_test_ext().execute_with(|| {
        // given
        assert_eq!(CollatorSelection::desired_candidates(), 2);

        // can set
        assert_ok!(CollatorSelection::set_desired_candidates(
            RuntimeOrigin::signed(RootAccount::get()),
            7
        ));
        assert_eq!(CollatorSelection::desired_candidates(), 7);

        // rejects bad origin
        assert_noop!(
            CollatorSelection::set_desired_candidates(RuntimeOrigin::signed(1), 8),
            BadOrigin
        );
    });
}

#[test]
fn set_candidacy_bond() {
    new_test_ext().execute_with(|| {
        // given
        assert_eq!(CollatorSelection::candidacy_bond(), 10);

        // can set
        assert_ok!(CollatorSelection::set_candidacy_bond(
            RuntimeOrigin::signed(RootAccount::get()),
            7
        ));
        assert_eq!(CollatorSelection::candidacy_bond(), 7);

        // rejects bad origin.
        assert_noop!(
            CollatorSelection::set_candidacy_bond(RuntimeOrigin::signed(1), 8),
            BadOrigin
        );
    });
}

#[test]
fn set_eviction_baseline() {
    new_test_ext().execute_with(|| {
        // given
        assert_eq!(
            CollatorSelection::eviction_baseline(),
            Percent::from_percent(80)
        );

        // can set
        assert_ok!(CollatorSelection::set_eviction_baseline(
            RuntimeOrigin::signed(RootAccount::get()),
            Percent::from_percent(100)
        ));
        assert_eq!(
            CollatorSelection::eviction_baseline(),
            Percent::from_percent(100)
        );

        // saturates to 100
        assert_ok!(CollatorSelection::set_eviction_baseline(
            RuntimeOrigin::signed(RootAccount::get()),
            Percent::from_percent(101)
        ));
        assert_eq!(
            CollatorSelection::eviction_baseline(),
            Percent::from_percent(100)
        );

        // rejects bad origin.
        assert_noop!(
            CollatorSelection::set_eviction_baseline(
                RuntimeOrigin::signed(1),
                Percent::from_percent(8)
            ),
            BadOrigin
        );
    });
}

#[test]
fn set_eviction_tolerance() {
    new_test_ext().execute_with(|| {
        // given
        assert_eq!(
            CollatorSelection::eviction_tolerance(),
            Percent::from_percent(10)
        );

        // can set
        assert_ok!(CollatorSelection::set_eviction_tolerance(
            RuntimeOrigin::signed(RootAccount::get()),
            Percent::from_percent(5)
        ));
        assert_eq!(
            CollatorSelection::eviction_tolerance(),
            Percent::from_percent(5)
        );

        // saturates to 100
        assert_ok!(CollatorSelection::set_eviction_tolerance(
            RuntimeOrigin::signed(RootAccount::get()),
            Percent::from_percent(101)
        ));
        assert_eq!(
            CollatorSelection::eviction_tolerance(),
            Percent::from_percent(100)
        );

        // rejects bad origin.
        assert_noop!(
            CollatorSelection::set_eviction_tolerance(
                RuntimeOrigin::signed(1),
                Percent::from_percent(8)
            ),
            BadOrigin
        );
    });
}
#[test]
fn cannot_register_candidate_if_too_many() {
    new_test_ext().execute_with(|| {
        // reset desired candidates:
        <crate::DesiredCandidates<Test>>::put(0);

        // can't accept anyone anymore.
        assert_noop!(
            CollatorSelection::register_as_candidate(RuntimeOrigin::signed(3)),
            Error::<Test>::TooManyCandidates,
        );

        // reset desired candidates:
        <crate::DesiredCandidates<Test>>::put(1);
        assert_ok!(CollatorSelection::register_as_candidate(
            RuntimeOrigin::signed(4)
        ));

        // but no more
        assert_noop!(
            CollatorSelection::register_as_candidate(RuntimeOrigin::signed(5)),
            Error::<Test>::TooManyCandidates,
        );
    })
}

#[test]
fn cannot_register_as_candidate_if_invulnerable() {
    new_test_ext().execute_with(|| {
        assert_eq!(CollatorSelection::invulnerables(), vec![1, 2]);

        // can't 1 because it is invulnerable.
        assert_noop!(
            CollatorSelection::register_as_candidate(RuntimeOrigin::signed(1)),
            Error::<Test>::AlreadyInvulnerable,
        );
    })
}

#[test]
fn cannot_register_as_candidate_if_keys_not_registered() {
    new_test_ext().execute_with(|| {
        // can't 7 because keys not registered.
        assert_noop!(
            CollatorSelection::register_as_candidate(RuntimeOrigin::signed(7)),
            Error::<Test>::ValidatorNotRegistered
        );
    })
}

#[test]
fn cannot_register_dupe_candidate() {
    new_test_ext().execute_with(|| {
        // can add 3 as candidate
        assert_ok!(CollatorSelection::register_as_candidate(
            RuntimeOrigin::signed(3)
        ));
        let addition = CandidateInfo {
            who: 3,
            deposit: 10,
        };
        assert_eq!(CollatorSelection::candidates(), vec![addition]);
        assert_eq!(Balances::free_balance(3), 90);

        // but no more
        assert_noop!(
            CollatorSelection::register_as_candidate(RuntimeOrigin::signed(3)),
            Error::<Test>::AlreadyCandidate,
        );
    })
}

#[test]
fn cannot_register_as_candidate_if_poor() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::free_balance(3), 100);
        assert_eq!(Balances::free_balance(33), 0);

        // works
        assert_ok!(CollatorSelection::register_as_candidate(
            RuntimeOrigin::signed(3)
        ));

        // poor
        assert_noop!(
            CollatorSelection::register_as_candidate(RuntimeOrigin::signed(33)),
            BalancesError::<Test>::InsufficientBalance,
        );
    });
}

#[test]
fn register_as_candidate_works() {
    new_test_ext().execute_with(|| {
        // given
        assert_eq!(CollatorSelection::desired_candidates(), 2);
        assert_eq!(CollatorSelection::candidacy_bond(), 10);
        assert_eq!(CollatorSelection::candidates(), Vec::new());
        assert_eq!(CollatorSelection::invulnerables(), vec![1, 2]);

        // take two endowed, non-invulnerables accounts.
        assert_eq!(Balances::free_balance(3), 100);
        assert_eq!(Balances::free_balance(4), 100);

        assert_ok!(CollatorSelection::register_as_candidate(
            RuntimeOrigin::signed(3)
        ));
        assert_ok!(CollatorSelection::register_as_candidate(
            RuntimeOrigin::signed(4)
        ));

        assert_eq!(Balances::free_balance(3), 90);
        assert_eq!(Balances::free_balance(4), 90);

        assert_eq!(CollatorSelection::candidates().len(), 2);
    });
}

#[test]
fn leave_intent() {
    new_test_ext().execute_with(|| {
        // register a candidate.
        assert_ok!(CollatorSelection::register_as_candidate(
            RuntimeOrigin::signed(3)
        ));
        assert_eq!(Balances::free_balance(3), 90);

        // register too so can leave above min candidates
        assert_ok!(CollatorSelection::register_as_candidate(
            RuntimeOrigin::signed(5)
        ));
        assert_eq!(Balances::free_balance(5), 90);

        // cannot leave if not candidate.
        assert_noop!(
            CollatorSelection::leave_intent(RuntimeOrigin::signed(4)),
            Error::<Test>::NotCandidate
        );

        // bond is returned
        assert_ok!(CollatorSelection::leave_intent(RuntimeOrigin::signed(3)));
        assert_eq!(Balances::free_balance(3), 100);
    });
}

#[test]
fn authorship_event_handler() {
    new_test_ext().execute_with(|| {
        // put 100 in the pot + 5 for ED
        Balances::make_free_balance_be(&CollatorSelection::account_id(), 105);

        // 4 is the default author.
        assert_eq!(Balances::free_balance(4), 100);
        assert_ok!(CollatorSelection::register_as_candidate(
            RuntimeOrigin::signed(4)
        ));
        // triggers `note_author`
        Authorship::on_initialize(1);

        let collator = CandidateInfo {
            who: 4,
            deposit: 10,
        };

        assert_eq!(CollatorSelection::candidates(), vec![collator]);

        // half of the pot goes to the collator who's the author (4 in tests).
        assert_eq!(Balances::free_balance(4), 140);
        // half + ED stays.
        assert_eq!(Balances::free_balance(CollatorSelection::account_id()), 55);
    });
}

#[test]
fn fees_edgecases() {
    new_test_ext().execute_with(|| {
        // Nothing panics, no reward when no ED in balance
        Authorship::on_initialize(1);
        // put some money into the pot at ED
        Balances::make_free_balance_be(&CollatorSelection::account_id(), 5);
        // 4 is the default author.
        assert_eq!(Balances::free_balance(4), 100);
        assert_ok!(CollatorSelection::register_as_candidate(
            RuntimeOrigin::signed(4)
        ));
        // triggers `note_author`
        Authorship::on_initialize(1);

        let collator = CandidateInfo {
            who: 4,
            deposit: 10,
        };

        assert_eq!(CollatorSelection::candidates(), vec![collator]);
        // Nothing received
        assert_eq!(Balances::free_balance(4), 90);
        // all fee stays
        assert_eq!(Balances::free_balance(CollatorSelection::account_id()), 5);
    });
}

#[test]
fn session_management_works() {
    new_test_ext().execute_with(|| {
        initialize_to_block(1);

        assert_eq!(SessionChangeBlock::get(), 0);
        assert_eq!(SessionHandlerCollators::get(), vec![1, 2]);

        initialize_to_block(4);

        assert_eq!(SessionChangeBlock::get(), 0);
        assert_eq!(SessionHandlerCollators::get(), vec![1, 2]);

        // add a new collator
        assert_ok!(CollatorSelection::register_as_candidate(
            RuntimeOrigin::signed(3)
        ));
        assert_ok!(Session::set_keys(
            RuntimeOrigin::signed(3),
            UintAuthorityId(3).into(),
            vec![]
        ));
        // session won't see this.
        assert_eq!(SessionHandlerCollators::get(), vec![1, 2]);
        // but we have a new candidate.
        assert_eq!(CollatorSelection::candidates().len(), 1);

        initialize_to_block(10);
        assert_eq!(SessionChangeBlock::get(), 10);
        // pallet-session has 1 session delay; current validators are the same.
        assert_eq!(Session::validators(), vec![1, 2]);
        // queued ones are changed, and now we have 3.
        assert_eq!(Session::queued_keys().len(), 3);
        // session handlers (aura, et. al.) cannot see this yet.
        assert_eq!(SessionHandlerCollators::get(), vec![1, 2]);

        initialize_to_block(20);
        assert_eq!(SessionChangeBlock::get(), 20);
        // changed are now reflected to session handlers.
        assert_eq!(SessionHandlerCollators::get(), vec![1, 2, 3]);
    });
}

#[test]
fn kick_mechanism_parity() {
    new_test_ext().execute_with(|| {
        assert_ok!(CollatorSelection::register_as_candidate(
            RuntimeOrigin::signed(3)
        ));
        assert_ok!(Session::set_keys(
            RuntimeOrigin::signed(3),
            UintAuthorityId(3).into(),
            vec![]
        ));
        assert_ok!(CollatorSelection::register_as_candidate(
            RuntimeOrigin::signed(4)
        ));
        assert_ok!(Session::set_keys(
            RuntimeOrigin::signed(4),
            UintAuthorityId(4).into(),
            vec![]
        ));
        initialize_to_block(10);
        assert_eq!(CollatorSelection::candidates().len(), 2);
        initialize_to_block(20);
        assert_eq!(SessionChangeBlock::get(), 20);
        assert_eq!(CollatorSelection::candidates().len(), 2);
        assert_eq!(Session::validators().len(), 4); // all candidates active
        initialize_to_block(30);
        // 4 authored all blocks in this the past session, gets to stay 3 was kicked on session change
        assert_eq!(CollatorSelection::candidates().len(), 1);
        // 3 will be kicked after 1 session delay
        assert_eq!(SessionHandlerCollators::get(), vec![1, 2, 3, 4]);
        let collator = CandidateInfo {
            who: 4,
            deposit: 10,
        };
        assert_eq!(CollatorSelection::candidates(), vec![collator]);
        // assert_eq!(CollatorSelection::last_authored_block(4), 20); // NOTE: not used in manta fork
        initialize_to_block(40);
        // 3 gets kicked after 1 session delay
        assert_eq!(SessionHandlerCollators::get(), vec![1, 2, 4]);
        // kicked collator gets funds back
        assert_eq!(Balances::free_balance(3), 100);
    });
}

#[test]
fn manta_kick_algorithm_normal_operation() {
    new_test_ext().execute_with(|| {
        // add collator candidates
        setup_3_candidates();
        assert_eq!(candidate_ids(), vec![CHAD, DAVE, EVE]);

        // 80th percentile = 10, kick *below* 9, remove CHAD,EVE
        BlocksPerCollatorThisSession::<Test>::insert(ALICE, 10);
        BlocksPerCollatorThisSession::<Test>::insert(BOB, 10);
        BlocksPerCollatorThisSession::<Test>::insert(CHAD, 4);
        BlocksPerCollatorThisSession::<Test>::insert(DAVE, 9);
        BlocksPerCollatorThisSession::<Test>::insert(EVE, 0);
        assert_eq!(
            CollatorSelection::evict_bad_collators(CollatorSelection::candidates()),
            vec![EVE, CHAD]
        );

        // readd them
        assert_ok!(CollatorSelection::register_as_candidate(
            RuntimeOrigin::signed(CHAD)
        ));
        assert_ok!(CollatorSelection::register_as_candidate(
            RuntimeOrigin::signed(EVE)
        ));

        // Don't try kicking invulnerables ( ALICE and BOB ), percentile = 9, threshold is 8.1 => kick 8 and below
        BlocksPerCollatorThisSession::<Test>::insert(ALICE, 0);
        BlocksPerCollatorThisSession::<Test>::insert(BOB, 10);
        BlocksPerCollatorThisSession::<Test>::insert(CHAD, 4);
        BlocksPerCollatorThisSession::<Test>::insert(DAVE, 9);
        BlocksPerCollatorThisSession::<Test>::insert(EVE, 0);
        assert_eq!(
            CollatorSelection::evict_bad_collators(CollatorSelection::candidates()),
            vec![EVE, CHAD]
        );
    });
}

#[test]
fn manta_kick_algorithm_boundaries() {
    new_test_ext().execute_with(|| {
        // add collator candidates
        setup_3_candidates();
        assert_eq!(candidate_ids(), vec![CHAD, DAVE, EVE]);

        let empty_vec = Vec::<<Test as frame_system::Config>::AccountId>::new();
        // Kick anyone not at perfect performance
        EvictionBaseline::<Test>::put(Percent::from_percent(100));
        EvictionTolerance::<Test>::put(Percent::from_percent(0));

        BlocksPerCollatorThisSession::<Test>::insert(CHAD, 9);
        BlocksPerCollatorThisSession::<Test>::insert(DAVE, 11);
        BlocksPerCollatorThisSession::<Test>::insert(EVE, 10);
        assert_eq!(
            CollatorSelection::evict_bad_collators(CollatorSelection::candidates()),
            vec![CHAD, EVE]
        );
        assert_ok!(CollatorSelection::register_as_candidate(
            RuntimeOrigin::signed(CHAD)
        ));
        // Allow any underperformance => eviction disabled
        EvictionTolerance::<Test>::put(Percent::from_percent(100));
        assert_eq!(
            CollatorSelection::evict_bad_collators(CollatorSelection::candidates()),
            empty_vec
        );
        // 0-th percentile = use worst collator as benchmark => eviction disabled
        EvictionBaseline::<Test>::put(Percent::from_percent(0));
        EvictionTolerance::<Test>::put(Percent::from_percent(0));
        assert_eq!(
            CollatorSelection::evict_bad_collators(CollatorSelection::candidates()),
            empty_vec
        );
        // Same performance => no kick
        EvictionBaseline::<Test>::put(Percent::from_percent(100));
        EvictionTolerance::<Test>::put(Percent::from_percent(0));
        BlocksPerCollatorThisSession::<Test>::insert(CHAD, 10);
        BlocksPerCollatorThisSession::<Test>::insert(DAVE, 10);
        assert_eq!(
            CollatorSelection::evict_bad_collators(CollatorSelection::candidates()),
            empty_vec
        );
        // Exactly on threshold => no kick
        EvictionBaseline::<Test>::put(Percent::from_percent(100));
        EvictionTolerance::<Test>::put(Percent::from_percent(10));
        BlocksPerCollatorThisSession::<Test>::insert(CHAD, 10);
        BlocksPerCollatorThisSession::<Test>::insert(DAVE, 9);
        assert_eq!(
            CollatorSelection::evict_bad_collators(CollatorSelection::candidates()),
            empty_vec
        );
        // Rational threshold = 8.1, kick 8 and below
        EvictionBaseline::<Test>::put(Percent::from_percent(100));
        EvictionTolerance::<Test>::put(Percent::from_percent(10));
        BlocksPerCollatorThisSession::<Test>::insert(CHAD, 8);
        BlocksPerCollatorThisSession::<Test>::insert(DAVE, 10);
        assert_eq!(
            CollatorSelection::evict_bad_collators(CollatorSelection::candidates()),
            vec![CHAD]
        );
    });
}

#[test]
fn manta_collator_onboarding_sequence() {
    new_test_ext().execute_with(|| {
        // add new collator candidates, they will become validators next session
        // Sessions rotate every 10 blocks, so we kick on each x0-th block
        setup_3_candidates();
        assert_eq!(Session::validators(), vec![ALICE, BOB]);
        assert_eq!(candidate_ids(), vec![CHAD, DAVE, EVE]);

        // RAD: mock.rs specifies DAVE as author of all blocks in find_author, DAVE will produce all 10 blocks in a session
        // RAD: other tests like authorship_event_handler depend on DAVE producing blocks
        initialize_to_block(10);
        assert_eq!(Session::validators(), vec![ALICE, BOB]); // collators ALICE and BOB must not have been kicked, invulnerable
        assert_eq!(candidate_ids(), vec![CHAD, DAVE, EVE]); // collators CHAD,DAVE,EVE are not yet active

        initialize_to_block(20);
        assert_eq!(Session::validators(), vec![ALICE, BOB, CHAD, DAVE, EVE]); // all collators online
    });
}

#[test]
fn manta_dont_kick_invulnerables() {
    new_test_ext().execute_with(|| {
        setup_3_candidates();
        initialize_to_block(20);
        assert_eq!(Session::validators(), vec![ALICE, BOB, CHAD, DAVE, EVE]);

        initialize_to_block(29);
        set_all_validator_perf_to(10); // NOTE: Validator DAVE produces 10 blocks each session in testing
        BlocksPerCollatorThisSession::<Test>::insert(1u64, 0);
        BlocksPerCollatorThisSession::<Test>::insert(2u64, 0);
        assert_eq!(Session::validators(), vec![ALICE, BOB, CHAD, DAVE, EVE]);
        assert_eq!(candidate_ids(), vec![CHAD, DAVE, EVE]);
    });
}

#[test]
fn manta_remove_underperformer_even_if_it_recovers() {
    new_test_ext().execute_with(|| {
        setup_3_candidates();
        initialize_to_block(20);
        assert_eq!(Session::validators(), vec![ALICE, BOB, CHAD, DAVE, EVE]);

        initialize_to_block(29);
        set_all_validator_perf_to(10);
        BlocksPerCollatorThisSession::<Test>::insert(EVE, 5);
        assert_eq!(Session::validators(), vec![ALICE, BOB, CHAD, DAVE, EVE]);
        assert_eq!(candidate_ids(), vec![CHAD, DAVE, EVE]);
        initialize_to_block(39);
        set_all_validator_perf_to(10);
        assert_eq!(Session::validators(), vec![ALICE, BOB, CHAD, DAVE, EVE]);
        assert_eq!(candidate_ids(), vec![CHAD, DAVE]); // EVE got removed from candidates
        initialize_to_block(49);
        set_all_validator_perf_to(10);
        assert_eq!(Session::validators(), vec![ALICE, BOB, CHAD, DAVE]); // and from validators
        assert_eq!(candidate_ids(), vec![CHAD, DAVE]);
    });
}

#[test]
fn manta_remove_underperformer_even_if_it_is_immediately_readded_as_candidate() {
    // TC3: EVE underperforms for one session, is kicked and immediately readded - loses one session then onboards again
    new_test_ext().execute_with(|| {
        setup_3_candidates();
        initialize_to_block(20);
        assert_eq!(Session::validators(), vec![ALICE, BOB, CHAD, DAVE, EVE]);

        initialize_to_block(29);
        set_all_validator_perf_to(10);
        BlocksPerCollatorThisSession::<Test>::insert(EVE, 0);
        initialize_to_block(30);
        assert_eq!(Session::validators(), vec![ALICE, BOB, CHAD, DAVE, EVE]);
        assert_eq!(candidate_ids(), vec![CHAD, DAVE]); // EVE got kicked
        assert_ok!(CollatorSelection::register_as_candidate(
            RuntimeOrigin::signed(EVE)
        )); // and is immediately readded
        assert_eq!(candidate_ids(), vec![CHAD, DAVE, EVE]);
        initialize_to_block(39);
        set_all_validator_perf_to(10);
        initialize_to_block(40);
        assert_eq!(Session::validators(), vec![ALICE, BOB, CHAD, DAVE]); // is removed from validators
        assert_eq!(candidate_ids(), vec![CHAD, DAVE, EVE]); // but not from candiadates
        initialize_to_block(49);
        set_all_validator_perf_to(10);
        initialize_to_block(50);
        assert_eq!(Session::validators(), vec![ALICE, BOB, CHAD, DAVE, EVE]); // and onboards again one session later
        assert_eq!(candidate_ids(), vec![CHAD, DAVE, EVE]);
    })
}

#[test]
fn manta_dont_kick_uniform_underperformance() {
    // TC4: Everybody underperforms (algorithm knows no target number, just relative performance), nobody gets kicked
    new_test_ext().execute_with(|| {
        setup_3_candidates();
        initialize_to_block(20);
        assert_eq!(Session::validators(), vec![ALICE, BOB, CHAD, DAVE, EVE]);

        set_all_validator_perf_to(6);
        initialize_to_block(30);
        assert_eq!(Session::validators(), vec![ALICE, BOB, CHAD, DAVE, EVE]);
        assert_eq!(candidate_ids(), vec![CHAD, DAVE, EVE]);
    })
}

#[test]
fn manta_dont_kick_collator_at_tolerance() {
    // TC5: EVE is on threshold, don't kick ( at 5 nodes, the 80th percentile is the second highest value of the set = 100, 10% threshold is 10 )
    new_test_ext().execute_with(|| {
        setup_3_candidates();
        initialize_to_block(20);
        assert_eq!(Session::validators(), vec![ALICE, BOB, CHAD, DAVE, EVE]);

        initialize_to_block(29);
        set_all_validator_perf_to(100);
        BlocksPerCollatorThisSession::<Test>::insert(EVE, 90);
        initialize_to_block(39);
        set_all_validator_perf_to(100);
        assert_eq!(candidate_ids(), vec![CHAD, DAVE, EVE]);
    })
}
#[test]
#[should_panic = "duplicate invulnerables in genesis."]
fn cannot_set_genesis_value_twice() {
    sp_tracing::try_init_simple();
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    let invulnerables = vec![1, 1];

    let collator_selection = collator_selection::GenesisConfig::<Test> {
        desired_candidates: 2,
        eviction_baseline: Percent::from_percent(80),
        eviction_tolerance: Percent::from_percent(10),
        candidacy_bond: 10,
        invulnerables,
    };
    // collator selection must be initialized before session.
    collator_selection.assimilate_storage(&mut t).unwrap();
}

#[test]
fn register_candidate_should_work() {
    new_test_ext().execute_with(|| {
        // add collator by root should work
        let candidate = 3;
        assert_ok!(CollatorSelection::register_candidate(
            RuntimeOrigin::signed(RootAccount::get()),
            candidate
        ));

        // cannot add the same collator twice
        assert_noop!(
            CollatorSelection::register_candidate(
                RuntimeOrigin::signed(RootAccount::get()),
                candidate
            ),
            Error::<Test>::AlreadyCandidate
        );

        let collator = CandidateInfo {
            who: candidate,
            deposit: 10,
        };
        assert_eq!(CollatorSelection::candidates(), vec![collator]);

        // normal user cannot add collator
        assert_noop!(
            CollatorSelection::register_candidate(RuntimeOrigin::signed(5), 4),
            BadOrigin,
        );

        // Cannot add collator if it reaches desired candidate
        // Now it should be 2 candidates.
        assert_ok!(CollatorSelection::register_candidate(
            RuntimeOrigin::signed(RootAccount::get()),
            4
        ));
        assert_eq!(CollatorSelection::candidates().len(), 2);
        assert_noop!(
            CollatorSelection::register_candidate(RuntimeOrigin::signed(RootAccount::get()), 5),
            Error::<Test>::TooManyCandidates
        );
    });
}

#[test]
fn remove_collator_should_work() {
    new_test_ext().execute_with(|| {
        // add collator by root should work
        let candidate = 3;
        assert_ok!(CollatorSelection::register_candidate(
            RuntimeOrigin::signed(RootAccount::get()),
            candidate
        ));

        // normal user cannot remove specified collator
        assert_noop!(
            CollatorSelection::remove_collator(RuntimeOrigin::signed(5), candidate),
            BadOrigin
        );

        // remove collator should work
        assert_ok!(CollatorSelection::remove_collator(
            RuntimeOrigin::signed(RootAccount::get()),
            candidate
        ));

        // cannot remove a unregistered collator
        assert_noop!(
            CollatorSelection::remove_collator(
                RuntimeOrigin::signed(RootAccount::get()),
                candidate
            ),
            Error::<Test>::NotCandidate
        );

        // Cannot remove invulnerables
        let invulnerable = 2;
        assert_noop!(
            CollatorSelection::remove_collator(
                RuntimeOrigin::signed(RootAccount::get()),
                invulnerable
            ),
            Error::<Test>::NotAllowRemoveInvulnerable
        );
    });
}

#[test]
fn increase_bond_after_register_candidate() {
    // It's a corner case:
    // 1. Set orignal bond as 10KMA.
    // 2. Register candidate.
    // 3. Increase bond to 15KMA.
    // 3. Unregister candidate.
    // 4. The owner should get 10KMA back instead of 15KMA.
    // Increasing bond should not affect previous candidates.
    new_test_ext().execute_with(|| {
        // add candidate_1 by root
        let candidate_1 = 3;

        // reserve some tokens first
        let locked_amount = 50;
        assert_ok!(<Test as crate::Config>::Currency::reserve(
            &candidate_1,
            locked_amount
        ));

        // get free balances after first reserve
        let candidate_1_balances_before_registration = Balances::free_balance(candidate_1);

        assert_ok!(CollatorSelection::register_candidate(
            RuntimeOrigin::signed(RootAccount::get()),
            candidate_1
        ));

        // check candidate_1's reserved balance
        let prev_bond = CollatorSelection::candidacy_bond();
        // candidate_1 should be reserved prev_bond KMA
        assert_eq!(
            prev_bond + locked_amount,
            Balances::reserved_balance(candidate_1)
        );

        // increase bond
        let new_bond = prev_bond + 5;
        assert_ok!(CollatorSelection::set_candidacy_bond(
            RuntimeOrigin::signed(RootAccount::get()),
            new_bond
        ));

        // register new candidate after increase bond
        let candidate_2 = 4;

        let locked_amount_2 = 55;
        assert_ok!(<Test as crate::Config>::Currency::reserve(
            &candidate_2,
            locked_amount_2
        ));

        // get free balances after first reserve
        let candidate_2_balances_before_registration = Balances::free_balance(candidate_2);

        assert_ok!(CollatorSelection::register_candidate(
            RuntimeOrigin::signed(RootAccount::get()),
            candidate_2
        ));
        // check new bond
        assert_eq!(new_bond, CollatorSelection::candidacy_bond());
        // candidate_2 should be reserved new_bond KMA
        assert_eq!(
            new_bond + locked_amount_2,
            Balances::reserved_balance(candidate_2)
        );

        // remove candidate_1
        assert_ok!(CollatorSelection::remove_collator(
            RuntimeOrigin::signed(RootAccount::get()),
            candidate_1
        ));
        // check candidate_1
        assert_eq!(locked_amount, Balances::reserved_balance(candidate_1));
        assert_eq!(
            candidate_1_balances_before_registration,
            Balances::free_balance(candidate_1)
        );

        // remove candidate_2
        assert_ok!(CollatorSelection::remove_collator(
            RuntimeOrigin::signed(RootAccount::get()),
            candidate_2
        ));
        // check candidate_2
        assert_eq!(locked_amount_2, Balances::reserved_balance(candidate_2));
        assert_eq!(
            candidate_2_balances_before_registration,
            Balances::free_balance(candidate_2)
        );
    });
}
