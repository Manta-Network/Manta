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

//! Precomputed Transactions

use anyhow::Result;
use indoc::indoc;
use manta_crypto::{
    merkle_tree::{forest::TreeArrayMerkleForest, full::Full},
    rand::{CryptoRng, RngCore, SeedableRng},
};
use manta_pay::{
    config::{utxo::MerkleTreeConfiguration, AssetId, AssetValue, Parameters, ProvingContext},
    parameters::load_parameters,
    test,
};
use manta_support::manta_pay::TransferPost;
use rand_chacha::ChaCha20Rng;
use scale_codec::Encode;
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
    thread,
};
use std::sync::Arc;
/// UTXO Accumulator for Building Circuits
type UtxoAccumulator =
    TreeArrayMerkleForest<MerkleTreeConfiguration, Full<MerkleTreeConfiguration>, 256>;

///
#[inline]
fn to_private_example<R>(
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

/// Builds sample transactions for testing.
#[inline]
fn main() -> Result<()> {
    use std::time::Instant;
    let now = Instant::now();

    let target_file = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?.join("precomputed_coins.rs"));
    assert!(
        !target_file.exists(),
        "Specify a file to place the generated files: {target_file:?}.",
    );
    fs::create_dir_all(
        target_file
            .parent()
            .expect("This file should have a parent."),
    )?;

    let directory = tempfile::tempdir().expect("Unable to generate temporary test directory.");
    println!("[INFO] Temporary Directory: {directory:?}");

    let mut rng = Arc::new(ChaCha20Rng::from_seed([0; 32]));
    let (proving_context, _, parameters, utxo_accumulator_model) =
        load_parameters(directory.path()).expect("Unable to load parameters.");

    
    let step = 20;
    let mut threads = vec![];
    for i in 0..4 {
        let start = i * step;
        let end = step + start;
        let utxo = utxo_accumulator_model.clone();
        let mut utxo_accumulator = Arc::new(UtxoAccumulator::new(utxo));
        let mut r = rng.clone();
        let context = Arc::new(proving_context.clone());
        let params = Arc::new(parameters.clone());
        let thread_join_handle = thread::spawn(move || {
            let mut mints = Vec::new();
            for i in start..end {
                let asset_id = (0 + (i % 3)).into();
                println!("Iteration count: {:?}", i);

                let to_private = to_private_example(
                    &context.to_private,
                    &params,
                    &mut *Arc::make_mut(&mut utxo_accumulator),
                    asset_id,
                    1,
                    &mut *Arc::make_mut(&mut r)
                );
                mints.push(to_private.clone());
                println!("to_private size: {:?}", to_private.encode().len());
            }
            let path = format!("/home/jamie/my-repo/Manta/pallets/manta-sbt/precomputed_mints-{}-to_{}", start, end);
            let mut file = File::create(path).unwrap();
            file.write_all(&<[TransferPost]>::encode(&mints)).unwrap();
        });
        threads.push(thread_join_handle);
    }

    for t in threads {
        t.join();
    }

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    Ok(directory.close()?)
}
