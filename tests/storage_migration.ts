import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { manta_pay_types, rpc_api } from './types';
import { xxhashAsU8a } from '@polkadot/util-crypto';
import { readFile, writeFile } from 'fs/promises';
import { u8aToHex, hexToU8a } from '@polkadot/util';
import { ExecutionContext, emojis, delay } from './test-util';

const dolphin_config = {
    ws_address: "wss://ws.rococo.dolphin.engineering"
}

function convert_shard_utxo_keys(data: Uint8Array): Uint8Array{
    const shard_idx_data = data.slice(0, 1);
    const utxo_idx_data = data.slice(1,);
    return new Uint8Array([
        ...xxhashAsU8a(shard_idx_data, 64),
        ...shard_idx_data,
        ...xxhashAsU8a(utxo_idx_data, 64),
        ...utxo_idx_data
    ]);
}

function convert_single_map_keys(data: Uint8Array): Uint8Array{
    return new Uint8Array([
        ...xxhashAsU8a(data, 64),
        ...data
    ])
}

async function insert_value_batches(
    context: ExecutionContext,    
    kvs: Array<string[]>, 
    batch_size: number,
    timeout: number,
){
    let success_batch = 0;
    const expected_batch = Math.ceil(kvs.length/batch_size);
    for(let check_point = 0;  check_point < kvs.length; ){
        const finish_point = check_point + batch_size > kvs.length ? kvs.length : check_point + batch_size;
        const data = kvs.slice(check_point, finish_point);
        const call_data = context.api.tx.system.setStorage(data);
        const unsub = await context.api.tx.sudo.sudo(call_data).signAndSend(context.keyring, {nonce: -1}, ({ events = [], status }) => {
            if (status.isFinalized) {
                success_batch ++;
                console.log("%s %i batchs insertion finalized.", emojis.write, success_batch);
                unsub();
            }
        });
        check_point = finish_point;
    }

    // wait all txs finalized
    for(let i =0; i < timeout; i ++){
        await delay(1000);
        if (success_batch === expected_batch) {
            console.log("total wait: %i sec.", i + 1);
            return success_batch;
        }
    }
    throw "timeout";
}

async function insert_values(
    context: ExecutionContext,
    kvs: Array<string[]>, 
    batch_size = 4096,
    batch_count_before_gap = 4,
    timeout_for_big_batch = 1000, 
){
    const big_batch_size = batch_size * batch_count_before_gap;
    for(let check_point = 0; check_point < kvs.length; ){
        const finish_point = check_point + big_batch_size > kvs.length ? kvs.length : check_point + big_batch_size;
        console.log(">>>>>> writng big batch from %i", check_point);
        await insert_value_batches(context, kvs.slice(check_point, finish_point), batch_size, timeout_for_big_batch);
        check_point = finish_point;
    }
}

async function drop_keys_in_batches(
    context: ExecutionContext,    
    keys: Array<string>, 
    batch_size: number,
    timeout: number,
){
    let success_batch = 0;
    const expected_batch = Math.ceil(keys.length/batch_size);
    for(let check_point = 0;  check_point < keys.length; ){
        const finish_point = check_point + batch_size > keys.length ? keys.length : check_point + batch_size;
        const data = keys.slice(check_point, finish_point);
        const call_data = context.api.tx.system.killStorage(data);
        const unsub = await context.api.tx.sudo.sudo(call_data).signAndSend(context.keyring, {nonce: -1}, ({ events = [], status }) => {
            if (status.isFinalized) {
                success_batch ++;
                console.log("%s %i batchs insertion finalized.", emojis.write, success_batch);
                unsub();
            }
        });
        check_point = finish_point;
    }

    // wait all txs finalized
    for(let i =0; i < timeout; i ++){
        await delay(1000);
        if (success_batch === expected_batch) {
            console.log("total wait: %i sec.", i + 1);
            return success_batch;
        }
    }
    throw "timeout";
}

async function drop_keys(
    context: ExecutionContext,
    keys: Array<string>, 
    batch_size = 4096,
    batch_count_before_gap = 4,
    timeout_for_big_batch = 1000, 
){
    const big_batch_size = batch_size * batch_count_before_gap;
    for(let check_point = 0; check_point < keys.length; ){
        const finish_point = check_point + big_batch_size > keys.length ? keys.length : check_point + big_batch_size;
        console.log(">>>>>> writng big batch from %i", check_point);
        await drop_keys_in_batches(context, keys.slice(check_point, finish_point), batch_size, timeout_for_big_batch);
        check_point = finish_point;
    }
}


function convert_single_map_keys_hex(key: string, changed_bytes: number): string {
    const bytes = hexToU8a(key);
    const transformed_last_bytes = convert_single_map_keys(bytes.slice(-changed_bytes));
    const result = new Uint8Array([
        ...bytes.slice(0, -changed_bytes),
        ...transformed_last_bytes
    ]);
    return u8aToHex(result);
}

