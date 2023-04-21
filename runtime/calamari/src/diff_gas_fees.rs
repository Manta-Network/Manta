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
    dispatch::DispatchInfo,
    traits::{schedule::MaybeHashed, OriginTrait},
    weights::GetDispatchInfo,
};
use manta_primitives::assets::{AssetRegistryMetadata, TestingDefault};
use manta_support::manta_pay::TransferPost;
use pallet_transaction_payment::Multiplier;
use runtime_common::MinimumMultiplier;
use sp_runtime::{
    traits::{Hash, Saturating},
    AccountId32, Perbill, Percent,
};

const GAS_FEE_FLUCTUATION: Percent = Percent::from_percent(10);
const ALICE: AccountId32 = AccountId32::new([1u8; 32]);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct GasFeeDetail {
    module: String,
    extrinsic: String,
    gas_fee: f64,
}

fn get_call_details(call: &crate::Call) -> (DispatchInfo, u32) {
    let dispatch_info =
        <<Runtime as frame_system::Config>::Call as GetDispatchInfo>::get_dispatch_info(call);
    let call_len = call.using_encoded(|e| e.len()) as u32;
    (dispatch_info, call_len)
}

#[test]
fn diff_gas_fees() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    let csv_path = format!("{VERSION}-tx-fees.csv");
    let mut rdr = csv::Reader::from_path(csv_path).unwrap();

    let all_extrinsics_gas_fees = calculate_all_current_extrinsic_gas_fee();

    let mut last_release_gas_fees = rdr.deserialize().into_iter().map(|e| {
        let record: GasFeeDetail = e.unwrap();
        record
    });

    for GasFeeDetail {
        module,
        extrinsic,
        gas_fee,
    } in all_extrinsics_gas_fees
    {
        match last_release_gas_fees.find(|e| e.extrinsic == extrinsic) {
            Some(found) => {
                let fluctuation = Percent::from_float((gas_fee - found.gas_fee).abs() / found.gas_fee);
                assert!(fluctuation <= GAS_FEE_FLUCTUATION, "The gas fee fluctuation for the extrinsic {extrinsic} is {:?}, bigger than {:?}.", fluctuation, GAS_FEE_FLUCTUATION);
            }
            None => panic!("The extrinsic {module}.{extrinsic} is missing from current gas fees list, please add to latest csv file."),
        }
    }
}

#[test]
#[ignore]
fn write_all_current_extrinsic_gas_fee_to_csv() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    let csv_path = format!("{VERSION}-tx-fees.csv");

    let mut wtr = csv::Writer::from_path(csv_path).unwrap();
    let all_extrinsics_gas_fees = calculate_all_current_extrinsic_gas_fee();

    for extrinsic in all_extrinsics_gas_fees {
        wtr.serialize(extrinsic).unwrap();
    }
    wtr.flush().unwrap();
}

