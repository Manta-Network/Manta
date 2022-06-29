import { ApiPromise, WsProvider } from '@polkadot/api';
import { manta_pay_types, rpc_api } from './types';
import { readFile, writeFile } from 'fs/promises';

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
}

main().catch(console.error).finally(() => process.exit());