async function main(){
    const wsProvider = new WsProvider(dolphin_config.ws_address);

    const api = await ApiPromise.create({ 
        provider: wsProvider,
        types: manta_pay_types,
        rpc: rpc_api});
    
    const keyring = new Keyring({ type: 'sr25519' });
    const sudo_key_pair = keyring.addFromMnemonic('bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice');
    const context: ExecutionContext = {
        api: api,
        keyring: sudo_key_pair,
    }
    const manta_keys_read_raw = await readFile('./manta_pay_keys.json');
    const manta_keys_read = JSON.parse(manta_keys_read_raw.toString());
    // shards key: take last 9 bytes
    // shard_tree key: take last 1 bytes
    // utxo_acc_outputs: take last 32 bytes
    // utxo_set: take last 32 bytes
    // void_number_set: take last 32 bytes
    // void_number_set_insertion_order: take last 8 bytes

    // drop keys
    await drop_keys(context, manta_keys_read.shards);
    const shards_keys = await api.query.mantaPay.shards.keys();
    console.log("shards key count: %i", shards_keys.length);
    await drop_keys(context, manta_keys_read.shard_trees);
    const shard_trees_keys = await api.query.mantaPay.shardTrees.keys();
    console.log("shard trees key count: %i", shard_trees_keys.length);
    await drop_keys(context, manta_keys_read.utxo_acc_outputs);
    const utxo_acc_outputs = await api.query.mantaPay.utxoAccumulatorOutputs.keys();
    console.log("key count: %i", utxo_acc_outputs.length);
    await drop_keys(context, manta_keys_read.utxo_set);
    const utxo_set = await api.query.mantaPay.utxoSet.keys();
    console.log("drop %i keys from UtxoSet", utxo_set.length);

    await drop_keys(context, manta_keys_read.void_number_set);
    const void_number_set = await api.query.mantaPay.voidNumberSet.keys();
    console.log("drop %i keys from VoidNumberSet", void_number_set.length);

    await drop_keys(context, manta_keys_read.void_number_set_insertion_order);
    const vnsio = await api.query.mantaPay.voidNumberSetInsertionOrder.keys();
    console.log("Fetched %i keys from VNSIO", vnsio.length);
    const shards_raw = await readFile('./shards.json');
    const shards = JSON.parse(shards_raw.toString());
    const shard_trees_raw = await readFile('./shards_trees.json');
    const shard_trees = JSON.parse(shard_trees_raw.toString());
    const void_number_set_insertion_order_raw = await readFile('./void_number_set_insertion_order.json');
    const void_number_set_insertion_order = JSON.parse(void_number_set_insertion_order_raw.toString());

    // inserting new shards
    const new_shards_keys = (manta_keys_read.shards as Array<string>).map((entry)=>{
        const bytes = hexToU8a(entry);
        const transformed_last_bytes = convert_shard_utxo_keys(bytes.slice(-9));
        const result = new Uint8Array([
            ...bytes.slice(0, -9),
            ...transformed_last_bytes
        ]);
        return u8aToHex(result);
    });

    if(new_shards_keys.length !== (shards as Array<string>).length) 
        throw "shards keys and values are not the same len.";
    const new_shards_kvs = new_shards_keys.map((e, i)=>{
        return [e, shards[i]];
    });
    await insert_values(context, new_shards_kvs);

    // inserting new shard trees
    const new_shard_tree_keys = (manta_keys_read.shard_trees as Array<string>).map((e)=>{
        return convert_single_map_keys_hex(e, 1);
    });
    if(new_shard_tree_keys.length !== (shard_trees as Array<string>).length)
        throw "shard tree keys and value are not the same len";
    const new_shard_tree_kvs = new_shard_tree_keys.map((e, i)=>{
        return [e, shard_trees[i]]
    });
    await insert_values(context, new_shard_tree_kvs);

    // inserting new utxo_acc_outputs
    const new_utxo_acc_output_keys = (manta_keys_read.utxo_acc_outputs as Array<string>).map((e)=>{
        return convert_single_map_keys_hex(e, 32);
    });
    const new_utxo_acc_output_kvs = new_utxo_acc_output_keys.map((e)=>{
        return [e, '0x'];
    });
    await insert_values(context, new_utxo_acc_output_kvs);
   
    // inserting new utxo_set
    const new_utxo_set_keys = (manta_keys_read.utxo_set as Array<string>).map((e)=>{
        return convert_single_map_keys_hex(e, 32);
    });
    const new_utxo_set_kvs = new_utxo_set_keys.map((e)=>{
        return [e, '0x'];
    })
    await insert_values(context, new_utxo_set_kvs);

    // inserting void number set
    const new_void_number_set_keys = (manta_keys_read.void_number_set as Array<string>).map((e)=>{
        return convert_single_map_keys_hex(e, 32);
    });
    const new_void_number_set_kvs = new_void_number_set_keys.map((e)=>{
        return [e, '0x'];
    });
    await insert_values(context, new_void_number_set_kvs);

    // insert void_number_set_insertion_order
    const new_void_number_set_insertion_order_keys = (manta_keys_read.void_number_set_insertion_order as Array<string>).map((e)=>{
        return convert_single_map_keys_hex(e, 8);
    });
    if (new_void_number_set_insertion_order_keys.length !== (void_number_set_insertion_order as Array<string>).length)
        throw "void number set insertion order size unmatch";
    const new_void_number_set_insertion_order_kvs = new_void_number_set_insertion_order_keys.map((e, i)=>{
        return [e, void_number_set_insertion_order[i]];
    });
    await insert_values(context, new_void_number_set_insertion_order_kvs);
}   

main().catch(console.error).finally(() => process.exit());