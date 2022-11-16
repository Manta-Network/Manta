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

use anyhow::Result;
use indoc::indoc;
use manta_crypto::{
    merkle_tree::{forest::TreeArrayMerkleForest, full::Full},
    rand::{CryptoRng, RngCore, SeedableRng},
};
use manta_pay::{
    config::{
        utxo::v2::MerkleTreeConfiguration, AssetId, AssetValue, MultiProvingContext, Parameters,
        ProvingContext,
    },
    parameters::load_parameters,
    test,
};
use pallet_manta_pay::types::TransferPost;
use rand_chacha::ChaCha20Rng;
use scale_codec::Encode;
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
};
/// UTXO Accumulator for Building Circuits
type UtxoAccumulator =
    TreeArrayMerkleForest<MerkleTreeConfiguration, Full<MerkleTreeConfiguration>, 256>;

///
#[inline]
fn sample_to_private<R>(
    proving_context: &ProvingContext,
    parameters: &Parameters,
    utxo_accumulator: &mut UtxoAccumulator,
    asset_id: AssetId,
    value: AssetValue,
    rng: &mut R,
) -> TransferPost
where
    R: CryptoRng + RngCore + ?Sized,
{
    TransferPost::from(test::payment::to_private::prove_full(
        proving_context,
        parameters,
        utxo_accumulator,
        asset_id,
        value,
        rng,
    ))
}

/// Samples a [`PrivateTransfer`] transaction under two [`ToPrivate`]s.
#[inline]
fn sample_private_transfer<R>(
    proving_context: &MultiProvingContext,
    parameters: &Parameters,
    utxo_accumulator: &mut UtxoAccumulator,
    asset_id: AssetId,
    values: [AssetValue; 2],
    rng: &mut R,
) -> ([TransferPost; 2], TransferPost)
where
    R: CryptoRng + RngCore + ?Sized,
{
    let ([to_private_0, to_private_1], private_transfer) =
        test::payment::private_transfer::prove_full(
            proving_context,
            parameters,
            utxo_accumulator,
            asset_id,
            values,
            rng,
        );
    (
        [
            TransferPost::from(to_private_0),
            TransferPost::from(to_private_1),
        ],
        TransferPost::from(private_transfer),
    )
}

/// Samples a [`ToPublic`] transaction under two [`ToPrivate`]s.
#[inline]
fn sample_to_public<R>(
    proving_context: &MultiProvingContext,
    parameters: &Parameters,
    utxo_accumulator: &mut UtxoAccumulator,
    asset_id: AssetId,
    values: [AssetValue; 2],
    rng: &mut R,
) -> ([TransferPost; 2], TransferPost)
where
    R: CryptoRng + RngCore + ?Sized,
{
    let ([to_private_0, to_private_1], to_public) = test::payment::to_public::prove_full(
        proving_context,
        parameters,
        utxo_accumulator,
        asset_id,
        values,
        rng,
    );
    (
        [
            TransferPost::from(to_private_0),
            TransferPost::from(to_private_1),
        ],
        TransferPost::from(to_public),
    )
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
        target_file
            .parent()
            .expect("This file should have a parent."),
    )?;

    let directory = tempfile::tempdir().expect("Unable to generate temporary test directory.");
    println!("[INFO] Temporary Directory: {:?}", directory);

    let mut rng = ChaCha20Rng::from_seed([0; 32]);
    let (proving_context, _, parameters, utxo_accumulator_model) =
        load_parameters(directory.path()).expect("Unable to load parameters.");
    let mut utxo_accumulator = UtxoAccumulator::new(utxo_accumulator_model.clone());
    let asset_id = 8.into();

    let to_private = sample_to_private(
        &proving_context.to_private,
        &parameters,
        &mut utxo_accumulator,
        asset_id,
        10_000,
        &mut rng,
    );
    let (private_transfer_input, private_transfer) = sample_private_transfer(
        &proving_context,
        &parameters,
        &mut utxo_accumulator,
        asset_id,
        [10_000, 20_000],
        &mut rng,
    );
    let (to_public_input, to_public) = sample_to_public(
        &proving_context,
        &parameters,
        &mut utxo_accumulator,
        asset_id,
        [10_000, 20_000],
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

    write_const_array!(target_file, TO_PRIVATE, to_private)?;
    write_const_nested_array!(target_file, PRIVATE_TRANSFER_INPUT, private_transfer_input)?;
    write_const_array!(target_file, PRIVATE_TRANSFER, private_transfer)?;
    write_const_nested_array!(target_file, TO_PUBLIC_INPUT, to_public_input)?;
    write_const_array!(target_file, TO_PUBLIC, to_public)?;

    Ok(directory.close()?)
}