fn calculate_all_current_extrinsic_gas_fee() -> Vec<GasFeeDetail> {
    let multiplier = MinimumMultiplier::get();
    let decimal: Multiplier = Multiplier::from_u32(10).saturating_pow(12);

    let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap()
        .into();
    // set the minimum
    t.execute_with(|| {
        pallet_transaction_payment::NextFeeMultiplier::<Runtime>::set(multiplier);
    });

    let mut calamari_runtime_calls = vec![];
    // frame_system
    {
        // remark
        let call = crate::Call::System(frame_system::Call::remark {
            remark: vec![1u8; 32],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("frame_system", "remark", dispatch_info, call_len));
        // remark_with_event
        let call = crate::Call::System(frame_system::Call::remark_with_event {
            remark: vec![1u8; 32],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("frame_system", "remark_with_event", dispatch_info, call_len));
    }

    // pallet_treasury
    {}

    // pallet_timestamp
    {}

    // pallet_preimage
    {}

    // pallet_multisig
    {}

    // pallet_membership
    {}

    // pallet_democracy
    {}

    // pallet_collective
    {}

    // cumulus_pallet_xcmp_queue
    {}

    // calamari_vesting
    {
        // vest
        let call = crate::Call::CalamariVesting(calamari_vesting::Call::vest {});
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("calamari_vesting", "vest", dispatch_info, call_len));
    }

    // manta_collator_selection
    {
        // set_invulnerables
        let call =
            crate::Call::CollatorSelection(manta_collator_selection::Call::set_invulnerables {
                new: vec![ALICE],
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "manta_collator_selection",
            "set_invulnerables",
            dispatch_info,
            call_len,
        ));

        // set_desired_candidates
        let call = crate::Call::CollatorSelection(
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
        let call =
            crate::Call::CollatorSelection(manta_collator_selection::Call::set_candidacy_bond {
                bond: 1,
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "manta_collator_selection",
            "set_candidacy_bond",
            dispatch_info,
            call_len,
        ));

        // register_as_candidate
        let call = crate::Call::CollatorSelection(
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
        let call =
            crate::Call::CollatorSelection(manta_collator_selection::Call::register_candidate {
                new_candidate: ALICE,
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "manta_collator_selection",
            "register_candidate",
            dispatch_info,
            call_len,
        ));

        // leave_intent
        let call = crate::Call::CollatorSelection(manta_collator_selection::Call::leave_intent {});
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "manta_collator_selection",
            "leave_intent",
            dispatch_info,
            call_len,
        ));

        // remove_collator
        let call =
            crate::Call::CollatorSelection(manta_collator_selection::Call::remove_collator {
                collator: ALICE,
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "manta_collator_selection",
            "remove_collator",
            dispatch_info,
            call_len,
        ));

        // set_eviction_baseline
        let call =
            crate::Call::CollatorSelection(manta_collator_selection::Call::set_eviction_baseline {
                percentile: Default::default(),
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "manta_collator_selection",
            "set_eviction_baseline",
            dispatch_info,
            call_len,
        ));

        // set_eviction_tolerance
        let call = crate::Call::CollatorSelection(
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
        // register_asset
        let call = crate::Call::AssetManager(pallet_asset_manager::Call::register_asset {
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
        let call = crate::Call::AssetManager(pallet_asset_manager::Call::update_asset_location {
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
        let call = crate::Call::AssetManager(pallet_asset_manager::Call::update_asset_metadata {
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
        let call = crate::Call::AssetManager(pallet_asset_manager::Call::set_units_per_second {
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
        let call = crate::Call::AssetManager(pallet_asset_manager::Call::mint_asset {
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
        let call = crate::Call::AssetManager(pallet_asset_manager::Call::set_min_xcm_fee {
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
    }

    // pallet_assets
    {
        // create
        let call = crate::Call::Assets(pallet_assets::Call::create {
            id: 1,
            admin: ALICE.into(),
            min_balance: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "create", dispatch_info, call_len));

        // force_create
        let call = crate::Call::Assets(pallet_assets::Call::force_create {
            id: 1,
            owner: ALICE.into(),
            is_sufficient: true,
            min_balance: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "force_create", dispatch_info, call_len));

        // destroy
        // let encoded_witness = [1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0];
        let encoded_witness = b"100010001000";
        let witness =
            pallet_assets::DestroyWitness::decode(&mut encoded_witness.as_slice()).unwrap();
        let call = crate::Call::Assets(pallet_assets::Call::destroy { id: 1, witness });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "destroy", dispatch_info, call_len));

        // mint
        let call = crate::Call::Assets(pallet_assets::Call::mint {
            id: 1,
            beneficiary: ALICE.into(),
            amount: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "mint", dispatch_info, call_len));

        // burn
        let call = crate::Call::Assets(pallet_assets::Call::burn {
            id: 1,
            who: ALICE.into(),
            amount: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "burn", dispatch_info, call_len));

        // transfer
        let call = crate::Call::Assets(pallet_assets::Call::transfer {
            id: 1,
            target: ALICE.into(),
            amount: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "transfer", dispatch_info, call_len));

        // transfer_keep_alive
        let call = crate::Call::Assets(pallet_assets::Call::transfer_keep_alive {
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
        let call = crate::Call::Assets(pallet_assets::Call::force_transfer {
            id: 1,
            source: ALICE.into(),
            dest: ALICE.into(),
            amount: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "force_transfer", dispatch_info, call_len));

        // freeze
        let call = crate::Call::Assets(pallet_assets::Call::freeze {
            id: 1,
            who: ALICE.into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "freeze", dispatch_info, call_len));

        // thaw
        let call = crate::Call::Assets(pallet_assets::Call::thaw {
            id: 1,
            who: ALICE.into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "thaw", dispatch_info, call_len));

        // freeze_asset
        let call = crate::Call::Assets(pallet_assets::Call::freeze_asset { id: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "freeze_asset", dispatch_info, call_len));

        // thaw_asset
        let call = crate::Call::Assets(pallet_assets::Call::thaw_asset { id: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "thaw_asset", dispatch_info, call_len));

        // transfer_ownership
        let call = crate::Call::Assets(pallet_assets::Call::transfer_ownership {
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
        let call = crate::Call::Assets(pallet_assets::Call::set_team {
            id: 1,
            issuer: ALICE.into(),
            admin: ALICE.into(),
            freezer: ALICE.into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "set_team", dispatch_info, call_len));

        // set_metadata
        let call = crate::Call::Assets(pallet_assets::Call::set_metadata {
            id: 1,
            name: vec![1u8; 32],
            symbol: vec![1u8; 32],
            decimals: 12,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "set_metadata", dispatch_info, call_len));

        // clear_metadata
        let call = crate::Call::Assets(pallet_assets::Call::clear_metadata { id: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "clear_metadata", dispatch_info, call_len));

        // force_set_metadata
        let call = crate::Call::Assets(pallet_assets::Call::force_set_metadata {
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
        let call = crate::Call::Assets(pallet_assets::Call::force_clear_metadata { id: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_assets",
            "force_clear_metadata",
            dispatch_info,
            call_len,
        ));

        // force_asset_status
        let call = crate::Call::Assets(pallet_assets::Call::force_asset_status {
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
        let call = crate::Call::Assets(pallet_assets::Call::approve_transfer {
            id: 1,
            delegate: ALICE.into(),
            amount: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "approve_transfer", dispatch_info, call_len));

        // cancel_approval
        let call = crate::Call::Assets(pallet_assets::Call::cancel_approval {
            id: 1,
            delegate: ALICE.into(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "cancel_approval", dispatch_info, call_len));

        // force_cancel_approval
        let call = crate::Call::Assets(pallet_assets::Call::force_cancel_approval {
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
        let call = crate::Call::Assets(pallet_assets::Call::transfer_approved {
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
        let call = crate::Call::Assets(pallet_assets::Call::touch { id: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "touch", dispatch_info, call_len));

        // refund
        let call = crate::Call::Assets(pallet_assets::Call::refund {
            id: 1,
            allow_burn: true,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_assets", "refund", dispatch_info, call_len));
    }

    // pallet_author_inherent
    {
        // kick_off_authorship_validation
        let call = crate::Call::AuthorInherent(
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
        // transfer
        let call = crate::Call::Balances(pallet_balances::Call::transfer {
            dest: ALICE.into(),
            value: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_balances", "transfer", dispatch_info, call_len));

        // set_balance
        let call = crate::Call::Balances(pallet_balances::Call::set_balance {
            who: ALICE.into(),
            new_free: 1,
            new_reserved: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_balances", "set_balance", dispatch_info, call_len));

        // force_transfer
        let call = crate::Call::Balances(pallet_balances::Call::force_transfer {
            source: ALICE.into(),
            dest: ALICE.into(),
            value: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_balances", "force_transfer", dispatch_info, call_len));

        // transfer_keep_alive
        let call = crate::Call::Balances(pallet_balances::Call::transfer_keep_alive {
            dest: ALICE.into(),
            value: 1,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_balances",
            "transfer_keep_alive",
            dispatch_info,
            call_len,
        ));

        // transfer_all
        let call = crate::Call::Balances(pallet_balances::Call::transfer_all {
            dest: ALICE.into(),
            keep_alive: false,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_balances", "transfer_all", dispatch_info, call_len));

        // force_unreserve
        let call = crate::Call::Balances(pallet_balances::Call::force_unreserve {
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
        let to_private_proof = [0u8; 552];
        let private_transfer_proof = [0u8; 1290];
        let to_public_proof = [0u8; 1000];
        // to_private
        let to_private_post = TransferPost::decode(&mut to_private_proof.as_slice()).unwrap();
        let call = crate::Call::MantaPay(pallet_manta_pay::Call::to_private {
            post: to_private_post,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_manta_pay", "to_private", dispatch_info, call_len));

        // private_transfer
        let private_transfer_post =
            TransferPost::decode(&mut private_transfer_proof.as_slice()).unwrap();
        let call = crate::Call::MantaPay(pallet_manta_pay::Call::private_transfer {
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
        let call = crate::Call::MantaPay(pallet_manta_pay::Call::to_public {
            post: to_public_post,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_manta_pay", "to_public", dispatch_info, call_len));

        // public_transfer
        let call = crate::Call::MantaPay(pallet_manta_pay::Call::public_transfer {
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
        let to_private_proof = [0u8; 552];
        // to_private
        let to_private_post =
            Box::new(TransferPost::decode(&mut to_private_proof.as_slice()).unwrap());
        let call = crate::Call::MantaSbt(pallet_manta_sbt::Call::to_private {
            post: to_private_post.clone(),
            metadata: Default::default(),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_manta_sbt", "to_private", dispatch_info, call_len));

        // reserve_sbt
        let call = crate::Call::MantaSbt(pallet_manta_sbt::Call::reserve_sbt {});
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_manta_sbt", "reserve_sbt", dispatch_info, call_len));

        // allowlist_evm_account
        let call = crate::Call::MantaSbt(pallet_manta_sbt::Call::allowlist_evm_account {
            evm_address: pallet_manta_sbt::EvmAddressType::Bab(Default::default()),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_manta_sbt",
            "allowlist_evm_account",
            dispatch_info,
            call_len,
        ));

        // mint_sbt_eth
        let call = crate::Call::MantaSbt(pallet_manta_sbt::Call::mint_sbt_eth {
            post: to_private_post,
            chain_id: 1,
            eth_signature: [1u8; 65],
            address_type: pallet_manta_sbt::EvmAddressType::Bab(Default::default()),
            collection_id: Some(1),
            item_id: Some(1),
            metadata: None,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_manta_sbt", "mint_sbt_eth", dispatch_info, call_len));

        // change_allowlist_account
        let call = crate::Call::MantaSbt(pallet_manta_sbt::Call::change_allowlist_account {
            account: Some(ALICE),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_manta_sbt",
            "change_allowlist_account",
            dispatch_info,
            call_len,
        ));

        // set_mint_chain_info
        let call = crate::Call::MantaSbt(pallet_manta_sbt::Call::set_mint_chain_info {
            mint_type: pallet_manta_sbt::MintType::Bab,
            start_time: Default::default(),
            end_time: None,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_manta_sbt",
            "set_mint_chain_info",
            dispatch_info,
            call_len,
        ));
    }

    // pallet_parachain_staking
    {
        // set_staking_expectations
        let call = crate::Call::ParachainStaking(
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
        let call = crate::Call::ParachainStaking(pallet_parachain_staking::Call::set_inflation {
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
        let call = crate::Call::ParachainStaking(
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
        let call = crate::Call::ParachainStaking(
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
        let call = crate::Call::ParachainStaking(
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
        let call =
            crate::Call::ParachainStaking(pallet_parachain_staking::Call::set_total_selected {
                new: 1,
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "set_total_selected",
            dispatch_info,
            call_len,
        ));

        // set_collator_commission
        let call = crate::Call::ParachainStaking(
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
        let call =
            crate::Call::ParachainStaking(pallet_parachain_staking::Call::set_blocks_per_round {
                new: 1,
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "set_blocks_per_round",
            dispatch_info,
            call_len,
        ));

        // join_candidates
        let call = crate::Call::ParachainStaking(pallet_parachain_staking::Call::join_candidates {
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
        let call = crate::Call::ParachainStaking(
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
        let call = crate::Call::ParachainStaking(
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
        let call = crate::Call::ParachainStaking(
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
        let call = crate::Call::ParachainStaking(pallet_parachain_staking::Call::go_offline {});
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "go_offline",
            dispatch_info,
            call_len,
        ));

        // go_offline
        let call = crate::Call::ParachainStaking(pallet_parachain_staking::Call::go_offline {});
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "go_offline",
            dispatch_info,
            call_len,
        ));

        // candidate_bond_more
        let call =
            crate::Call::ParachainStaking(pallet_parachain_staking::Call::candidate_bond_more {
                more: 1,
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "candidate_bond_more",
            dispatch_info,
            call_len,
        ));

        // schedule_candidate_bond_less
        let call = crate::Call::ParachainStaking(
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
        let call = crate::Call::ParachainStaking(
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
        let call = crate::Call::ParachainStaking(
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
        let call = crate::Call::ParachainStaking(pallet_parachain_staking::Call::delegate {
            candidate: ALICE,
            amount: 1,
            candidate_delegation_count: 2,
            delegation_count: 3,
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "delegate",
            dispatch_info,
            call_len,
        ));

        // schedule_leave_delegators
        let call = crate::Call::ParachainStaking(
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
        let call = crate::Call::ParachainStaking(
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
        let call = crate::Call::ParachainStaking(
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
        let call = crate::Call::ParachainStaking(
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
        let call =
            crate::Call::ParachainStaking(pallet_parachain_staking::Call::delegator_bond_more {
                candidate: ALICE,
                more: 1,
            });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_parachain_staking",
            "delegator_bond_more",
            dispatch_info,
            call_len,
        ));

        // schedule_delegator_bond_less
        let call = crate::Call::ParachainStaking(
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
        let call = crate::Call::ParachainStaking(
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
        let call = crate::Call::ParachainStaking(
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
        // cancel
        let call = crate::Call::Scheduler(pallet_scheduler::Call::cancel { when: 1, index: 1 });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_scheduler", "cancel", dispatch_info, call_len));

        // schedule
        let hash = <Runtime as frame_system::Config>::Hashing::hash_of(&call);
        let hashed = MaybeHashed::Hash(hash.clone());
        let call = crate::Call::Scheduler(pallet_scheduler::Call::schedule {
            when: 1,
            maybe_periodic: None,
            priority: 1,
            call: Box::new(hashed),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_scheduler", "schedule", dispatch_info, call_len));

        // schedule_named
        let hash = <Runtime as frame_system::Config>::Hashing::hash_of(&call);
        let hashed = MaybeHashed::Hash(hash.clone());
        let call = crate::Call::Scheduler(pallet_scheduler::Call::schedule_named {
            id: vec![1u8; 32],
            when: 1,
            maybe_periodic: None,
            priority: 1,
            call: Box::new(hashed),
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
            crate::Call::Scheduler(pallet_scheduler::Call::cancel_named { id: vec![1u8; 32] });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_scheduler", "cancel_named", dispatch_info, call_len));

        // schedule_after
        let hash = <Runtime as frame_system::Config>::Hashing::hash_of(&call);
        let hashed = MaybeHashed::Hash(hash.clone());
        let call = crate::Call::Scheduler(pallet_scheduler::Call::schedule_after {
            after: 1,
            maybe_periodic: None,
            priority: 1,
            call: Box::new(hashed),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push((
            "pallet_scheduler",
            "schedule_after",
            dispatch_info,
            call_len,
        ));

        // schedule_named_after
        let hash = <Runtime as frame_system::Config>::Hashing::hash_of(&call);
        let hashed = MaybeHashed::Hash(hash.clone());
        let call = crate::Call::Scheduler(pallet_scheduler::Call::schedule_named_after {
            id: vec![1u8; 32],
            after: 1,
            maybe_periodic: None,
            priority: 1,
            call: Box::new(hashed),
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
        // set_keys
        let keys = crate::opaque::SessionKeys::from_seed_unchecked("//Alice");
        let call = crate::Call::Session(pallet_session::Call::set_keys {
            keys: keys,
            proof: vec![1u8; 32],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_session", "set_keys", dispatch_info, call_len));

        // purge_keys
        let call = crate::Call::Session(pallet_session::Call::purge_keys {});
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_session", "purge_keys", dispatch_info, call_len));
    }

    // pallet_tx_pause
    {
        // set_mint_chain_info
        let call = crate::Call::TransactionPause(pallet_tx_pause::Call::pause_transaction {
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
        let call = crate::Call::TransactionPause(pallet_tx_pause::Call::unpause_transaction {
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
        let call = crate::Call::TransactionPause(pallet_tx_pause::Call::pause_transactions {
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
        let call = crate::Call::TransactionPause(pallet_tx_pause::Call::unpause_transactions {
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
        let call = crate::Call::TransactionPause(pallet_tx_pause::Call::pause_pallets {
            pallet_names: vec![vec![1u8; 32], vec![2u8; 32]],
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_tx_pause", "pause_pallets", dispatch_info, call_len));

        // unpause_pallets
        let call = crate::Call::TransactionPause(pallet_tx_pause::Call::unpause_pallets {
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
        // batch
        let call = crate::Call::Utility(pallet_utility::Call::batch { calls: vec![] });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_utility", "batch", dispatch_info, call_len));

        // as_derivative
        let call = crate::Call::Utility(pallet_utility::Call::as_derivative {
            index: 1,
            call: Box::new(call),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_utility", "as_derivative", dispatch_info, call_len));

        // batch_all
        let call = crate::Call::Utility(pallet_utility::Call::batch_all { calls: vec![] });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_utility", "batch_all", dispatch_info, call_len));

        // dispatch_as
        let origin: crate::Origin = frame_system::RawOrigin::Signed(ALICE).into();
        let as_origin: <crate::Origin as frame_support::traits::OriginTrait>::PalletsOrigin =
            origin.caller().clone();
        let call = crate::Call::Utility(pallet_utility::Call::dispatch_as {
            as_origin: Box::new(as_origin),
            call: Box::new(call),
        });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_utility", "dispatch_as", dispatch_info, call_len));

        // force_batch
        let call = crate::Call::Utility(pallet_utility::Call::force_batch { calls: vec![] });
        let (dispatch_info, call_len) = get_call_details(&call);
        calamari_runtime_calls.push(("pallet_utility", "force_batch", dispatch_info, call_len));
    }

    let mut all_extrinsics_gas_fees = vec![];
    t.execute_with(|| {
        for (pallet_name, extrinsic_name, dispatch_info, call_len) in calamari_runtime_calls {
            let fee = TransactionPayment::compute_fee(call_len, &dispatch_info, 0);
            let float_gax_fees = Multiplier::try_from(fee).unwrap().div(decimal).to_float();
            let gas_fee = GasFeeDetail {
                module: pallet_name.to_owned(),
                extrinsic: extrinsic_name.to_owned(),
                gas_fee: float_gax_fees,
            };
            all_extrinsics_gas_fees.push(gas_fee);
        }
    });

    all_extrinsics_gas_fees
}
