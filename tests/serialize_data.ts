import { ApiPromise, WsProvider } from '@polkadot/api';
import { manta_pay_types, rpc_api } from './types';
import { readFile, writeFile } from 'fs/promises';
import { StorageData } from '@polkadot/types/interfaces';

const dolphin_config = {
    ws_address: "wss://ws.rococo.dolphin.engineering"
}

async function getStorageValuesByBatch(api: ApiPromise, keys: Array<string>, batch_size: number = 4096) {
    let storage_values: Array<StorageData> = [];
    for(let remains = keys.length; remains > 0; ){
        console.log("start a batched get, remains: %i", remains);
        let true_size = remains > batch_size? batch_size : remains;
        let key_batch = keys.slice(keys.length - remains, keys.length - (remains - true_size));
        let data = await api.rpc.state.queryStorageAt(key_batch);
        (data as Array<StorageData>).forEach((value)=>{
            storage_values.push(value);
        });
        remains -= true_size;
    }
    return storage_values;
}

async function main(){
    const wsProvider = new WsProvider(dolphin_config.ws_address);

    const api = await ApiPromise.create({ 
        provider: wsProvider,
        types: manta_pay_types,
        rpc: rpc_api});
    
    let manta_keys_raw = await readFile('./manta_pay_keys.json');
    let manta_keys = JSON.parse(manta_keys_raw.toString());
    let shards = await getStorageValuesByBatch(api, manta_keys.shards);
    let shard_trees = await getStorageValuesByBatch(api, manta_keys.shard_trees);
    let void_number_set_insertion_order = await getStorageValuesByBatch(api, manta_keys.void_number_set_insertion_order);
    await writeFile('./shards.json', JSON.stringify(shards));
    await writeFile('./shards_trees.json', JSON.stringify(shard_trees));
    await writeFile('./void_number_set_insertion_order.json', JSON.stringify(void_number_set_insertion_order));
    console.log("ledger data serialized.");
}

main().catch(console.error).finally(() => process.exit());