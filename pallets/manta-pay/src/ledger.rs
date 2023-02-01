extern crate alloc;

use crate::common::Wrap;
use crate::types::{
    fp_decode, fp_encode, FullIncomingNote, OutgoingNote, Utxo, FP_DECODE, FP_ENCODE,
};
use crate::{
    errors::{ReceiverLedgerError, SenderLedgerError},
    pallet::*,
    Config,
};
use alloc::{vec, vec::Vec};
use core::marker::PhantomData;
use manta_pay::{
    config::{self, utxo::MerkleTreeConfiguration},
    manta_accounting::transfer::{receiver::ReceiverLedger, sender::SenderLedger},
    manta_crypto::merkle_tree::{self, forest::Configuration as _},
    manta_parameters::{self, Get as _},
    manta_util::codec::Decode as _,
};

pub use crate::types::{Checkpoint, PullResponse, RawCheckpoint};

pub struct ProxyLedger<T>(pub PhantomData<T>)
where
    T: Config;

impl<T> SenderLedger<config::Parameters> for ProxyLedger<T>
where
    T: Config,
{
    type SuperPostingKey = (Wrap<()>, ());
    type ValidUtxoAccumulatorOutput = Wrap<config::UtxoAccumulatorOutput>;
    type ValidNullifier = Wrap<config::Nullifier>;
    type Error = SenderLedgerError;

    #[inline]
    fn is_unspent(
        &self,
        nullifier: config::Nullifier,
    ) -> Result<Self::ValidNullifier, Self::Error> {
        if NullifierCommitmentSet::<T>::contains_key(
            fp_encode(nullifier.nullifier.commitment).map_err(SenderLedgerError::FpEncodeError)?,
        ) {
            Err(SenderLedgerError::AssetSpent)
        } else {
            Ok(Wrap(nullifier))
        }
    }

    #[inline]
    fn has_matching_utxo_accumulator_output(
        &self,
        output: config::UtxoAccumulatorOutput,
    ) -> Result<Self::ValidUtxoAccumulatorOutput, Self::Error> {
        let accumulator_output = fp_encode(output).map_err(SenderLedgerError::FpEncodeError)?;
        // NOTE: Checking for an empty(zeroed) byte array. This happens for UTXOs with `value = 0`,
        // for which you dont need a membership proof, but you still need a root (in this case
        // zeroed).
        if accumulator_output == [0u8; 32]
            || UtxoAccumulatorOutputs::<T>::contains_key(accumulator_output)
        {
            return Ok(Wrap(output));
        }
        Err(SenderLedgerError::InvalidUtxoAccumulatorOutput)
    }
    #[inline]
    fn spend_all<I>(
        &mut self,
        super_key: &Self::SuperPostingKey,
        iter: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = (Self::ValidUtxoAccumulatorOutput, Self::ValidNullifier)>,
    {
        let _ = super_key;
        let index = NullifierSetSize::<T>::get();
        let mut i = 0;
        for (_, nullifier) in iter {
            let nullifier_commitment = fp_encode(nullifier.0.nullifier.commitment)
                .map_err(SenderLedgerError::FpEncodeError)?;
            NullifierCommitmentSet::<T>::insert(nullifier_commitment, ());
            NullifierSetInsertionOrder::<T>::insert(
                index + i,
                (
                    nullifier_commitment,
                    OutgoingNote::try_from(nullifier.0.outgoing_note)
                        .map_err(SenderLedgerError::OutgoingNoteDecodeError)?,
                ),
            );
            i += 1;
        }
        if i != 0 {
            NullifierSetSize::<T>::set(index + i);
        }
        Ok(())
    }
}

impl<T> ReceiverLedger<config::Parameters> for ProxyLedger<T>
where
    T: Config,
{
    type SuperPostingKey = (Wrap<()>, ());
    type ValidUtxo = Wrap<config::Utxo>;
    type Error = ReceiverLedgerError<T>;

    #[inline]
    fn is_not_registered(&self, utxo: config::Utxo) -> Result<Self::ValidUtxo, Self::Error> {
        if UtxoSet::<T>::contains_key(
            Utxo::try_from(utxo).map_err(ReceiverLedgerError::UtxoDecodeError)?,
        ) {
            Err(ReceiverLedgerError::AssetRegistered)
        } else {
            Ok(Wrap(utxo))
        }
    }

    #[inline]
    fn register_all<I>(
        &mut self,
        super_key: &Self::SuperPostingKey,
        iter: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = (Self::ValidUtxo, config::Note)>,
    {
        let _ = super_key;
        let utxo_accumulator_model = config::UtxoAccumulatorModel::decode(
            manta_parameters::pay::parameters::UtxoAccumulatorModel::get()
                .ok_or(ReceiverLedgerError::ChecksumError)?,
        )
        .map_err(ReceiverLedgerError::MTParametersDecodeError)?;
        let utxo_accumulator_item_hash = config::utxo::UtxoAccumulatorItemHash::decode(
            manta_parameters::pay::parameters::UtxoAccumulatorItemHash::get()
                .ok_or(ReceiverLedgerError::ChecksumError)?,
        )
        .map_err(ReceiverLedgerError::UtxoAccumulatorItemHashDecodeError)?;
        let mut shard_indices = iter
            .into_iter()
            .map(|(utxo, note)| {
                (
                    MerkleTreeConfiguration::tree_index(
                        &utxo.0.item_hash(&utxo_accumulator_item_hash, &mut ()),
                    ),
                    utxo.0,
                    note,
                )
            })
            .collect::<Vec<_>>();
        shard_indices.sort_by_key(|(s, _, _)| *s);
        let mut shard_insertions = Vec::<(_, Vec<_>)>::new();
        for (shard_index, utxo, note) in shard_indices {
            match shard_insertions.last_mut() {
                Some((index, pairs)) if shard_index == *index => pairs.push((utxo, note)),
                _ => shard_insertions.push((shard_index, vec![(utxo, note)])),
            }
        }
        for (shard_index, insertions) in shard_insertions {
            let mut tree = ShardTrees::<T>::get(shard_index);
            let cloned_tree = tree.clone();
            let mut next_root = Option::<config::UtxoAccumulatorOutput>::None;
            let mut current_path = cloned_tree
                .current_path
                .try_into()
                .map_err(ReceiverLedgerError::PathDecodeError)?;
            let mut leaf_digest = tree
                .leaf_digest
                .map(|x| fp_decode(x.to_vec()).map_err(ReceiverLedgerError::FpDecodeError))
                .map_or(Ok(None), |r| r.map(Some))?;
            for (utxo, note) in insertions {
                next_root = Some(
                    merkle_tree::single_path::raw::insert(
                        &utxo_accumulator_model,
                        &mut leaf_digest,
                        &mut current_path,
                        utxo.item_hash(&utxo_accumulator_item_hash, &mut ()),
                    )
                    .ok_or(ReceiverLedgerError::MerkleTreeCapacityError)?,
                );
                let next_index = current_path.leaf_index().0 as u64;
                let utxo = Utxo::try_from(utxo).map_err(ReceiverLedgerError::UtxoDecodeError)?;
                UtxoSet::<T>::insert(utxo, ());
                Shards::<T>::insert(
                    shard_index,
                    next_index,
                    (
                        utxo,
                        FullIncomingNote::try_from(note)
                            .map_err(ReceiverLedgerError::FullNoteDecodeError)?,
                    ),
                );
            }
            tree.current_path = current_path
                .try_into()
                .map_err(ReceiverLedgerError::PathDecodeError)?;
            tree.leaf_digest = leaf_digest
                .map(|x| fp_encode(x).map_err(ReceiverLedgerError::FpEncodeError))
                .map_or(Ok(None), |r| r.map(Some))?;
            if let Some(next_root) = next_root {
                ShardTrees::<T>::insert(shard_index, tree);
                UtxoAccumulatorOutputs::<T>::insert(
                    fp_encode(next_root).map_err(ReceiverLedgerError::FpEncodeError)?,
                    (),
                );
            }
        }
        Ok(())
    }
}
