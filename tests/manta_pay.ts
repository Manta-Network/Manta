import { ApiPromise } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { u8aToHex, numberToU8a } from '@polkadot/util';
import { single_map_storage_key, double_map_storage_key, delay, emojis, HashType } from './test-util';

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
 * @returns a utxo of 32 + 32 + 68 bytes
 */
export function generate_utxo(shard_idx: number, utxo_idx: number): Uint8Array {
    return new Uint8Array([
        ...numberToU8a(shard_idx, 32 * 8),
        ...numberToU8a(utxo_idx, 64 * 8),
        ...numberToU8a(0, 36 * 8)
    ]);
}

/**
 * generate void number according to index.
 * @param index index of void number.
 * @returns a 32 byte array.
 */
export function generate_void_number(index: number): Uint8Array {
    return numberToU8a(index, 32 * 8);
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
 * geenrate a utxo batch from a checkpoint
 * @param per_shard_amount number of utxo per shard generated.
 * @param checkpoint the starting indices of each shard.
 * @returns an array of pair: [(storage_key, utxo_data)]
 */
function generate_batched_utxos(per_shard_amount: number, checkpoint: Array<number>){
    const data = [];
    for(let shard_idx = 0; shard_idx < manta_pay_config.shard_number; ++ shard_idx) {
        for(let utxo_idx  = checkpoint[shard_idx]; utxo_idx < checkpoint[shard_idx] + per_shard_amount; ++ utxo_idx) {
            const shards_storage_key = double_map_storage_key(
                "MantaPay", "Shards", shard_idx, 8, HashType.TwoxConcat, utxo_idx, 64, HashType.TwoxConcat);
            const value_str = u8aToHex(generate_utxo(shard_idx, utxo_idx));
            data.push([shards_storage_key, value_str]);
        }
    }
    const new_checkpoint = next_checkpoint(per_shard_amount, checkpoint); 
    return {data: data, checkpoint: new_checkpoint};
}

/**
 *  Insert utxos in batches
 * @param api api object connecting to node.
 * @param keyring keyring to sign the extrinsics.
 * @param batch_number number of batches.
 * @param per_shard_amount number of utxos per shard per batch.
 * @param init_checkpoint initial checkpoint (an array of starting indices in shard).
 * @param max_wait_time_sec maximum waiting time (in second).
 * @returns number of batches sucessfully inserted.
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
        const call_data = api.tx.system.setStorage(data);
        // https://substrate.stackexchange.com/questions/1776/how-to-use-polkadot-api-to-send-multiple-transactions-simultaneously
        const unsub = await api.tx.sudo.sudo(call_data).signAndSend(keyring, {nonce: -1}, ({ events = [], status }) => {
           if (status.isFinalized) {
                success_batch ++;
                console.log("%s %i batch utxos insertion finalized.", emojis.write, success_batch);
                unsub();
            }
        });
    }
    
    // wait all txs finalized
    for(let i =0; i < max_wait_time_sec; i ++){
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
            "MantaPay", "VoidNumberSetInsertionOrder", idx, 64, HashType.TwoxConcat);
        data.push([vn_storage_key, u8aToHex(generate_void_number(idx))]);
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
 * @param max_wait_time_sec maxium time before timeout (in sec).
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
        const call_data = api.tx.system.setStorage(data);
        const unsub = await api.tx.sudo.sudo(call_data).signAndSend(keyring, {nonce: -1}, ({ events = [], status }) => {
            if (status.isFinalized) {
                success_batch ++;
                console.log("%s %i batchs void number insertion finalized.", emojis.write, success_batch);
                unsub();
            }
        });
        sender_idx += amount_per_batch;
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
            api, keyring, config.utxo_batch_number, config.utxo_batch_size_per_shard, receiver_checkpoint, 1000);
        console.log(">>>> Complete %i big batch with %i UTXOs", 
            big_batch_idx + 1 , utxo_batch_done * config.utxo_batch_size_per_shard * manta_pay_config.shard_number);
    }
    console.log(">>>>>>>>> UTXO INSERT DONE >>>>>>>>");

    console.log(">>>> Inserting void numbers: %i per batch, %i batchs", 
        config.vn_batch_size, config.vn_batch_number);
    const vn_batch_done = await insert_void_numbers_in_batch(api, keyring, config.vn_batch_size, config.vn_batch_number, 0, 1000);
    console.log(">>>> Complete inserting %i void numbers", vn_batch_done * config.vn_batch_size);
}

