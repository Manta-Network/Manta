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

//! Precomputed Transactions

/* TODO:

use anyhow::Result;
use indoc::indoc;
use manta_accounting::{
    asset::{Asset, AssetId},
    transfer::{self, test::assert_valid_proof, SpendingKey},
};
use manta_crypto::{
    accumulator::Accumulator,
    merkle_tree::{forest::TreeArrayMerkleForest, full::Full},
    rand::{CryptoRng, Rand, RngCore, Sample, SeedableRng},
};
use manta_pay::{
    config::{
        FullParameters, MerkleTreeConfiguration, Mint, MultiProvingContext, MultiVerifyingContext,
        Parameters, PrivateTransfer, ProvingContext, Reclaim, UtxoAccumulatorModel,
        VerifyingContext,
    },
    parameters::load_parameters,
};
use pallet_manta_pay::types::TransferPost;
use rand_chacha::ChaCha20Rng;
use scale_codec::Encode;
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

/// UTXO Accumulator for Building Circuits
type UtxoAccumulator =
    TreeArrayMerkleForest<MerkleTreeConfiguration, Full<MerkleTreeConfiguration>, 256>;

/// Samples a [`Mint`] transaction.
#[inline]
fn sample_mint<R>(
    proving_context: &ProvingContext,
    verifying_context: &VerifyingContext,
    parameters: &Parameters,
    utxo_accumulator_model: &UtxoAccumulatorModel,
    asset: Asset,
    rng: &mut R,
) -> TransferPost
where
    R: CryptoRng + RngCore + ?Sized,
{
    let mint = Mint::from_spending_key(parameters, &SpendingKey::gen(rng), asset, rng)
        .into_post(
            FullParameters::new(parameters, utxo_accumulator_model),
            proving_context,
            rng,
        )
        .expect("Unable to build MINT proof.");
    assert_valid_proof(verifying_context, &mint);
    mint.into()
}

/// Samples a [`PrivateTransfer`] transaction under two [`Mint`]s.
#[inline]
fn sample_private_transfer<R>(
    proving_context: &MultiProvingContext,
    verifying_context: &MultiVerifyingContext,
    parameters: &Parameters,
    utxo_accumulator_model: &UtxoAccumulatorModel,
    asset_0: Asset,
    asset_1: Asset,
    rng: &mut R,
) -> ([TransferPost; 2], TransferPost)
where
    R: CryptoRng + RngCore + ?Sized,
{
    let mut utxo_accumulator = UtxoAccumulator::new(utxo_accumulator_model.clone());
    let spending_key_0 = SpendingKey::new(rng.gen(), rng.gen());
    let (mint_0, pre_sender_0) = transfer::test::sample_mint(
        &proving_context.mint,
        FullParameters::new(parameters, utxo_accumulator.model()),
        &spending_key_0,
        asset_0,
        rng,
    )
    .expect("Unable to build MINT proof.");
    assert_valid_proof(&verifying_context.mint, &mint_0);
    let sender_0 = pre_sender_0
        .insert_and_upgrade(&mut utxo_accumulator)
        .expect("Just inserted so this should not fail.");
    let spending_key_1 = SpendingKey::new(rng.gen(), rng.gen());
    let (mint_1, pre_sender_1) = transfer::test::sample_mint(
        &proving_context.mint,
        FullParameters::new(parameters, utxo_accumulator.model()),
        &spending_key_1,
        asset_1,
        rng,
    )
    .expect("Unable to build MINT proof.");
    assert_valid_proof(&verifying_context.mint, &mint_1);
    let sender_1 = pre_sender_1
        .insert_and_upgrade(&mut utxo_accumulator)
        .expect("Just inserted so this should not fail.");
    let private_transfer = PrivateTransfer::build(
        [sender_0, sender_1],
        [
            spending_key_0.receiver(parameters, rng.gen(), asset_1),
            spending_key_1.receiver(parameters, rng.gen(), asset_0),
        ],
    )
    .into_post(
        FullParameters::new(parameters, utxo_accumulator.model()),
        &proving_context.private_transfer,
        rng,
    )
    .expect("Unable to build PRIVATE_TRANSFER proof.");
    assert_valid_proof(&verifying_context.private_transfer, &private_transfer);
    ([mint_0.into(), mint_1.into()], private_transfer.into())
}

/// Samples a [`Reclaim`] transaction under two [`Mint`]s.
#[inline]
fn sample_reclaim<R>(
    proving_context: &MultiProvingContext,
    verifying_context: &MultiVerifyingContext,
    parameters: &Parameters,
    utxo_accumulator_model: &UtxoAccumulatorModel,
    asset_0: Asset,
    asset_1: Asset,
    rng: &mut R,
) -> ([TransferPost; 2], TransferPost)
where
    R: CryptoRng + RngCore + ?Sized,
{
    let mut utxo_accumulator = UtxoAccumulator::new(utxo_accumulator_model.clone());
    let spending_key_0 = SpendingKey::new(rng.gen(), rng.gen());
    let (mint_0, pre_sender_0) = transfer::test::sample_mint(
        &proving_context.mint,
        FullParameters::new(parameters, utxo_accumulator.model()),
        &spending_key_0,
        asset_0,
        rng,
    )
    .expect("Unable to build MINT proof.");
    assert_valid_proof(&verifying_context.mint, &mint_0);
    let sender_0 = pre_sender_0
        .insert_and_upgrade(&mut utxo_accumulator)
        .expect("Just inserted so this should not fail.");
    let spending_key_1 = SpendingKey::new(rng.gen(), rng.gen());
    let (mint_1, pre_sender_1) = transfer::test::sample_mint(
        &proving_context.mint,
        FullParameters::new(parameters, utxo_accumulator.model()),
        &spending_key_1,
        asset_1,
        rng,
    )
    .expect("Unable to build MINT proof.");
    assert_valid_proof(&verifying_context.mint, &mint_1);
    let sender_1 = pre_sender_1
        .insert_and_upgrade(&mut utxo_accumulator)
        .expect("Just inserted so this should not fail.");
    let reclaim = Reclaim::build(
        [sender_0, sender_1],
        [spending_key_0.receiver(parameters, rng.gen(), asset_1)],
        asset_0,
    )
    .into_post(
        FullParameters::new(parameters, utxo_accumulator.model()),
        &proving_context.reclaim,
        rng,
    )
    .expect("Unable to build RECLAIM proof.");
    assert_valid_proof(&verifying_context.reclaim, &reclaim);
    ([mint_0.into(), mint_1.into()], reclaim.into())
}

/// Writes a new `const` definition to `$writer`.
macro_rules! write_const_array {
    ($writer:ident, $name:ident, $value:expr) => {
        writeln!(
            $writer,
            "pub(crate) const {}: &[u8] = &{:?};\n",
            stringify!($name),
            $value.encode().as_slice()
        )
    };
}

/// Writes a new `const` definition to `$writer`.
macro_rules! write_const_nested_array {
    ($writer:ident, $name:ident, $value:expr) => {
        writeln!(
            $writer,
            "pub(crate) const {}: &[&[u8]] = &[{}];\n",
            stringify!($name),
            $value
                .iter()
                .flat_map(|v| {
                    format!("&{:?},", v.encode().as_slice())
                        .chars()
                        .collect::<Vec<_>>()
                })
                .collect::<String>(),
        )
    };
}

/// Builds sample transactions for testing.
#[inline]
fn main() -> Result<()> {
    let target_file = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?.join("precomputed_coins.rs"));
    assert!(
        !target_file.exists(),
        "Specify a file to place the generated files: {:?}.",
        target_file,
    );
    fs::create_dir_all(
        &target_file
            .parent()
            .expect("This file should have a parent."),
    )?;

    let directory = tempfile::tempdir().expect("Unable to generate temporary test directory.");
    println!("[INFO] Temporary Directory: {:?}", directory);

    let mut rng = ChaCha20Rng::from_seed([0; 32]);
    let (proving_context, verifying_context, parameters, utxo_accumulator_model) =
        load_parameters(directory.path()).expect("Unable to load parameters.");
    let asset_id: u32 = 8;

    let mint = sample_mint(
        &proving_context.mint,
        &verifying_context.mint,
        &parameters,
        &utxo_accumulator_model,
        AssetId(asset_id).value(100_000),
        &mut rng,
    );
    let (private_transfer_input, private_transfer) = sample_private_transfer(
        &proving_context,
        &verifying_context,
        &parameters,
        &utxo_accumulator_model,
        AssetId(asset_id).value(10_000),
        AssetId(asset_id).value(20_000),
        &mut rng,
    );
    let (reclaim_input, reclaim) = sample_reclaim(
        &proving_context,
        &verifying_context,
        &parameters,
        &utxo_accumulator_model,
        AssetId(asset_id).value(10_000),
        AssetId(asset_id).value(20_000),
        &mut rng,
    );

    let mut target_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(target_file)?;

    writeln!(
        target_file,
        indoc! {r"
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

            //! Precomputed Coins
            //!
            //! THIS FILE IS AUTOMATICALLY GENERATED by `src/bin/precompute_coins.rs`. DO NOT EDIT.
        "}
    )?;

    write_const_array!(target_file, MINT, mint)?;
    write_const_nested_array!(target_file, PRIVATE_TRANSFER_INPUT, private_transfer_input)?;
    write_const_array!(target_file, PRIVATE_TRANSFER, private_transfer)?;
    write_const_nested_array!(target_file, RECLAIM_INPUT, reclaim_input)?;
    write_const_array!(target_file, RECLAIM, reclaim)?;

    directory
        .close()
        .expect("Unable to delete temporary test directory.");

    Ok(())
}

*/

fn main() {}
