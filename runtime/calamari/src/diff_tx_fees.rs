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

#![cfg(test)]

use crate::{Runtime, TransactionPayment};
use codec::{Decode, Encode};
use frame_support::{
    dispatch::{DispatchInfo, GetDispatchInfo},
    traits::{GetCallMetadata, OriginTrait, StorePreimage},
};
use manta_primitives::assets::{AssetRegistryMetadata, TestingDefault};
use manta_support::manta_pay::TransferPost;
use pallet_transaction_payment::Multiplier;
use runtime_common::MinimumMultiplier;
use sp_runtime::{
    traits::{Saturating, Zero},
    AccountId32, FixedPointNumber, MultiAddress, Perbill, Percent,
};
use std::str::FromStr;
use xcm::prelude::*;

const TX_FEE_FLUCTUATION: Percent = Percent::from_percent(10);
const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
const BOB: AccountId32 = AccountId32::new([2u8; 32]);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TxFeeDetail {
    module: String,
    extrinsic: String,
    tx_fee_with_decimal: String,
    tx_fee_without_decimal: String,
    fee_multiplier: String,
}

fn fee_multipliers() -> [Multiplier; 3] {
    [
        MinimumMultiplier::get(),                       // 0.0002
        Multiplier::saturating_from_rational(1, 5u128), // 0.2
        Multiplier::saturating_from_rational(1, 1u128), // 1.0
    ]
}

fn get_call_details(call: &crate::RuntimeCall) -> (DispatchInfo, u32) {
    let dispatch_info =
        <<Runtime as frame_system::Config>::RuntimeCall as GetDispatchInfo>::get_dispatch_info(
            call,
        );
    let call_len = call.using_encoded(|e| e.len()) as u32;
    (dispatch_info, call_len)
}

#[test]
fn diff_tx_fees() {
    const CURRENT_PATH: &str = env!("CARGO_MANIFEST_DIR");
    let mut latest_version = String::new();
    for file in std::fs::read_dir(format!("{CURRENT_PATH}/tx-fees-data")).unwrap() {
        let _version = file
            .unwrap()
            .path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .split("-tx-fees.csv")
            .collect::<Vec<&str>>()[0]
            .to_string();
        if _version.contains("README.md") {
            continue;
        }
        let _version = version_compare::Version::from(&_version).unwrap();
        let version = version_compare::Version::from(&latest_version).unwrap();
        if version < _version {
            latest_version = _version.to_string();
        }
    }
    let version = version_compare::Version::from(&latest_version).unwrap();
    let csv_path = format!("{CURRENT_PATH}/tx-fees-data/{version}-tx-fees.csv");
    let mut rdr = csv::Reader::from_path(csv_path).unwrap();

    let (calamari_runtime_calls, mut test_runner) = calculate_all_current_extrinsic_tx_fee();

    let mut last_release_tx_fees = rdr.deserialize().map(|e| {
        let record: TxFeeDetail = e.unwrap();
        record
    });

    // check whether there's new pallet introduced
    test_runner.execute_with(|| {
        let opaque_metadata = crate::Runtime::metadata();
        let pallets = match opaque_metadata.1 {
            frame_support::metadata::RuntimeMetadata::V14(metadata) => metadata.pallets,
            _ => unreachable!(),
        };
        assert_eq!(pallets.len(), 39);
    });

    let fee_multipliers = fee_multipliers();
    for multiplier in fee_multipliers {
        test_runner.execute_with(|| {
            pallet_transaction_payment::NextFeeMultiplier::<Runtime>::set(multiplier);
        });
        test_runner.execute_with(|| {
            for (pallet_name, extrinsic_name, dispatch_info, call_len) in calamari_runtime_calls.iter()
            {
                match last_release_tx_fees.find(|e| e.extrinsic.eq(extrinsic_name) && e.fee_multiplier.eq(&multiplier.to_float().to_string())) {
                    Some(found) => {
                        let _tx_fee_with_decimal = TransactionPayment::compute_fee(*call_len, dispatch_info, 0);
                        let tx_fee_with_decimal = Multiplier::from_str(&_tx_fee_with_decimal.to_string()).unwrap();
                        let last_tx_fee = Multiplier::from_str(&found.tx_fee_with_decimal).unwrap();
                        let fluctuation = {
                            let diff_value = tx_fee_with_decimal.saturating_sub(last_tx_fee).div(last_tx_fee);
                            if diff_value < Multiplier::zero() {
                                Percent::from_float(diff_value.neg().to_float())
                            } else {
                                Percent::from_float(diff_value.to_float())
                            }
                        };
                        let _multiplier = found.fee_multiplier;
                        assert!(fluctuation <= TX_FEE_FLUCTUATION, "The tx fee fluctuation for the extrinsic {extrinsic_name} is {fluctuation:?}, bigger than {TX_FEE_FLUCTUATION:?} with multiplier {_multiplier}.");
                    }
                    None => println!("The extrinsic {pallet_name}.{extrinsic_name} is missing from current tx fees list, please add it to latest csv file."),
                }
            }
        });
    }
}

#[test]
#[ignore]
fn generate_all_current_extrinsics_tx_fee_to_csv() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const CURRENT_PATH: &str = env!("CARGO_MANIFEST_DIR");
    let csv_path = format!("{CURRENT_PATH}/tx-fees-data/{VERSION}-tx-fees.csv");

    let mut wtr = csv::Writer::from_path(csv_path).unwrap();
    let (calamari_runtime_calls, mut test_runner) = calculate_all_current_extrinsic_tx_fee();

    let fee_multipliers = fee_multipliers();
    let decimal: Multiplier = Multiplier::from_u32(10).saturating_pow(12);

    let mut all_extrinsics_tx_fees = vec![];
    for multiplier in fee_multipliers {
        // set the multiplier
        test_runner.execute_with(|| {
            pallet_transaction_payment::NextFeeMultiplier::<Runtime>::set(multiplier);
        });
        test_runner.execute_with(|| {
            for (pallet_name, extrinsic_name, dispatch_info, call_len) in
                calamari_runtime_calls.clone()
            {
                let tx_fee_with_decimal =
                    TransactionPayment::compute_fee(call_len, &dispatch_info, 0);
                let tx_fee_without_decimal = Multiplier::try_from(tx_fee_with_decimal)
                    .unwrap()
                    .div(decimal)
                    .to_float();
                let tx_fee = TxFeeDetail {
                    module: pallet_name.to_owned(),
                    extrinsic: extrinsic_name.to_owned(),
                    tx_fee_with_decimal: tx_fee_with_decimal.to_string(),
                    tx_fee_without_decimal: tx_fee_without_decimal.to_string(),
                    fee_multiplier: multiplier.to_float().to_string(),
                };
                all_extrinsics_tx_fees.push(tx_fee);
            }
        });
    }

    for extrinsic in all_extrinsics_tx_fees {
        wtr.serialize(extrinsic).unwrap();
    }
    wtr.flush().unwrap();
}

