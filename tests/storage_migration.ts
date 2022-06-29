import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { manta_pay_types, rpc_api } from './types';
import { xxhashAsU8a } from '@polkadot/util-crypto';
import { readFile, writeFile } from 'fs/promises';
import { u8aToHex, hexToU8a } from '@polkadot/util';
import { StoragePrepareConfig, setup_storage, manta_pay_config} from './manta_pay';
import { StorageData } from '@polkadot/types/interfaces';

const dolphin_config = {
    ws_address: "wss://ws.rococo.dolphin.engineering"
}

function transform_shard_utxo_keys(data: Uint8Array): Uint8Array{
    let shard_idx_data = data.slice(0, 1);
    let utxo_idx_data = data.slice(1,);
    return new Uint8Array([
        ...xxhashAsU8a(shard_idx_data, 64),
        ...shard_idx_data,
        ...xxhashAsU8a(utxo_idx_data.reverse(), 64),
        ...utxo_idx_data.reverse()
    ]);
}

async function main(){
    const wsProvider = new WsProvider(dolphin_config.ws_address);

    const api = await ApiPromise.create({ 
        provider: wsProvider,
        types: manta_pay_types,
        rpc: rpc_api});
    
    const keyring = new Keyring({ type: 'sr25519' });
    const sudo_key_pair = keyring.addFromMnemonic('bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice');
    const storage_prepare_config: StoragePrepareConfig = {
        utxo_batch_number: 4,
        utxo_batch_size_per_shard: 16,
        utxo_big_batch_number: 1,
        vn_batch_number: 2,
        vn_batch_size: 4096,
    }

    const manta_keys_read_raw = await readFile('./manta_pay_keys.json');
    const manta_keys_read = JSON.parse(manta_keys_read_raw.toString());
    const shards_raw = await readFile('./shards.json');
    const shards = JSON.parse(shards_raw.toString());
    console.log((shards[0] as Uint8Array).slice());
    console.log(u8aToHex(shards[0] as Uint8Array));
    // const shard_trees_raw = await readFile('./shards_trees.json');
    // const shard_trees = JSON.parse(shard_trees_raw.toString());
    // const void_number_set_insertion_order_raw = await readFile('./void_number_set_insertion_order.json');
    // const void_number_set_insertion_order = JSON.parse(void_number_set_insertion_order_raw.toString());
    //console.log("shards:");
    //console.log(u8aToHex(shards[0]));
    //console.log(u8aToHex(shards[(shards as Array<string>).length - 1]));
    // console.log("shard_trees:");
    // console.log(u8aToHex(shard_trees[0]));
    // console.log(u8aToHex(shard_trees[(shard_trees as Array<string>).length - 1]));
    // console.log("void_number_set_insertion_order:");
    // console.log(u8aToHex(void_number_set_insertion_order[0]));
    // console.log(u8aToHex(void_number_set_insertion_order[(void_number_set_insertion_order as Array<string>).length - 1]));
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