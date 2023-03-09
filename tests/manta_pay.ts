import { ApiPromise } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { blake2AsHex } from "@polkadot/util-crypto";
import { u8aToHex, numberToU8a } from '@polkadot/util';
import { single_map_storage_key, double_map_storage_key, delay, HashType } from './test-util';

// number of shards at MantaPay
export const manta_pay_config = {
    shard_number: 256,
    max_receivers_pull_size: 32768,
    max_senders_pull_size: 32768,
}

/**
 * generate utxo deterministically using shard_idx and utxo_idx.
 * @param shard_idx shard_idx of generated utxo.
 * @param utxo_idx utxo_idx within shard
 * @returns a Shards entry of size 370 bytes
 */
export function generate_shards_entry(shard_idx: number, utxo_idx: number): Uint8Array {
    return new Uint8Array([
        ...numberToU8a(shard_idx, 32 * 8),
        ...numberToU8a(utxo_idx, 64 * 8),
        ...numberToU8a(0, 274 * 8)
    ]);
}

/**
 * generate void number according to index.
 * @param index index of void number.
 * @returns a 128 byte array.
 */
export function generate_nullifier_set_entry(index: number): Uint8Array {
    return numberToU8a(index, 128 * 8);
}

/**
 * move checkpoint to next.
 * @param per_shard_amount number of movement per shard.
 * @param checkpoint initial checkpoint.
 * @returns moved checkpoint.
 */
function next_checkpoint(
    per_shard_amount: number,
    checkpoint: Array<number> ): Array<number>
{
    if (checkpoint.length !== manta_pay_config.shard_number) {
        throw "Checkpoint of wrong size";
    }

    const new_checkpoint = new Array<number>(manta_pay_config.shard_number);
    for(let i = 0; i < manta_pay_config.shard_number; ++i){
        new_checkpoint[i] = checkpoint[i] + per_shard_amount;
    }

    return new_checkpoint;
}

/**
 * generate a utxo batch from a checkpoint
 * @param per_shard_amount number of utxo per shard generated.
 * @param checkpoint the starting indices of each shard.
 * @returns an array of pair: [(storage_key, utxo_data)]
 */
function generate_batched_utxos(per_shard_amount: number, checkpoint: Array<number>){
    const data = [];
    for(let shard_idx = 0; shard_idx < manta_pay_config.shard_number; ++ shard_idx) {
        for(let utxo_idx = checkpoint[shard_idx]; utxo_idx < checkpoint[shard_idx] + per_shard_amount; ++ utxo_idx) {
            const shards_storage_key = double_map_storage_key(
                "MantaPay", "Shards", shard_idx, 8, HashType.TwoxConcat, utxo_idx, 64, HashType.TwoxConcat);
            const value_str = u8aToHex(generate_shards_entry(shard_idx, utxo_idx));
            data.push([shards_storage_key, value_str]);
        }
    }
    const new_checkpoint = next_checkpoint(per_shard_amount, checkpoint);
    return {data: data, checkpoint: new_checkpoint};
}

var referendumIndexObject = { referendumIndex: 0 };

/**
 *  Insert utxos in batches
 * @param api api object connecting to node.
 * @param keyring keyring to sign the extrinsics.
 * @param batch_number number of batches.
 * @param per_shard_amount number of utxos per shard per batch.
 * @param init_checkpoint initial checkpoint (an array of starting indices in shard).
 * @param max_wait_time_sec maximum waiting time (in second).
 * @returns number of batches successfully inserted.
 */
async function insert_utxos_in_batches(
    api: ApiPromise,
    keyring: KeyringPair,
    batch_number: number,
    per_shard_amount: number,
    init_checkpoint: Array<number>,
    max_wait_time_sec: number ): Promise<number>
{
    if (init_checkpoint.length !== manta_pay_config.shard_number) {
        throw "Checkpoint of wrong size";
    }

    let success_batch = 0;
    let cur_checkpoint = init_checkpoint;
    for (let batch_idx = 0; batch_idx < batch_number; batch_idx ++){
        const {data, checkpoint} = generate_batched_utxos(per_shard_amount, cur_checkpoint);
        cur_checkpoint = checkpoint;
        const callData = api.tx.system.setStorage(data);
        await execute_with_root_via_governance(api, keyring, callData, referendumIndexObject);
        await delay(5000);
    }

    // wait all txs finalized
    for(let i = 0; i < max_wait_time_sec; i ++){
        await delay(1000);
        if (success_batch === batch_number) {
            console.log("total wait: %i sec.", i + 1);
            return success_batch;
        }
    }
    return success_batch;
}

/**
 * generate void number insertion data.
 * @param start_index starting index.
 * @param amount_per_batch amount of void numbers generated.
 * @returns generated void number insertion data: [(storage_key, void_number)]
 */
function generate_vn_insertion_data(
    start_index: number,
    amount_per_batch: number): string[][] {
    const data = [];
    for (let idx = start_index; idx < start_index + amount_per_batch; idx ++){
        const vn_storage_key = single_map_storage_key(
            "MantaPay", "NullifierSetInsertionOrder", idx, 64, HashType.TwoxConcat);
        data.push([vn_storage_key, u8aToHex(generate_nullifier_set_entry(idx))]);
    }
    return data;
}