fn calculate_all_current_extrinsic_tx_fee() -> (
    Vec<(&'static str, &'static str, DispatchInfo, u32)>,
    sp_io::TestExternalities,
) {
    let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap()
        .into();

    let mut calamari_runtime_calls = vec![];
    // frame_system
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("System").len(),
            8,
            "Please update new extrinsic here."
        );
        // remark, which length is 32
        let call = crate::RuntimeCall::System(frame_system::Call::remark {
            remark: vec![1u8; 32],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("frame_system", "remark-length=32", dispatch_info, call_len));

        // remark, which length is 64
        let call = crate::RuntimeCall::System(frame_system::Call::remark {
            remark: vec![1u8; 32],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("frame_system", "remark-length=64", dispatch_info, call_len));

        // set_heap_pages
        let call = crate::RuntimeCall::System(frame_system::Call::set_heap_pages { pages: 64 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("frame_system", "set_heap_pages", dispatch_info, call_len));

        // set_code
        let call = crate::RuntimeCall::System(frame_system::Call::set_code {
            code: vec![1u8; 32],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("frame_system", "set_code", dispatch_info, call_len));

        // set_code_without_checks
        let call = crate::RuntimeCall::System(frame_system::Call::set_code_without_checks {
            code: vec![1u8; 32],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "frame_system",
            "set_code_without_checks",
            dispatch_info,
            call_len,
        ));

        // set_storage
        let call = crate::RuntimeCall::System(frame_system::Call::set_storage {
            items: vec![(vec![1u8; 32], vec![2u8; 32])],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("frame_system", "set_storage", dispatch_info, call_len));

        // kill_storage
        let call = crate::RuntimeCall::System(frame_system::Call::kill_storage {
            keys: vec![vec![1u8; 32], vec![2u8; 32]],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("frame_system", "kill_storage", dispatch_info, call_len));

        // kill_prefix
        let call = crate::RuntimeCall::System(frame_system::Call::kill_prefix {
            prefix: vec![1u8; 32],
            subkeys: 8,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("frame_system", "kill_prefix", dispatch_info, call_len));

        // remark_with_event, which length is 32
        let call = crate::RuntimeCall::System(frame_system::Call::remark_with_event {
            remark: vec![1u8; 32],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "frame_system",
            "remark_with_event-length=32",
            dispatch_info,
            call_len,
        ));

        // remark_with_event, which length is 64
        let call = crate::RuntimeCall::System(frame_system::Call::remark_with_event {
            remark: vec![1u8; 32],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "frame_system",
            "remark_with_event-length=64",
            dispatch_info,
            call_len,
        ));
    }

    // pallet_treasury
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("Treasury").len(),
            5,
            "Please update new extrinsic here."
        );
        // propose_spend
        let call = crate::RuntimeCall::Treasury(pallet_treasury::Call::propose_spend {
            value: 8,
            beneficiary: ALICE.into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_treasury", "propose_spend", dispatch_info, call_len));

        // reject_proposal
        let call =
            crate::RuntimeCall::Treasury(pallet_treasury::Call::reject_proposal { proposal_id: 8 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_treasury",
            "reject_proposal",
            dispatch_info,
            call_len,
        ));

        // approve_proposal
        let call = crate::RuntimeCall::Treasury(pallet_treasury::Call::approve_proposal {
            proposal_id: 8,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_treasury",
            "approve_proposal",
            dispatch_info,
            call_len,
        ));

        // spend
        let call = crate::RuntimeCall::Treasury(pallet_treasury::Call::spend {
            amount: 8,
            beneficiary: ALICE.into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_treasury", "spend", dispatch_info, call_len));

        // remove_approval
        let call =
            crate::RuntimeCall::Treasury(pallet_treasury::Call::remove_approval { proposal_id: 8 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_treasury",
            "remove_approval",
            dispatch_info,
            call_len,
        ));
    }

    // pallet_timestamp
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("Timestamp").len(),
            1,
            "Please update new extrinsic here."
        );
        // set
        let call = crate::RuntimeCall::Timestamp(pallet_timestamp::Call::set {
            now: Default::default(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_timestamp", "set", dispatch_info, call_len));
    }

    // pallet_preimage
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("Preimage").len(),
            4,
            "Please update new extrinsic here."
        );
        // note_preimage, preimage length is 32
        let call = crate::RuntimeCall::Preimage(pallet_preimage::Call::note_preimage {
            bytes: vec![1u8; 32],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_preimage",
            "note_preimage-length=32",
            dispatch_info,
            call_len,
        ));

        // note_preimage, preimage length is 64
        let call = crate::RuntimeCall::Preimage(pallet_preimage::Call::note_preimage {
            bytes: vec![1u8; 64],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_preimage",
            "note_preimage-length=32",
            dispatch_info,
            call_len,
        ));

        // unnote_preimage
        let call = crate::RuntimeCall::Preimage(pallet_preimage::Call::unnote_preimage {
            hash: Default::default(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_preimage",
            "unnote_preimage",
            dispatch_info,
            call_len,
        ));

        // request_preimage
        let call = crate::RuntimeCall::Preimage(pallet_preimage::Call::request_preimage {
            hash: Default::default(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_preimage",
            "request_preimage",
            dispatch_info,
            call_len,
        ));

        // unrequest_preimage
        let call = crate::RuntimeCall::Preimage(pallet_preimage::Call::unrequest_preimage {
            hash: Default::default(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_preimage",
            "unrequest_preimage",
            dispatch_info,
            call_len,
        ));
    }

    // pallet_multisig
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("Multisig").len(),
            4,
            "Please update new extrinsic here."
        );
        // as_multi_threshold_1
        let dummy_call = crate::RuntimeCall::Preimage(pallet_preimage::Call::unrequest_preimage {
            hash: Default::default(),
        });
        let call = crate::RuntimeCall::Multisig(pallet_multisig::Call::as_multi_threshold_1 {
            other_signatories: vec![ALICE],
            call: Box::new(dummy_call.clone()),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_multisig",
            "as_multi_threshold_1",
            dispatch_info,
            call_len,
        ));

        // as_multi
        let call = crate::RuntimeCall::Multisig(pallet_multisig::Call::as_multi {
            threshold: 2,
            other_signatories: vec![ALICE],
            maybe_timepoint: None,
            call: Box::new(dummy_call),
            max_weight: Default::default(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_multisig", "as_multi", dispatch_info, call_len));

        // approve_as_multi
        let call = crate::RuntimeCall::Multisig(pallet_multisig::Call::approve_as_multi {
            threshold: 2,
            other_signatories: vec![ALICE],
            maybe_timepoint: None,
            call_hash: [1u8; 32],
            max_weight: Default::default(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_multisig",
            "approve_as_multi",
            dispatch_info,
            call_len,
        ));

        // cancel_as_multi
        let call = crate::RuntimeCall::Multisig(pallet_multisig::Call::cancel_as_multi {
            threshold: 2,
            other_signatories: vec![ALICE],
            timepoint: Default::default(),
            call_hash: [1u8; 32],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_multisig",
            "cancel_as_multi",
            dispatch_info,
            call_len,
        ));
    }

    // pallet_membership
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("CouncilMembership").len(),
            7,
            "Please update new extrinsic here."
        );
        // add_member
        let call = crate::RuntimeCall::CouncilMembership(pallet_membership::Call::add_member {
            who: ALICE.into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_membership", "add_member", dispatch_info, call_len));

        // remove_member
        let call = crate::RuntimeCall::CouncilMembership(pallet_membership::Call::remove_member {
            who: ALICE.into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_membership",
            "remove_member",
            dispatch_info,
            call_len,
        ));

        // swap_member
        let call = crate::RuntimeCall::CouncilMembership(pallet_membership::Call::swap_member {
            remove: ALICE.into(),
            add: BOB.into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_membership", "swap_member", dispatch_info, call_len));

        // reset_members
        let call = crate::RuntimeCall::CouncilMembership(pallet_membership::Call::reset_members {
            members: vec![ALICE, BOB],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_membership",
            "reset_members",
            dispatch_info,
            call_len,
        ));

        // change_key
        let call = crate::RuntimeCall::CouncilMembership(pallet_membership::Call::change_key {
            new: ALICE.into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_membership", "change_key", dispatch_info, call_len));

        // set_prime
        let call = crate::RuntimeCall::CouncilMembership(pallet_membership::Call::set_prime {
            who: ALICE.into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_membership", "set_prime", dispatch_info, call_len));

        // clear_prime
        let call = crate::RuntimeCall::CouncilMembership(pallet_membership::Call::clear_prime {});
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_membership", "clear_prime", dispatch_info, call_len));
    }

    // pallet_democracy
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("Democracy").len(),
            18,
            "Please update new extrinsic here."
        );
        // propose
        let dummy_call = crate::RuntimeCall::Democracy(pallet_democracy::Call::cancel_proposal {
            prop_index: 2,
        });
        let call = crate::RuntimeCall::Democracy(pallet_democracy::Call::propose {
            proposal: crate::Preimage::bound(dummy_call.clone()).unwrap(),
            value: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_democracy", "propose", dispatch_info, call_len));

        // second
        let call = crate::RuntimeCall::Democracy(pallet_democracy::Call::second { proposal: 3 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_democracy", "second", dispatch_info, call_len));

        // vote
        let call = crate::RuntimeCall::Democracy(pallet_democracy::Call::vote {
            ref_index: 3,
            vote: pallet_democracy::AccountVote::Standard {
                vote: Default::default(),
                balance: 2,
            },
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_democracy", "vote", dispatch_info, call_len));

        // emergency_cancel
        let call = crate::RuntimeCall::Democracy(pallet_democracy::Call::emergency_cancel {
            ref_index: 3,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_democracy",
            "emergency_cancel",
            dispatch_info,
            call_len,
        ));

        // external_propose
        let call = crate::RuntimeCall::Democracy(pallet_democracy::Call::external_propose {
            proposal: crate::Preimage::bound(dummy_call.clone()).unwrap(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_democracy",
            "external_propose",
            dispatch_info,
            call_len,
        ));

        // external_propose_majority
        let call =
            crate::RuntimeCall::Democracy(pallet_democracy::Call::external_propose_majority {
                proposal: crate::Preimage::bound(dummy_call.clone()).unwrap(),
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_democracy",
            "external_propose_majority",
            dispatch_info,
            call_len,
        ));

        // external_propose_default
        let call =
            crate::RuntimeCall::Democracy(pallet_democracy::Call::external_propose_default {
                proposal: crate::Preimage::bound(dummy_call).unwrap(),
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_democracy",
            "external_propose_default",
            dispatch_info,
            call_len,
        ));

        // fast_track
        let call = crate::RuntimeCall::Democracy(pallet_democracy::Call::fast_track {
            proposal_hash: Default::default(),
            voting_period: 300,
            delay: 30,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_democracy", "fast_track", dispatch_info, call_len));

        // veto_external
        let call = crate::RuntimeCall::Democracy(pallet_democracy::Call::veto_external {
            proposal_hash: Default::default(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_democracy", "veto_external", dispatch_info, call_len));

        // cancel_referendum
        let call = crate::RuntimeCall::Democracy(pallet_democracy::Call::cancel_referendum {
            ref_index: 3,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_democracy",
            "cancel_referendum",
            dispatch_info,
            call_len,
        ));

        // delegate
        let call = crate::RuntimeCall::Democracy(pallet_democracy::Call::delegate {
            to: ALICE.into(),
            conviction: pallet_democracy::Conviction::Locked3x,
            balance: 3,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_democracy", "delegate", dispatch_info, call_len));

        // undelegate
        let call = crate::RuntimeCall::Democracy(pallet_democracy::Call::undelegate {});
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_democracy", "undelegate", dispatch_info, call_len));

        // clear_public_proposals
        let call = crate::RuntimeCall::Democracy(pallet_democracy::Call::clear_public_proposals {});
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_democracy",
            "clear_public_proposals",
            dispatch_info,
            call_len,
        ));

        // unlock
        let call = crate::RuntimeCall::Democracy(pallet_democracy::Call::unlock {
            target: MultiAddress::Id(ALICE),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_democracy", "unlock", dispatch_info, call_len));

        // remove_vote
        let call = crate::RuntimeCall::Democracy(pallet_democracy::Call::remove_vote { index: 2 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_democracy", "remove_vote", dispatch_info, call_len));

        // remove_other_vote
        let call = crate::RuntimeCall::Democracy(pallet_democracy::Call::remove_other_vote {
            target: ALICE.into(),
            index: 2,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_democracy",
            "remove_other_vote",
            dispatch_info,
            call_len,
        ));

        // blacklist
        let call = crate::RuntimeCall::Democracy(pallet_democracy::Call::blacklist {
            proposal_hash: Default::default(),
            maybe_ref_index: Some(2),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_democracy", "blacklist", dispatch_info, call_len));

        // cancel_proposal
        let call = crate::RuntimeCall::Democracy(pallet_democracy::Call::cancel_proposal {
            prop_index: 2,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_democracy",
            "cancel_proposal",
            dispatch_info,
            call_len,
        ));
    }

    // pallet_collective
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("TechnicalCommittee").len(),
            7,
            "Please update new extrinsic here."
        );
        // set_members
        let call = crate::RuntimeCall::TechnicalCommittee(pallet_collective::Call::set_members {
            new_members: vec![ALICE, BOB],
            prime: Some(ALICE),
            old_count: 6,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_collective", "set_members", dispatch_info, call_len));

        // execute
        let dummy_call = crate::RuntimeCall::Democracy(pallet_democracy::Call::cancel_proposal {
            prop_index: 2,
        });
        let call = crate::RuntimeCall::TechnicalCommittee(pallet_collective::Call::execute {
            proposal: Box::new(dummy_call.clone()),
            length_bound: 6,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_collective", "execute", dispatch_info, call_len));

        // propose
        let call = crate::RuntimeCall::TechnicalCommittee(pallet_collective::Call::propose {
            threshold: 3,
            proposal: Box::new(dummy_call),
            length_bound: 6,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_collective", "propose", dispatch_info, call_len));

        // vote
        let call = crate::RuntimeCall::TechnicalCommittee(pallet_collective::Call::vote {
            proposal: Default::default(),
            index: 2,
            approve: true,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_collective", "vote", dispatch_info, call_len));

        // close
        let call = crate::RuntimeCall::TechnicalCommittee(pallet_collective::Call::close {
            proposal_hash: Default::default(),
            index: 2,
            proposal_weight_bound: Default::default(),
            length_bound: 6,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_collective", "close", dispatch_info, call_len));

        // disapprove_proposal
        let call =
            crate::RuntimeCall::TechnicalCommittee(pallet_collective::Call::disapprove_proposal {
                proposal_hash: Default::default(),
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_collective",
            "disapprove_proposal",
            dispatch_info,
            call_len,
        ));
    }

    // cumulus_pallet_xcmp_queue
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("XcmpQueue").len(),
            9,
            "Please update new extrinsic here."
        );
        // service_overweight
        let call =
            crate::RuntimeCall::XcmpQueue(cumulus_pallet_xcmp_queue::Call::service_overweight {
                index: 1,
                weight_limit: 64,
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "cumulus_pallet_xcmp_queue",
            "service_overweight",
            dispatch_info,
            call_len,
        ));

        // suspend_xcm_execution
        let call = crate::RuntimeCall::XcmpQueue(
            cumulus_pallet_xcmp_queue::Call::suspend_xcm_execution {},
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "cumulus_pallet_xcmp_queue",
            "suspend_xcm_execution",
            dispatch_info,
            call_len,
        ));

        // resume_xcm_execution
        let call =
            crate::RuntimeCall::XcmpQueue(cumulus_pallet_xcmp_queue::Call::resume_xcm_execution {});
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "cumulus_pallet_xcmp_queue",
            "resume_xcm_execution",
            dispatch_info,
            call_len,
        ));

        // update_suspend_threshold
        let call = crate::RuntimeCall::XcmpQueue(
            cumulus_pallet_xcmp_queue::Call::update_suspend_threshold { new: 2 },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "cumulus_pallet_xcmp_queue",
            "update_suspend_threshold",
            dispatch_info,
            call_len,
        ));

        // update_drop_threshold
        let call = crate::RuntimeCall::XcmpQueue(
            cumulus_pallet_xcmp_queue::Call::update_suspend_threshold { new: 2 },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "cumulus_pallet_xcmp_queue",
            "update_drop_threshold",
            dispatch_info,
            call_len,
        ));

        // update_resume_threshold
        let call = crate::RuntimeCall::XcmpQueue(
            cumulus_pallet_xcmp_queue::Call::update_resume_threshold { new: 2 },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "cumulus_pallet_xcmp_queue",
            "update_resume_threshold",
            dispatch_info,
            call_len,
        ));

        // update_threshold_weight
        let call = crate::RuntimeCall::XcmpQueue(
            cumulus_pallet_xcmp_queue::Call::update_threshold_weight { new: 64 },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "cumulus_pallet_xcmp_queue",
            "update_threshold_weight",
            dispatch_info,
            call_len,
        ));

        // update_weight_restrict_decay
        let call = crate::RuntimeCall::XcmpQueue(
            cumulus_pallet_xcmp_queue::Call::update_weight_restrict_decay { new: 64 },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "cumulus_pallet_xcmp_queue",
            "update_weight_restrict_decay",
            dispatch_info,
            call_len,
        ));

        // update_xcmp_max_individual_weight
        let call = crate::RuntimeCall::XcmpQueue(
            cumulus_pallet_xcmp_queue::Call::update_xcmp_max_individual_weight { new: 64 },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "cumulus_pallet_xcmp_queue",
            "update_xcmp_max_individual_weight",
            dispatch_info,
            call_len,
        ));
    }

    // orml_xtokens
    // cannot run this part out side of mock runtime.
    t.execute_with(|| {
        assert_eq!(
            crate::RuntimeCall::get_call_names("XTokens").len(),
            6,
            "Please update new extrinsic here."
        );
        // transfer
        let dest = VersionedMultiLocation::V1(Default::default());
        let call = crate::RuntimeCall::XTokens(orml_xtokens::Call::transfer {
            currency_id: crate::xcm_config::CurrencyId::MantaCurrency(1),
            amount: 10,
            dest: Box::new(dest.clone()),
            dest_weight_limit: WeightLimit::Unlimited,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("orml_xtokens", "transfer", dispatch_info, call_len));

        // transfer_multiasset
        let _asset = MultiAsset {
            id: Concrete(MultiLocation {
                parents: 1,
                interior: X1(Parachain(1)),
            }),
            fun: Fungible(10000000000000),
        };
        let asset = xcm::VersionedMultiAsset::V1(_asset.clone());
        let call = crate::RuntimeCall::XTokens(orml_xtokens::Call::transfer_multiasset {
            asset: Box::new(asset.clone()),
            dest: Box::new(dest.clone()),
            dest_weight_limit: WeightLimit::Unlimited,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "orml_xtokens",
            "transfer_multiasset",
            dispatch_info,
            call_len,
        ));

        // transfer_with_fee
        let call = crate::RuntimeCall::XTokens(orml_xtokens::Call::transfer_with_fee {
            currency_id: crate::xcm_config::CurrencyId::MantaCurrency(1),
            amount: 10,
            fee: 20,
            dest: Box::new(dest.clone()),
            dest_weight_limit: WeightLimit::Unlimited,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("orml_xtokens", "transfer_with_fee", dispatch_info, call_len));

        // transfer_multiasset_with_fee
        let _fee_asset = MultiAsset {
            id: Concrete(MultiLocation {
                parents: 1,
                interior: X1(Parachain(1)),
            }),
            fun: Fungible(5000000000000),
        };
        let fee_asset = xcm::VersionedMultiAsset::V1(_fee_asset.clone());
        let call = crate::RuntimeCall::XTokens(orml_xtokens::Call::transfer_multiasset_with_fee {
            asset: Box::new(asset),
            fee: Box::new(fee_asset),
            dest: Box::new(dest.clone()),
            dest_weight_limit: WeightLimit::Unlimited,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "orml_xtokens",
            "transfer_multiasset_with_fee",
            dispatch_info,
            call_len,
        ));

        // transfer_multicurrencies
        let call = crate::RuntimeCall::XTokens(orml_xtokens::Call::transfer_multicurrencies {
            currencies: vec![
                (crate::xcm_config::CurrencyId::MantaCurrency(1), 10),
                (crate::xcm_config::CurrencyId::MantaCurrency(2), 20),
            ],
            fee_item: 1,
            dest: Box::new(dest.clone()),
            dest_weight_limit: WeightLimit::Unlimited,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "orml_xtokens",
            "transfer_multicurrencies",
            dispatch_info,
            call_len,
        ));

        // transfer_multiassets
        let assets = xcm::VersionedMultiAssets::V1(MultiAssets::from(vec![_asset, _fee_asset]));
        let call = crate::RuntimeCall::XTokens(orml_xtokens::Call::transfer_multiassets {
            assets: Box::new(assets),
            fee_item: 1,
            dest: Box::new(dest),
            dest_weight_limit: WeightLimit::Unlimited,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "orml_xtokens",
            "transfer_multiassets",
            dispatch_info,
            call_len,
        ));
    });

    // calamari_vesting
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("CalamariVesting").len(),
            3,
            "Please update new extrinsic here."
        );
        // vest
        let call = crate::RuntimeCall::CalamariVesting(calamari_vesting::Call::vest {});
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("calamari_vesting", "vest", dispatch_info, call_len));
    }

    // manta_collator_selection
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("CollatorSelection").len(),
            9,
            "Please update new extrinsic here."
        );
        // set_invulnerables
        let call = crate::RuntimeCall::CollatorSelection(
            manta_collator_selection::Call::set_invulnerables { new: vec![ALICE] },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "manta_collator_selection",
            "set_invulnerables",
            dispatch_info,
            call_len,
        ));

        // set_desired_candidates
        let call = crate::RuntimeCall::CollatorSelection(
            manta_collator_selection::Call::set_desired_candidates { max: 1 },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "manta_collator_selection",
            "set_desired_candidates",
            dispatch_info,
            call_len,
        ));

        // set_candidacy_bond
        let call = crate::RuntimeCall::CollatorSelection(
            manta_collator_selection::Call::set_candidacy_bond { bond: 1 },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "manta_collator_selection",
            "set_candidacy_bond",
            dispatch_info,
            call_len,
        ));

        // register_as_candidate
        let call = crate::RuntimeCall::CollatorSelection(
            manta_collator_selection::Call::register_as_candidate {},
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "manta_collator_selection",
            "register_as_candidate",
            dispatch_info,
            call_len,
        ));

        // register_candidate
        let call = crate::RuntimeCall::CollatorSelection(
            manta_collator_selection::Call::register_candidate {
                new_candidate: ALICE,
            },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "manta_collator_selection",
            "register_candidate",
            dispatch_info,
            call_len,
        ));

        // leave_intent
        let call =
            crate::RuntimeCall::CollatorSelection(manta_collator_selection::Call::leave_intent {});
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "manta_collator_selection",
            "leave_intent",
            dispatch_info,
            call_len,
        ));

        // remove_collator
        let call = crate::RuntimeCall::CollatorSelection(
            manta_collator_selection::Call::remove_collator { collator: ALICE },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "manta_collator_selection",
            "remove_collator",
            dispatch_info,
            call_len,
        ));

        // set_eviction_baseline
        let call = crate::RuntimeCall::CollatorSelection(
            manta_collator_selection::Call::set_eviction_baseline {
                percentile: Default::default(),
            },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "manta_collator_selection",
            "set_eviction_baseline",
            dispatch_info,
            call_len,
        ));

        // set_eviction_tolerance
        let call = crate::RuntimeCall::CollatorSelection(
            manta_collator_selection::Call::set_eviction_tolerance {
                percentage: Default::default(),
            },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "manta_collator_selection",
            "set_eviction_tolerance",
            dispatch_info,
            call_len,
        ));
    }

    // pallet_asset_manager
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("AssetManager").len(),
            9,
            "Please update new extrinsic here."
        );
        // register_asset
        let call = crate::RuntimeCall::AssetManager(pallet_asset_manager::Call::register_asset {
            location: Default::default(),
            metadata: AssetRegistryMetadata::testing_default(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_asset_manager",
            "register_asset",
            dispatch_info,
            call_len,
        ));

        // update_asset_location
        let call =
            crate::RuntimeCall::AssetManager(pallet_asset_manager::Call::update_asset_location {
                asset_id: 1,
                location: Default::default(),
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_asset_manager",
            "update_asset_location",
            dispatch_info,
            call_len,
        ));

        // update_asset_metadata
        let call =
            crate::RuntimeCall::AssetManager(pallet_asset_manager::Call::update_asset_metadata {
                asset_id: 1,
                metadata: AssetRegistryMetadata::testing_default(),
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_asset_manager",
            "update_asset_metadata",
            dispatch_info,
            call_len,
        ));

        // set_units_per_second
        let call =
            crate::RuntimeCall::AssetManager(pallet_asset_manager::Call::set_units_per_second {
                asset_id: 1,
                units_per_second: 1,
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_asset_manager",
            "set_units_per_second",
            dispatch_info,
            call_len,
        ));

        // mint_asset
        let call = crate::RuntimeCall::AssetManager(pallet_asset_manager::Call::mint_asset {
            asset_id: 1,
            beneficiary: ALICE,
            amount: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_asset_manager",
            "mint_asset",
            dispatch_info,
            call_len,
        ));

        // set_min_xcm_fee
        let call = crate::RuntimeCall::AssetManager(pallet_asset_manager::Call::set_min_xcm_fee {
            reserve_chain: Default::default(),
            min_xcm_fee: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_asset_manager",
            "set_min_xcm_fee",
            dispatch_info,
            call_len,
        ));

        // update_outgoing_filtered_assets
        let call = crate::RuntimeCall::AssetManager(
            pallet_asset_manager::Call::update_outgoing_filtered_assets {
                filtered_location: Default::default(),
                should_add: true,
            },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_asset_manager",
            "update_outgoing_filtered_assets",
            dispatch_info,
            call_len,
        ));

        // register_lp_asset
        let call =
            crate::RuntimeCall::AssetManager(pallet_asset_manager::Call::register_lp_asset {
                asset_0: 1,
                asset_1: 2,
                metadata: AssetRegistryMetadata::testing_default(),
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_asset_manager",
            "register_lp_asset",
            dispatch_info,
            call_len,
        ));

        // permissionless_register_asset
        let call = crate::RuntimeCall::AssetManager(
            pallet_asset_manager::Call::permissionless_register_asset {
                name: vec![].try_into().unwrap(),
                symbol: vec![].try_into().unwrap(),
                decimals: 12,
                total_supply: 1_000_000_000_000_000,
            },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_asset_manager",
            "permissionless_register_asset",
            dispatch_info,
            call_len,
        ));
    }

    // pallet_assets
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("Assets").len(),
            28,
            "Please update new extrinsic here."
        );
        // create
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::create {
            id: 1,
            admin: ALICE.into(),
            min_balance: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "create", dispatch_info, call_len));

        // force_create
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::force_create {
            id: 1,
            owner: ALICE.into(),
            is_sufficient: true,
            min_balance: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "force_create", dispatch_info, call_len));

        // start_destroy
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::start_destroy { id: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "start_destroy", dispatch_info, call_len));

        // destroy_accounts
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::destroy_accounts { id: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "destroy_accounts", dispatch_info, call_len));

        // destroy_approvals
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::destroy_approvals { id: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_assets",
            "destroy_approvals",
            dispatch_info,
            call_len,
        ));

        // finish_destroy
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::finish_destroy { id: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "finish_destroy", dispatch_info, call_len));

        // mint
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::mint {
            id: 1,
            beneficiary: ALICE.into(),
            amount: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "mint", dispatch_info, call_len));

        // burn
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::burn {
            id: 1,
            who: ALICE.into(),
            amount: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "burn", dispatch_info, call_len));

        // transfer
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::transfer {
            id: 1,
            target: ALICE.into(),
            amount: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "transfer", dispatch_info, call_len));

        // transfer_keep_alive
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::transfer_keep_alive {
            id: 1,
            target: ALICE.into(),
            amount: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_assets",
            "transfer_keep_alive",
            dispatch_info,
            call_len,
        ));

        // force_transfer
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::force_transfer {
            id: 1,
            source: ALICE.into(),
            dest: ALICE.into(),
            amount: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "force_transfer", dispatch_info, call_len));

        // freeze
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::freeze {
            id: 1,
            who: ALICE.into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "freeze", dispatch_info, call_len));

        // thaw
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::thaw {
            id: 1,
            who: ALICE.into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "thaw", dispatch_info, call_len));

        // freeze_asset
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::freeze_asset { id: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "freeze_asset", dispatch_info, call_len));

        // thaw_asset
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::thaw_asset { id: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "thaw_asset", dispatch_info, call_len));

        // transfer_ownership
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::transfer_ownership {
            id: 1,
            owner: ALICE.into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_assets",
            "transfer_ownership",
            dispatch_info,
            call_len,
        ));

        // set_team
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::set_team {
            id: 1,
            issuer: ALICE.into(),
            admin: ALICE.into(),
            freezer: ALICE.into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "set_team", dispatch_info, call_len));

        // set_metadata
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::set_metadata {
            id: 1,
            name: vec![1u8; 32],
            symbol: vec![1u8; 32],
            decimals: 12,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "set_metadata", dispatch_info, call_len));

        // clear_metadata
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::clear_metadata { id: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "clear_metadata", dispatch_info, call_len));

        // force_set_metadata
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::force_set_metadata {
            id: 1,
            name: vec![1u8; 32],
            symbol: vec![1u8; 32],
            decimals: 12,
            is_frozen: true,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_assets",
            "force_set_metadata",
            dispatch_info,
            call_len,
        ));

        // force_clear_metadata
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::force_clear_metadata { id: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_assets",
            "force_clear_metadata",
            dispatch_info,
            call_len,
        ));

        // force_asset_status
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::force_asset_status {
            id: 1,
            owner: ALICE.into(),
            issuer: ALICE.into(),
            admin: ALICE.into(),
            freezer: ALICE.into(),
            min_balance: 1,
            is_sufficient: true,
            is_frozen: false,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_assets",
            "force_asset_status",
            dispatch_info,
            call_len,
        ));

        // approve_transfer
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::approve_transfer {
            id: 1,
            delegate: ALICE.into(),
            amount: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "approve_transfer", dispatch_info, call_len));

        // cancel_approval
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::cancel_approval {
            id: 1,
            delegate: ALICE.into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "cancel_approval", dispatch_info, call_len));

        // force_cancel_approval
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::force_cancel_approval {
            id: 1,
            owner: ALICE.into(),
            delegate: ALICE.into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_assets",
            "force_cancel_approval",
            dispatch_info,
            call_len,
        ));

        // transfer_approved
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::transfer_approved {
            id: 1,
            owner: ALICE.into(),
            destination: ALICE.into(),
            amount: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_assets",
            "transfer_approved",
            dispatch_info,
            call_len,
        ));

        // touch
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::touch { id: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "touch", dispatch_info, call_len));

        // refund
        let call = crate::RuntimeCall::Assets(pallet_assets::Call::refund {
            id: 1,
            allow_burn: true,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "refund", dispatch_info, call_len));
    }

    // pallet_author_inherent
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("AuthorInherent").len(),
            1,
            "Please update new extrinsic here."
        );
        // kick_off_authorship_validation
        let call = crate::RuntimeCall::AuthorInherent(
            pallet_author_inherent::Call::kick_off_authorship_validation {},
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_author_inherent",
            "kick_off_authorship_validation",
            dispatch_info,
            call_len,
        ));
    }

    // pallet_balances
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("Balances").len(),
            6,
            "Please update new extrinsic here."
        );
        // transfer, 1 token
        let call = crate::RuntimeCall::Balances(pallet_balances::Call::transfer {
            dest: ALICE.into(),
            value: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_balances", "transfer-1", dispatch_info, call_len));

        // transfer, 1M tokens
        let call = crate::RuntimeCall::Balances(pallet_balances::Call::transfer {
            dest: ALICE.into(),
            value: 1_000_000,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_balances",
            "transfer-1000_000",
            dispatch_info,
            call_len,
        ));

        // set_balance
        let call = crate::RuntimeCall::Balances(pallet_balances::Call::set_balance {
            who: ALICE.into(),
            new_free: 1,
            new_reserved: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_balances", "set_balance", dispatch_info, call_len));

        // force_transfer
        let call = crate::RuntimeCall::Balances(pallet_balances::Call::force_transfer {
            source: ALICE.into(),
            dest: ALICE.into(),
            value: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_balances", "force_transfer", dispatch_info, call_len));

        // transfer_keep_alive, 1 token
        let call = crate::RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive {
            dest: ALICE.into(),
            value: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_balances",
            "transfer_keep_alive-1",
            dispatch_info,
            call_len,
        ));

        // transfer_keep_alive, 1M tokens
        let call = crate::RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive {
            dest: ALICE.into(),
            value: 1_000_000,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_balances",
            "transfer_keep_alive-1000_000",
            dispatch_info,
            call_len,
        ));

        // transfer_all
        let call = crate::RuntimeCall::Balances(pallet_balances::Call::transfer_all {
            dest: ALICE.into(),
            keep_alive: false,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_balances", "transfer_all", dispatch_info, call_len));

        // force_unreserve
        let call = crate::RuntimeCall::Balances(pallet_balances::Call::force_unreserve {
            who: ALICE.into(),
            amount: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_balances",
            "force_unreserve",
            dispatch_info,
            call_len,
        ));
    }

    // pallet_manta_pay
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("MantaPay").len(),
            4,
            "Please update new extrinsic here."
        );

        let to_private_proof = [0u8; 552];
        let private_transfer_proof = [0u8; 1290];
        let to_public_proof = [0u8; 1000];
        // to_private
        let to_private_post = TransferPost::decode(&mut to_private_proof.as_slice()).unwrap();
        let call = crate::RuntimeCall::MantaPay(pallet_manta_pay::Call::to_private {
            post: to_private_post,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_manta_pay", "to_private", dispatch_info, call_len));

        // private_transfer
        let private_transfer_post =
            TransferPost::decode(&mut private_transfer_proof.as_slice()).unwrap();
        let call = crate::RuntimeCall::MantaPay(pallet_manta_pay::Call::private_transfer {
            post: private_transfer_post,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_manta_pay",
            "private_transfer",
            dispatch_info,
            call_len,
        ));

        // to_public
        let to_public_post = TransferPost::decode(&mut to_public_proof.as_slice()).unwrap();
        let call = crate::RuntimeCall::MantaPay(pallet_manta_pay::Call::to_public {
            post: to_public_post,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_manta_pay", "to_public", dispatch_info, call_len));

        // public_transfer
        let call = crate::RuntimeCall::MantaPay(pallet_manta_pay::Call::public_transfer {
            asset: Default::default(),
            sink: ALICE,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_manta_pay",
            "public_transfer",
            dispatch_info,
            call_len,
        ));
    }

    // pallet_manta_sbt
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("MantaSbt").len(),
            13,
            "Please update new extrinsic here."
        );
        let to_private_proof = [0u8; 552];
        // to_private
        let to_private_post =
            Box::new(TransferPost::decode(&mut to_private_proof.as_slice()).unwrap());
        let call = crate::RuntimeCall::MantaSbt(pallet_manta_sbt::Call::to_private {
            mint_id: None,
            chain_id: None,
            signature: None,
            post: to_private_post,
            metadata: Default::default(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_manta_sbt", "to_private", dispatch_info, call_len));

        // force_to_private
        let to_private_post =
            Box::new(TransferPost::decode(&mut to_private_proof.as_slice()).unwrap());
        let call = crate::RuntimeCall::MantaSbt(pallet_manta_sbt::Call::force_to_private {
            post: to_private_post.clone(),
            mint_id: 1u32,
            metadata: Default::default(),
            minting_account: ALICE.clone(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_manta_sbt",
            "force_to_private",
            dispatch_info,
            call_len,
        ));

        // reserve_sbt
        let call = crate::RuntimeCall::MantaSbt(pallet_manta_sbt::Call::reserve_sbt {
            reservee: Default::default(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_manta_sbt", "reserve_sbt", dispatch_info, call_len));

        // allowlist_evm_account
        let call = crate::RuntimeCall::MantaSbt(pallet_manta_sbt::Call::allowlist_evm_account {
            mint_id: 1,
            evm_address: Default::default(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_manta_sbt",
            "allowlist_evm_account",
            dispatch_info,
            call_len,
        ));

        // set_next_sbt_id
        let call = crate::RuntimeCall::MantaSbt(pallet_manta_sbt::Call::set_next_sbt_id {
            asset_id: Some(1),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_manta_sbt",
            "set_next_sbt_id",
            dispatch_info,
            call_len,
        ));

        // remove_allowlist_evm_account
        let call =
            crate::RuntimeCall::MantaSbt(pallet_manta_sbt::Call::remove_allowlist_evm_account {
                mint_id: 1,
                evm_address: Default::default(),
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_manta_sbt",
            "remove_allowlist_evm_account",
            dispatch_info,
            call_len,
        ));

        // change_free_reserve_account
        let call =
            crate::RuntimeCall::MantaSbt(pallet_manta_sbt::Call::change_free_reserve_account {
                account: Some(ALICE.clone()),
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_manta_sbt",
            "change_free_reserve_account",
            dispatch_info,
            call_len,
        ));

        // force_mint_sbt_eth
        let call = crate::RuntimeCall::MantaSbt(pallet_manta_sbt::Call::force_mint_sbt_eth {
            post: to_private_post.clone(),
            mint_id: 1u32,
            address: Default::default(),
            collection_id: Some(128),
            item_id: Some(64),
            metadata: Default::default(),
            minting_account: ALICE.clone(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_manta_sbt",
            "force_mint_sbt_eth",
            dispatch_info,
            call_len,
        ));

        // change_force_account
        let call = crate::RuntimeCall::MantaSbt(pallet_manta_sbt::Call::change_force_account {
            account: Some(ALICE.clone()),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_manta_sbt",
            "change_force_account",
            dispatch_info,
            call_len,
        ));

        // mint_sbt_eth
        let call = crate::RuntimeCall::MantaSbt(pallet_manta_sbt::Call::mint_sbt_eth {
            post: to_private_post,
            chain_id: 1,
            eth_signature: [1u8; 65],
            mint_id: 1,
            collection_id: Some(1),
            item_id: Some(1),
            metadata: None,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_manta_sbt", "mint_sbt_eth", dispatch_info, call_len));

        // change_allowlist_account
        let call = crate::RuntimeCall::MantaSbt(pallet_manta_sbt::Call::change_allowlist_account {
            account: Some(ALICE),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_manta_sbt",
            "change_allowlist_account",
            dispatch_info,
            call_len,
        ));

        // update_mint_info
        let call = crate::RuntimeCall::MantaSbt(pallet_manta_sbt::Call::update_mint_info {
            mint_id: 1,
            start_time: Default::default(),
            end_time: None,
            mint_name: Default::default(),
            public: true,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_manta_sbt",
            "update_mint_info",
            dispatch_info,
            call_len,
        ));

        // new_mint_info
        let call = crate::RuntimeCall::MantaSbt(pallet_manta_sbt::Call::new_mint_info {
            start_time: Default::default(),
            end_time: None,
            mint_name: Default::default(),
            public: true,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_manta_sbt", "new_mint_info", dispatch_info, call_len));
    }

    // pallet_parachain_staking
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("ParachainStaking").len(),
            26,
            "Please update new extrinsic here."
        );
        // set_staking_expectations
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::set_staking_expectations {
                expectations: pallet_parachain_staking::Range::from(1),
            },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "set_staking_expectations",
            dispatch_info,
            call_len,
        ));

        // set_inflation
        let call =
            crate::RuntimeCall::ParachainStaking(pallet_parachain_staking::Call::set_inflation {
                schedule: pallet_parachain_staking::Range::from(Perbill::from_percent(20)),
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "set_inflation",
            dispatch_info,
            call_len,
        ));

        // set_parachain_bond_account
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::set_parachain_bond_account { new: ALICE },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "set_parachain_bond_account",
            dispatch_info,
            call_len,
        ));

        // set_parachain_bond_account
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::set_parachain_bond_account { new: ALICE },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "set_parachain_bond_account",
            dispatch_info,
            call_len,
        ));

        // set_parachain_bond_reserve_percent
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::set_parachain_bond_reserve_percent {
                new: Default::default(),
            },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "set_parachain_bond_reserve_percent",
            dispatch_info,
            call_len,
        ));

        // set_total_selected
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::set_total_selected { new: 1 },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "set_total_selected",
            dispatch_info,
            call_len,
        ));

        // set_collator_commission
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::set_collator_commission {
                new: Default::default(),
            },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "set_collator_commission",
            dispatch_info,
            call_len,
        ));

        // set_blocks_per_round
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::set_blocks_per_round { new: 1 },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "set_blocks_per_round",
            dispatch_info,
            call_len,
        ));

        // join_candidates
        let call =
            crate::RuntimeCall::ParachainStaking(pallet_parachain_staking::Call::join_candidates {
                bond: 1,
                candidate_count: 2,
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "join_candidates",
            dispatch_info,
            call_len,
        ));

        // schedule_leave_candidates
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::schedule_leave_candidates { candidate_count: 2 },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "schedule_leave_candidates",
            dispatch_info,
            call_len,
        ));

        // execute_leave_candidates
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::execute_leave_candidates {
                candidate: ALICE,
                candidate_delegation_count: 2,
            },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "execute_leave_candidates",
            dispatch_info,
            call_len,
        ));

        // cancel_leave_candidates
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::cancel_leave_candidates { candidate_count: 2 },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "cancel_leave_candidates",
            dispatch_info,
            call_len,
        ));

        // go_offline
        let call =
            crate::RuntimeCall::ParachainStaking(pallet_parachain_staking::Call::go_offline {});
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "go_offline",
            dispatch_info,
            call_len,
        ));

        // go_offline
        let call =
            crate::RuntimeCall::ParachainStaking(pallet_parachain_staking::Call::go_offline {});
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "go_offline",
            dispatch_info,
            call_len,
        ));

        // candidate_bond_more
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::candidate_bond_more { more: 1 },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "candidate_bond_more",
            dispatch_info,
            call_len,
        ));

        // schedule_candidate_bond_less
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::schedule_candidate_bond_less { less: 1 },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "schedule_candidate_bond_less",
            dispatch_info,
            call_len,
        ));

        // execute_candidate_bond_less
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::execute_candidate_bond_less { candidate: ALICE },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "execute_candidate_bond_less",
            dispatch_info,
            call_len,
        ));

        // cancel_candidate_bond_less
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::cancel_candidate_bond_less {},
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "cancel_candidate_bond_less",
            dispatch_info,
            call_len,
        ));

        // delegate
        let call = crate::RuntimeCall::ParachainStaking(pallet_parachain_staking::Call::delegate {
            candidate: ALICE,
            amount: 1,
            candidate_delegation_count: 2,
            delegation_count: 3,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "delegate_1_2_3",
            dispatch_info,
            call_len,
        ));

        let call = crate::RuntimeCall::ParachainStaking(pallet_parachain_staking::Call::delegate {
            candidate: ALICE,
            amount: 1,
            candidate_delegation_count: 25,
            delegation_count: 3,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "delegate_1_25_3",
            dispatch_info,
            call_len,
        ));

        let call = crate::RuntimeCall::ParachainStaking(pallet_parachain_staking::Call::delegate {
            candidate: ALICE,
            amount: 1,
            candidate_delegation_count: 2,
            delegation_count: 100,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "delegate_1_2_100",
            dispatch_info,
            call_len,
        ));

        // schedule_leave_delegators
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::schedule_leave_delegators {},
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "schedule_leave_delegators",
            dispatch_info,
            call_len,
        ));

        // execute_leave_delegators
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::execute_leave_delegators {
                delegator: ALICE,
                delegation_count: 3,
            },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "execute_leave_delegators",
            dispatch_info,
            call_len,
        ));

        // cancel_leave_delegators
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::cancel_leave_delegators {},
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "cancel_leave_delegators",
            dispatch_info,
            call_len,
        ));

        // schedule_revoke_delegation
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::schedule_revoke_delegation { collator: ALICE },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "schedule_revoke_delegation",
            dispatch_info,
            call_len,
        ));

        // delegator_bond_more
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::delegator_bond_more {
                candidate: ALICE,
                more: 1,
            },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "delegator_bond_more",
            dispatch_info,
            call_len,
        ));

        // schedule_delegator_bond_less
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::schedule_delegator_bond_less {
                candidate: ALICE,
                less: 1,
            },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "schedule_delegator_bond_less",
            dispatch_info,
            call_len,
        ));

        // execute_delegation_request
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::execute_delegation_request {
                delegator: ALICE,
                candidate: ALICE,
            },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "execute_delegation_request",
            dispatch_info,
            call_len,
        ));

        // cancel_delegation_request
        let call = crate::RuntimeCall::ParachainStaking(
            pallet_parachain_staking::Call::cancel_delegation_request { candidate: ALICE },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "cancel_delegation_request",
            dispatch_info,
            call_len,
        ));
    }

    // pallet_scheduler
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("Scheduler").len(),
            6,
            "Please update new extrinsic here."
        );
        // cancel
        let call =
            crate::RuntimeCall::Scheduler(pallet_scheduler::Call::cancel { when: 1, index: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_scheduler", "cancel", dispatch_info, call_len));

        // schedule
        let call = crate::RuntimeCall::Scheduler(pallet_scheduler::Call::schedule {
            when: 1,
            maybe_periodic: None,
            priority: 1,
            call: Box::new(call),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_scheduler", "schedule", dispatch_info, call_len));

        // schedule_named
        let call = crate::RuntimeCall::Scheduler(pallet_scheduler::Call::schedule_named {
            id: [1u8; 32],
            when: 1,
            maybe_periodic: None,
            priority: 1,
            call: Box::new(call),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_scheduler",
            "schedule_named",
            dispatch_info,
            call_len,
        ));

        // cancel_named
        let call =
            crate::RuntimeCall::Scheduler(pallet_scheduler::Call::cancel_named { id: [1u8; 32] });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_scheduler", "cancel_named", dispatch_info, call_len));

        // schedule_after
        let call = crate::RuntimeCall::Scheduler(pallet_scheduler::Call::schedule_after {
            after: 1,
            maybe_periodic: None,
            priority: 1,
            call: Box::new(call),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_scheduler",
            "schedule_after",
            dispatch_info,
            call_len,
        ));

        // schedule_named_after
        let call = crate::RuntimeCall::Scheduler(pallet_scheduler::Call::schedule_named_after {
            id: [1u8; 32],
            after: 1,
            maybe_periodic: None,
            priority: 1,
            call: Box::new(call),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_scheduler",
            "schedule_named_after",
            dispatch_info,
            call_len,
        ));
    }

    // pallet_session
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("Session").len(),
            2,
            "Please update new extrinsic here."
        );
        // set_keys
        let keys = crate::opaque::SessionKeys::from_seed_unchecked("//Alice");
        let call = crate::RuntimeCall::Session(pallet_session::Call::set_keys {
            keys,
            proof: vec![1u8; 32],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_session", "set_keys", dispatch_info, call_len));

        // purge_keys
        let call = crate::RuntimeCall::Session(pallet_session::Call::purge_keys {});
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_session", "purge_keys", dispatch_info, call_len));
    }

    // pallet_tx_pause
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("TransactionPause").len(),
            6,
            "Please update new extrinsic here."
        );
        // set_mint_chain_info
        let call = crate::RuntimeCall::TransactionPause(pallet_tx_pause::Call::pause_transaction {
            pallet_name: vec![1u8; 32],
            function_name: vec![1u8; 32],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_tx_pause",
            "pause_transaction",
            dispatch_info,
            call_len,
        ));

        // unpause_transaction
        let call =
            crate::RuntimeCall::TransactionPause(pallet_tx_pause::Call::unpause_transaction {
                pallet_name: vec![1u8; 32],
                function_name: vec![1u8; 32],
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_tx_pause",
            "unpause_transaction",
            dispatch_info,
            call_len,
        ));

        // pause_transactions
        let call =
            crate::RuntimeCall::TransactionPause(pallet_tx_pause::Call::pause_transactions {
                pallet_and_funcs: vec![(vec![1u8; 32], vec![vec![1u8; 32]; 2])],
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_tx_pause",
            "pause_transactions",
            dispatch_info,
            call_len,
        ));

        // unpause_transactions
        let call =
            crate::RuntimeCall::TransactionPause(pallet_tx_pause::Call::unpause_transactions {
                pallet_and_funcs: vec![(vec![1u8; 32], vec![vec![1u8; 32]; 2])],
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_tx_pause",
            "unpause_transactions",
            dispatch_info,
            call_len,
        ));

        // pause_pallets
        let call = crate::RuntimeCall::TransactionPause(pallet_tx_pause::Call::pause_pallets {
            pallet_names: vec![vec![1u8; 32], vec![2u8; 32]],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_tx_pause", "pause_pallets", dispatch_info, call_len));

        // unpause_pallets
        let call = crate::RuntimeCall::TransactionPause(pallet_tx_pause::Call::unpause_pallets {
            pallet_names: vec![vec![1u8; 32], vec![2u8; 32]],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_tx_pause",
            "unpause_pallets",
            dispatch_info,
            call_len,
        ));
    }

    // pallet_utility
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("Utility").len(),
            6,
            "Please update new extrinsic here."
        );
        // batch, one call within
        let call = crate::RuntimeCall::Utility(pallet_utility::Call::batch { calls: vec![] });
        let call = crate::RuntimeCall::Utility(pallet_utility::Call::batch {
            calls: vec![call; 1],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_utility", "batch-size=1", dispatch_info, call_len));

        // batch, 32 call within
        let call = crate::RuntimeCall::Utility(pallet_utility::Call::batch { calls: vec![] });
        let call = crate::RuntimeCall::Utility(pallet_utility::Call::batch {
            calls: vec![call; 32],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_utility", "batch-size=32", dispatch_info, call_len));

        // as_derivative
        let call = crate::RuntimeCall::Utility(pallet_utility::Call::as_derivative {
            index: 1,
            call: Box::new(call),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_utility", "as_derivative", dispatch_info, call_len));

        // batch_all
        let call = crate::RuntimeCall::Utility(pallet_utility::Call::batch_all { calls: vec![] });
        let call = crate::RuntimeCall::Utility(pallet_utility::Call::batch {
            calls: vec![call; 1],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_utility",
            "batch_all-size=1",
            dispatch_info,
            call_len,
        ));

        // batch_all
        let call = crate::RuntimeCall::Utility(pallet_utility::Call::batch_all { calls: vec![] });
        let call = crate::RuntimeCall::Utility(pallet_utility::Call::batch {
            calls: vec![call; 32],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_utility",
            "batch_all-size=32",
            dispatch_info,
            call_len,
        ));

        // dispatch_as
        let origin: crate::RuntimeOrigin = frame_system::RawOrigin::Signed(ALICE).into();
        let as_origin: <crate::RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin =
            origin.caller().clone();
        let call = crate::RuntimeCall::Utility(pallet_utility::Call::dispatch_as {
            as_origin: Box::new(as_origin),
            call: Box::new(call),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_utility", "dispatch_as", dispatch_info, call_len));

        // force_batch
        let call = crate::RuntimeCall::Utility(pallet_utility::Call::force_batch { calls: vec![] });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_utility", "force_batch", dispatch_info, call_len));
    }

    // pallet_randomness
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("Randomness").len(),
            1,
            "Please update new extrinsic here."
        );
        // set_babe_randomness_results
        let call =
            crate::RuntimeCall::Randomness(pallet_randomness::Call::set_babe_randomness_results {});
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_randomness",
            "set_babe_randomness_results",
            dispatch_info,
            call_len,
        ));
    }

    // pallet_name_service
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("NameService").len(),
            5,
            "Please update new extrinsic here."
        );
        // register
        let call = crate::RuntimeCall::NameService(pallet_name_service::Call::register {
            username: "test".as_bytes().to_vec(),
            registrant: ALICE.clone().into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_name_service", "register", dispatch_info, call_len));

        // accept_register
        let call = crate::RuntimeCall::NameService(pallet_name_service::Call::accept_register {
            username: "test".as_bytes().to_vec(),
            registrant: ALICE.clone().into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_name_service",
            "accept_register",
            dispatch_info,
            call_len,
        ));

        // set_primary_name
        let call = crate::RuntimeCall::NameService(pallet_name_service::Call::set_primary_name {
            username: "test".as_bytes().to_vec(),
            registrant: ALICE.clone().into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_name_service",
            "set_primary_name",
            dispatch_info,
            call_len,
        ));

        // cancel_pending_register
        let call =
            crate::RuntimeCall::NameService(pallet_name_service::Call::cancel_pending_register {
                username: "test".as_bytes().to_vec(),
                registrant: ALICE.clone().into(),
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_name_service",
            "cancel_pending_register",
            dispatch_info,
            call_len,
        ));

        // remove_register
        let call = crate::RuntimeCall::NameService(pallet_name_service::Call::remove_register {
            username: "test".as_bytes().to_vec(),
            registrant: ALICE.clone().into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_name_service",
            "remove_register",
            dispatch_info,
            call_len,
        ));
    }

    // pallet_farming
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("Farming").len(),
            14,
            "Please update new extrinsic here."
        );
        // create_farming_pool
        let call = crate::RuntimeCall::Farming(pallet_farming::Call::create_farming_pool {
            tokens_proportion: vec![
                (1, Perbill::from_percent(20)),
                (2, Perbill::from_percent(40)),
            ],
            basic_rewards: vec![(1, 64), (2, 128)],
            gauge_init: None,
            min_deposit_to_start: 10,
            after_block_to_start: 100,
            withdraw_limit_time: 200,
            claim_limit_time: 300,
            withdraw_limit_count: 6,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_farming",
            "create_farming_pool",
            dispatch_info,
            call_len,
        ));

        // charge
        let call = crate::RuntimeCall::Farming(pallet_farming::Call::charge {
            pool_id: 1,
            rewards: vec![(1, 64), (2, 128)],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_farming", "charge", dispatch_info, call_len));

        // deposit
        let call = crate::RuntimeCall::Farming(pallet_farming::Call::deposit {
            pool_id: 1,
            add_value: 100,
            gauge_info: Some((10, 64)),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_farming", "deposit", dispatch_info, call_len));

        // withdraw
        let call = crate::RuntimeCall::Farming(pallet_farming::Call::withdraw {
            pool_id: 1,
            remove_value: Some(100),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_farming", "withdraw", dispatch_info, call_len));

        // claim
        let call = crate::RuntimeCall::Farming(pallet_farming::Call::claim { pool_id: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_farming", "claim", dispatch_info, call_len));

        // withdraw_claim
        let call = crate::RuntimeCall::Farming(pallet_farming::Call::withdraw_claim { pool_id: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_farming", "withdraw_claim", dispatch_info, call_len));

        // close_pool
        let call = crate::RuntimeCall::Farming(pallet_farming::Call::close_pool { pool_id: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_farming", "close_pool", dispatch_info, call_len));

        // set_retire_limit
        let call =
            crate::RuntimeCall::Farming(pallet_farming::Call::set_retire_limit { limit: 16 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_farming",
            "set_retire_limit",
            dispatch_info,
            call_len,
        ));

        // retire_pool
        let call = crate::RuntimeCall::Farming(pallet_farming::Call::retire_pool { pool_id: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_farming", "retire_pool", dispatch_info, call_len));

        // reset_pool
        let call = crate::RuntimeCall::Farming(pallet_farming::Call::reset_pool {
            pool_id: 1,
            basic_rewards: Some(vec![(1, 64), (2, 128)]),
            min_deposit_to_start: Some(10),
            after_block_to_start: Some(100),
            withdraw_limit_time: Some(200),
            claim_limit_time: Some(300),
            withdraw_limit_count: Some(6),
            gauge_init: None,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_farming", "reset_pool", dispatch_info, call_len));

        // kill_pool
        let call = crate::RuntimeCall::Farming(pallet_farming::Call::kill_pool { pool_id: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_farming", "kill_pool", dispatch_info, call_len));

        // edit_pool
        let call = crate::RuntimeCall::Farming(pallet_farming::Call::edit_pool {
            pool_id: 1,
            basic_rewards: Some(vec![(1, 64), (2, 128)]),
            withdraw_limit_time: Some(200),
            claim_limit_time: Some(300),
            gauge_basic_rewards: Some(vec![(1, 64), (2, 128)]),
            withdraw_limit_count: Some(6),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_farming", "edit_pool", dispatch_info, call_len));

        // gauge_withdraw
        let call = crate::RuntimeCall::Farming(pallet_farming::Call::gauge_withdraw { gid: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_farming", "gauge_withdraw", dispatch_info, call_len));

        // force_gauge_claim
        let call = crate::RuntimeCall::Farming(pallet_farming::Call::force_gauge_claim { gid: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_farming",
            "force_gauge_claim",
            dispatch_info,
            call_len,
        ));
    }

    // pallet_lottery
    {
        assert_eq!(
            crate::RuntimeCall::get_call_names("Lottery").len(),
            12,
            "Please update new extrinsic here."
        );
        t.execute_with(|| {
            // deposit
            let call = crate::RuntimeCall::Lottery(pallet_lottery::Call::deposit { amount: 10 });
            let (dispatch_info, call_len) = get_call_details(&call);
            calamari_runtime_calls.push(("pallet_lottery", "deposit", dispatch_info, call_len));
            // request_withdraw
            let call =
                crate::RuntimeCall::Lottery(pallet_lottery::Call::request_withdraw { amount: 10 });
            let (dispatch_info, call_len) = get_call_details(&call);
            calamari_runtime_calls.push((
                "pallet_lottery",
                "request_withdraw",
                dispatch_info,
                call_len,
            ));

            // claim_my_winnings
            let call = crate::RuntimeCall::Lottery(pallet_lottery::Call::claim_my_winnings {});
            let (dispatch_info, call_len) = get_call_details(&call);
            calamari_runtime_calls.push((
                "pallet_lottery",
                "claim_my_winnings",
                dispatch_info,
                call_len,
            ));
            // rebalance_stake
            let call = crate::RuntimeCall::Lottery(pallet_lottery::Call::rebalance_stake {});
            let (dispatch_info, call_len) = get_call_details(&call);
            calamari_runtime_calls.push((
                "pallet_lottery",
                "rebalance_stake",
                dispatch_info,
                call_len,
            ));

            // start_lottery
            let call = crate::RuntimeCall::Lottery(pallet_lottery::Call::start_lottery {});
            let (dispatch_info, call_len) = get_call_details(&call);
            calamari_runtime_calls.push((
                "pallet_lottery",
                "start_lottery",
                dispatch_info,
                call_len,
            ));

            // stop_lottery
            let call = crate::RuntimeCall::Lottery(pallet_lottery::Call::stop_lottery {});
            let (dispatch_info, call_len) = get_call_details(&call);
            calamari_runtime_calls.push((
                "pallet_lottery",
                "stop_lottery",
                dispatch_info,
                call_len,
            ));

            // draw_lottery
            let call = crate::RuntimeCall::Lottery(pallet_lottery::Call::draw_lottery {});
            let (dispatch_info, call_len) = get_call_details(&call);
            calamari_runtime_calls.push((
                "pallet_lottery",
                "draw_lottery",
                dispatch_info,
                call_len,
            ));

            // process_matured_withdrawals
            let call =
                crate::RuntimeCall::Lottery(pallet_lottery::Call::process_matured_withdrawals {});
            let (dispatch_info, call_len) = get_call_details(&call);
            calamari_runtime_calls.push((
                "pallet_lottery",
                "process_matured_withdrawals",
                dispatch_info,
                call_len,
            ));

            // liquidate_lottery
            let call = crate::RuntimeCall::Lottery(pallet_lottery::Call::liquidate_lottery {});
            let (dispatch_info, call_len) = get_call_details(&call);
            calamari_runtime_calls.push((
                "pallet_lottery",
                "liquidate_lottery",
                dispatch_info,
                call_len,
            ));

            // set_min_deposit
            let call = crate::RuntimeCall::Lottery(pallet_lottery::Call::set_min_deposit {
                min_deposit: 10,
            });
            let (dispatch_info, call_len) = get_call_details(&call);
            calamari_runtime_calls.push((
                "pallet_lottery",
                "set_min_deposit",
                dispatch_info,
                call_len,
            ));

            // set_min_withdraw
            let call = crate::RuntimeCall::Lottery(pallet_lottery::Call::set_min_withdraw {
                min_withdraw: 10,
            });
            let (dispatch_info, call_len) = get_call_details(&call);
            calamari_runtime_calls.push((
                "pallet_lottery",
                "set_min_withdraw",
                dispatch_info,
                call_len,
            ));

            // set_gas_reserve
            let call = crate::RuntimeCall::Lottery(pallet_lottery::Call::set_gas_reserve {
                gas_reserve: 10,
            });
            let (dispatch_info, call_len) = get_call_details(&call);
            calamari_runtime_calls.push((
                "pallet_lottery",
                "set_gas_reserve",
                dispatch_info,
                call_len,
            ));
        });
    }

    // zenlink_protocol
    {
        use zenlink_protocol::AssetId as ZenlinkAssetId;
        assert_eq!(
            crate::RuntimeCall::get_call_names("ZenlinkProtocol").len(),
            16,
            "Please update new extrinsic here."
        );
        // set_fee_receiver
        let call = crate::RuntimeCall::ZenlinkProtocol(zenlink_protocol::Call::set_fee_receiver {
            receiver: Some(ALICE.clone()),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "zenlink_protocol",
            "set_fee_receiver",
            dispatch_info,
            call_len,
        ));

        // set_fee_point
        let call = crate::RuntimeCall::ZenlinkProtocol(zenlink_protocol::Call::set_fee_point {
            fee_point: 8,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("zenlink_protocol", "set_fee_point", dispatch_info, call_len));

        // transfer
        let asset_id = ZenlinkAssetId {
            chain_id: 2084,
            asset_type: 0,
            asset_index: 10, // native token
        };
        let call = crate::RuntimeCall::ZenlinkProtocol(zenlink_protocol::Call::transfer {
            asset_id,
            recipient: ALICE.clone(),
            amount: 10,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("zenlink_protocol", "transfer", dispatch_info, call_len));

        // create_pair
        let asset_1 = ZenlinkAssetId {
            chain_id: 2084,
            asset_type: 1,
            asset_index: 45, // non native token
        };
        let call = crate::RuntimeCall::ZenlinkProtocol(zenlink_protocol::Call::create_pair {
            asset_0: asset_id,
            asset_1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("zenlink_protocol", "transfer", dispatch_info, call_len));

        // add_liquidity
        let call = crate::RuntimeCall::ZenlinkProtocol(zenlink_protocol::Call::add_liquidity {
            asset_0: asset_id,
            asset_1,
            amount_0_desired: 2,
            amount_1_desired: 4,
            amount_0_min: 1,
            amount_1_min: 3,
            deadline: 20,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("zenlink_protocol", "add_liquidity", dispatch_info, call_len));

        // remove_liquidity
        let call = crate::RuntimeCall::ZenlinkProtocol(zenlink_protocol::Call::remove_liquidity {
            asset_0: asset_id,
            asset_1,
            liquidity: 4,
            amount_0_min: 1,
            amount_1_min: 3,
            recipient: ALICE.clone(),
            deadline: 20,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "zenlink_protocol",
            "remove_liquidity",
            dispatch_info,
            call_len,
        ));

        // swap_exact_assets_for_assets
        let asset_2 = ZenlinkAssetId {
            chain_id: 2084,
            asset_type: 8,
            asset_index: 45, // non native token
        };
        let call = crate::RuntimeCall::ZenlinkProtocol(
            zenlink_protocol::Call::swap_exact_assets_for_assets {
                amount_in: 1,
                amount_out_min: 4,
                path: vec![asset_id, asset_2, asset_1],
                recipient: ALICE.clone(),
                deadline: 20,
            },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "zenlink_protocol",
            "swap_exact_assets_for_assets",
            dispatch_info,
            call_len,
        ));

        // swap_assets_for_exact_assets
        let call = crate::RuntimeCall::ZenlinkProtocol(
            zenlink_protocol::Call::swap_assets_for_exact_assets {
                amount_out: 1,
                amount_in_max: 4,
                path: vec![asset_id, asset_2, asset_1],
                recipient: ALICE.clone(),
                deadline: 20,
            },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "zenlink_protocol",
            "swap_assets_for_exact_assets",
            dispatch_info,
            call_len,
        ));

        // bootstrap_create
        let call = crate::RuntimeCall::ZenlinkProtocol(zenlink_protocol::Call::bootstrap_create {
            asset_0: asset_id,
            asset_1,
            target_supply_0: 1000,
            target_supply_1: 5000,
            capacity_supply_0: 1000000,
            capacity_supply_1: 5000000,
            end: 20,
            rewards: vec![asset_2, asset_1],
            limits: vec![(asset_1, 50000), (asset_2, 60000)],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "zenlink_protocol",
            "bootstrap_create",
            dispatch_info,
            call_len,
        ));

        // bootstrap_contribute
        let call =
            crate::RuntimeCall::ZenlinkProtocol(zenlink_protocol::Call::bootstrap_contribute {
                asset_0: asset_id,
                asset_1,
                amount_0_contribute: 100,
                amount_1_contribute: 500,
                deadline: 20,
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "zenlink_protocol",
            "bootstrap_contribute",
            dispatch_info,
            call_len,
        ));

        // bootstrap_claim
        let call = crate::RuntimeCall::ZenlinkProtocol(zenlink_protocol::Call::bootstrap_claim {
            recipient: ALICE.clone(),
            asset_0: asset_id,
            asset_1,
            deadline: 20,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "zenlink_protocol",
            "bootstrap_claim",
            dispatch_info,
            call_len,
        ));

        // bootstrap_end
        let call = crate::RuntimeCall::ZenlinkProtocol(zenlink_protocol::Call::bootstrap_end {
            asset_0: asset_id,
            asset_1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("zenlink_protocol", "bootstrap_end", dispatch_info, call_len));

        // bootstrap_update
        let call = crate::RuntimeCall::ZenlinkProtocol(zenlink_protocol::Call::bootstrap_update {
            asset_0: asset_id,
            asset_1,
            target_supply_0: 1500,
            target_supply_1: 7500,
            capacity_supply_0: 10000000,
            capacity_supply_1: 50000000,
            end: 100,
            rewards: vec![asset_2, asset_1],
            limits: vec![(asset_1, 100000), (asset_2, 600000)],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "zenlink_protocol",
            "bootstrap_update",
            dispatch_info,
            call_len,
        ));

        // bootstrap_refund
        let call = crate::RuntimeCall::ZenlinkProtocol(zenlink_protocol::Call::bootstrap_refund {
            asset_0: asset_id,
            asset_1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "zenlink_protocol",
            "bootstrap_refund",
            dispatch_info,
            call_len,
        ));

        // bootstrap_charge_reward
        let call =
            crate::RuntimeCall::ZenlinkProtocol(zenlink_protocol::Call::bootstrap_charge_reward {
                asset_0: asset_id,
                asset_1,
                charge_rewards: vec![(asset_1, 1000), (asset_id, 6000)],
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "zenlink_protocol",
            "bootstrap_charge_reward",
            dispatch_info,
            call_len,
        ));

        // bootstrap_withdraw_reward
        let call = crate::RuntimeCall::ZenlinkProtocol(
            zenlink_protocol::Call::bootstrap_withdraw_reward {
                asset_0: asset_id,
                asset_1,
                recipient: ALICE.clone(),
            },
        );
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "zenlink_protocol",
            "bootstrap_withdraw_reward",
            dispatch_info,
            call_len,
        ));
    }

    (calamari_runtime_calls, t)
}
