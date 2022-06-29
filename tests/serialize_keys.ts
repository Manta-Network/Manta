import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { manta_pay_types, rpc_api } from './types';
import { readFile, writeFile } from 'fs/promises';
import { xxhashAsU8a } from '@polkadot/util-crypto';
import { u8aToHex } from '@polkadot/util';
import { StoragePrepareConfig, setup_storage, manta_pay_config} from './manta_pay';
import minimist, { ParsedArgs } from 'minimist';

const dolphin_config = {
    ws_address: "wss://ws.rococo.dolphin.engineering"
}

async function main(){
    const wsProvider = new WsProvider(dolphin_config.ws_address);

    const api = await ApiPromise.create({ 
        provider: wsProvider,
        types: manta_pay_types,
        rpc: rpc_api});
    
    // get storage keys 
    let shards = await api.query.mantaPay.shards.keys();
    console.log("Fetched %i keys from Shards", shards.length);
    let shard_trees = await api.query.mantaPay.shardTrees.keys();
    console.log("Fetched %i keys from ShardTrees", shard_trees.length);
    let utxo_acc_outputs = await api.query.mantaPay.utxoAccumulatorOutputs.keys();
    console.log("Fetched %i keys from UtxoAccumulatorOutputs", utxo_acc_outputs.length);
    let utxo_set = await api.query.mantaPay.utxoSet.keys();
    console.log("Fetched %i keys from UtxoSet", utxo_set.length);
    let void_number_set = await api.query.mantaPay.voidNumberSet.keys();
    console.log("Fetched %i keys from VoidNumberSet", void_number_set.length);
    let void_number_set_insertion_order = await api.query.mantaPay.voidNumberSetInsertionOrder.keys();
    console.log("Fetched %i keys from VNSIO", void_number_set_insertion_order.length);
    
    const manta_pay_keys = {
        shards: shards,
        shard_trees: shard_trees,
        utxo_acc_outputs: utxo_acc_outputs,
        utxo_set: utxo_set,
        void_number_set: void_number_set,
        void_number_set_insertion_order: void_number_set_insertion_order,
    };

    var manta_keys_raw = JSON.stringify(manta_pay_keys);
    console.log(manta_keys_raw.length);

    await writeFile('./manta_pay_keys.json', manta_keys_raw);
    console.log("write keys to manta_pay_keys.json");
    const manta_keys_read_raw = await readFile('./manta_pay_keys.json');
    const manta_keys_read = JSON.parse(manta_keys_read_raw.toString());
    console.log(manta_keys_read.shards.length);
    console.log(manta_keys_read.shards[0]);
    console.log(manta_keys_read.shard_trees.length);
    console.log(manta_keys_read.shard_trees[0]);
    console.log(manta_keys_read.utxo_acc_outputs.length);
    console.log(manta_keys_read.utxo_acc_outputs[0]);
    console.log(manta_keys_read.utxo_set.length);
    console.log(manta_keys_read.utxo_set[0]);
    console.log(manta_keys_read.void_number_set.length);
    console.log(manta_keys_read.void_number_set[0]);
    console.log(manta_keys_read.void_number_set_insertion_order.length);
    console.log(manta_keys_read.void_number_set_insertion_order[0]);

    // let transformed_data = [];
    // let test_data = new Uint8Array([193, 0, 0, 0, 0, 0, 0, 0, 0]);
    // console.log(u8aToHex(transform_shard_utxo_keys(test_data)));
    // data.forEach((entry)=>{
    //     let old_storage_key_raw = entry[0].toU8a();
    //     let new_storage_key_raw = new Uint8Array([
    //         ...old_storage_key_raw.slice(0,32),
    //         ...old_storage_key_raw.slice(32,)
    //     ])
    // });
    // console.log(data[0][0]);
    // console.log(data[0][0].toHuman());
    // console.log(data[0][0].toU8a());
    // console.log(data[0][0].toU8a().slice(32,));
    // //console.log(data[0][1].toU8a());
}

main().catch(console.error).finally(() => process.exit());