/**
 * Insert void numbers to ledgers in batch.
 * @param api API object connecting to the node.
 * @param keyring keyring to sign the extrinsics.
 * @param amount_per_batch amount of void numbers per batch.
 * @param batch_number number of batch inserted.
 * @param start_index starting index.
 * @param max_wait_time_sec maximum time before timeout (in sec).
 * @returns number of batches successfully inserted before timeout.
 */
async function insert_void_numbers_in_batch(
    api:ApiPromise,
    keyring: KeyringPair,
    amount_per_batch: number,
    batch_number: number,
    start_index: number,
    max_wait_time_sec: number): Promise<number>{

    let success_batch = 0;
    let sender_idx = start_index;
    for(let batch_idx = 0; batch_idx < batch_number; batch_idx ++) {
        console.log("start vn batch %i", batch_idx);
        const data = generate_vn_insertion_data(sender_idx, amount_per_batch);
        const callData = api.tx.system.setStorage(data);
        await execute_with_root_via_governance(api, keyring, callData, referendumIndexObject);
        sender_idx += amount_per_batch;
        await delay(5000);
    }
    // wait all txs finalized
    for(let i =0; i < max_wait_time_sec; i++){
        await delay(1000);
        if (success_batch === batch_number) {
            console.log("total wait: %i sec.", i + 1);
            return success_batch;
        }
    }
    return success_batch;
}

export type StoragePrepareConfig = {
    utxo_big_batch_number: number,
    utxo_batch_number: number,
    utxo_batch_size_per_shard: number,
    vn_batch_number: number,
    vn_batch_size: number,
}

/**
 * set up manta pay storage according to `config`.
 * @param api API object connecting to node.
 * @param keyring keyring to sign extrinsics.
 * @param init_utxo_idx initial utxo index.
 * @param config configuration to set up storage.
 */
export async function setup_storage(
    api:ApiPromise,
    keyring: KeyringPair,
    init_utxo_idx: number,
    config: StoragePrepareConfig) {

    const receiver_checkpoint = new Array<number>(manta_pay_config.shard_number);
    const check_idx = init_utxo_idx;
    console.log(">>>>>>>>> UTXO INSERT START >>>>>>>>");
    for (let big_batch_idx = 0; big_batch_idx < config.utxo_big_batch_number; big_batch_idx ++) {
        console.log(">>>> Inserting %i big batch UTXOs", big_batch_idx + 1);
        receiver_checkpoint.fill(check_idx);
        console.log("starting utxo idx: %i", receiver_checkpoint[0]);
        const utxo_batch_done = await insert_utxos_in_batches(
            api, keyring, config.utxo_batch_number, config.utxo_batch_size_per_shard, receiver_checkpoint, 250);
        console.log(">>>> Complete %i big batch with %i UTXOs",
            big_batch_idx + 1 , utxo_batch_done * config.utxo_batch_size_per_shard * manta_pay_config.shard_number);
    }
    console.log(">>>>>>>>> UTXO INSERT DONE >>>>>>>>");

    console.log(">>>> Inserting void numbers: %i per batch, %i batch",
        config.vn_batch_size, config.vn_batch_number);
    const vn_batch_done = await insert_void_numbers_in_batch(api, keyring, config.vn_batch_size, config.vn_batch_number, 0, 250);
    console.log(">>>> Complete inserting %i void numbers", vn_batch_done * config.vn_batch_size);
}

/**
 * Execute an extrinsic with Root origin via governance.
 * @param api API object connecting to node.
 * @param keyring keyring to sign extrinsics.
 * @param extrinsicData the callData of the extrinsic that will be executed
 * @param referendumIndexObject the index of the referendum that will be executed
 */
 export async function execute_with_root_via_governance(
    api: ApiPromise,
    keyring: KeyringPair,
    extrinsicData: any,
    referendumIndexObject: any
) {
    const encodedCallData = extrinsicData.method.toHex();
    await api.tx.preimage.notePreimage(encodedCallData).signAndSend(keyring, {nonce: -1});
    let encodedCallDataHash = blake2AsHex(encodedCallData);
    let externalProposeDefault = await api.tx.democracy.externalProposeDefault({
        Legacy: {
            hash: encodedCallDataHash
        }
    });
    const encodedExternalProposeDefault = externalProposeDefault.method.toHex();
    await api.tx.council.propose(1, encodedExternalProposeDefault, encodedExternalProposeDefault.length).signAndSend(keyring, {nonce: -1});
    let fastTrackCall = await api.tx.democracy.fastTrack(encodedCallDataHash, 1, 1);
    await api.tx.technicalCommittee.propose(1, fastTrackCall, fastTrackCall.encodedLength).signAndSend(keyring, {nonce: -1});
    await api.tx.democracy.vote(referendumIndexObject.referendumIndex, {
        Standard: { balance: 1_000_000_000_000, vote: { aye: true, conviction: 1 } },
    }).signAndSend(keyring, {nonce: -1});
    referendumIndexObject.referendumIndex++;
}
