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
        utxo::MerkleTreeConfiguration, AssetId, AssetValue, MultiProvingContext, Parameters,
        ProvingContext, UtxoAccumulatorModel,
    },
    parameters::{load_parameters, load_transfer_parameters, load_utxo_accumulator_model},
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
    TransferPost::try_from(test::payment::to_private::prove_full(
        proving_context,
        parameters,
        utxo_accumulator,
        asset_id,
        value,
        rng,
    ))
    .unwrap()
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
            TransferPost::try_from(to_private_0).unwrap(),
            TransferPost::try_from(to_private_1).unwrap(),
        ],
        TransferPost::try_from(private_transfer).unwrap(),
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
            TransferPost::try_from(to_private_0).unwrap(),
            TransferPost::try_from(to_private_1).unwrap(),
        ],
        TransferPost::try_from(to_public).unwrap(),
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
lazy_static::lazy_static! {
    static ref PROVING_CONTEXT: MultiProvingContext = load_proving_context();
    static ref PARAMETERS: Parameters = load_transfer_parameters();
    static ref UTXO_ACCUMULATOR_MODEL: UtxoAccumulatorModel = load_utxo_accumulator_model();
}

/// Loads the [`MultiProvingContext`].
#[inline]
fn load_proving_context() -> MultiProvingContext {
    manta_pay::parameters::load_proving_context(
        tempfile::tempdir()
            .expect("Unable to create temporary directory.")
            .path(),
    )
}
/// Builds sample transactions for testing.
#[inline]
fn main() -> Result<()> {
    use std::time::Instant;
    let now = Instant::now();

    let directory = tempfile::tempdir().expect("Unable to generate temporary test directory.");
    println!("[INFO] Temporary Directory: {:?}", directory);

    let mut rng = ChaCha20Rng::from_seed([0; 32]);
    // let (proving_context, _, parameters, utxo_accumulator_model) =
    //     load_parameters(directory.path()).expect("Unable to load parameters.");

    let mut utxo_accumulator = UtxoAccumulator::new(UTXO_ACCUMULATOR_MODEL.clone());

    let mut mints = Vec::new();
    let mut transfers = Vec::new();
    let mut reclaims = Vec::new();
    for i in 0..15_000 {
        // let asset_id = 8.into();
        let asset_id = (8 + (i % 10)).into();
        println!("Iteration count: {:?}", i);

        let to_private = sample_to_private(
            &PROVING_CONTEXT.to_private,
            &PARAMETERS,
            &mut utxo_accumulator,
            asset_id,
            10_000,
            &mut rng,
        );
        mints.push(to_private.clone());
        println!("to_private size: {:?}", to_private.encode().len());

        let (private_transfer_input, private_transfer) = sample_private_transfer(
            &PROVING_CONTEXT,
            &PARAMETERS,
            &mut utxo_accumulator,
            asset_id,
            [1_000, 2_000],
            &mut rng,
        );
        transfers.push(private_transfer_input[0].clone());
        transfers.push(private_transfer_input[1].clone());
        transfers.push(private_transfer.clone());
        println!(
            "private_transfer_input size: {:?}",
            private_transfer_input.encode().len()
        );
        println!(
            "private_transfer size: {:?}",
            private_transfer.encode().len()
        );

        let (to_public_input, to_public) = sample_to_public(
            &PROVING_CONTEXT,
            &PARAMETERS,
            &mut utxo_accumulator,
            asset_id,
            [1_000, 2_000],
            &mut rng,
        );
        reclaims.push(to_public_input[0].clone());
        reclaims.push(to_public_input[1].clone());
        reclaims.push(to_public.clone());
        println!("to_public_input size: {:?}", to_public_input.encode().len());
        println!("to_public size: {:?}", to_public.encode().len());
    }
    let mut file = File::create("./precomputed-15k-iterations/precomputed_mints_v3-5")?;
    file.write_all(&<[TransferPost]>::encode(&mints))?;
    let mut file = File::create("./precomputed-15k-iterations/precomputed_transfers_v3-5")?;
    file.write_all(&<[TransferPost]>::encode(&transfers))?;
    let mut file = File::create("./precomputed-15k-iterations/precomputed_reclaims_v3-5")?;
    file.write_all(&<[TransferPost]>::encode(&reclaims))?;

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    Ok(directory.close()?)
}
