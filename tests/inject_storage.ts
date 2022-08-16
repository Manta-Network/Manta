import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { manta_pay_types, rpc_api } from './types';
import { readFile } from 'fs/promises';
import { ExecutionContext, emojis, delay } from './test-util';

const dolphin_config = {
    ws_address: "ws://127.0.0.1:9801"
}
// "ws://127.0.0.1:9801"
// "wss://ws.rococo.dolphin.engineering"

async function insert_value_batches(
    context: ExecutionContext,    
    kvs: Array<String[]>, 
    batch_size: number,
    timeout: number,
){
    let success_batch = 0;
    let expected_batch = Math.ceil(kvs.length/batch_size);
    for(let check_point = 0;  check_point < kvs.length; ){
        let finish_point = check_point + batch_size > kvs.length ? kvs.length : check_point + batch_size;
        let data = kvs.slice(check_point, finish_point);
        let manta_pay_data = [];
        for(let i =0; i < finish_point - check_point; i ++){
            if ( data[i][0].includes("0xa66d1aecf") ) {
                manta_pay_data.push(data[i]);
                console.log("\n next: ", data[i]);
            }
        }
        let call_data = context.api.tx.system.setStorage(manta_pay_data);
        const unsub = await context.api.tx.sudo.sudo(call_data).signAndSend(context.keyring, {nonce: -1}, ({ events = [], status }) => {
            if (status.isFinalized) {
                success_batch ++;
                console.log("%s %i batches insertion finalized.", emojis.write, success_batch);
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
    kvs: Array<String[]>, 
    batch_size: number = 4096,
    batch_count_before_gap: number = 4,
    timeout_for_big_batch: number = 1000, 
){
    const big_batch_size = batch_size * batch_count_before_gap;
    for(let check_point = 0; check_point < kvs.length; ){
        let finish_point = check_point + big_batch_size > kvs.length ? kvs.length : check_point + big_batch_size;
        console.log(">>>>>> writing big batch from %i", check_point);
        await insert_value_batches(context, kvs.slice(check_point, finish_point), batch_size, timeout_for_big_batch);
        check_point = finish_point;
    }
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
    const kvs_raw = await readFile('./dolphin-storage.json');
    const kvs_raw_read = JSON.parse(kvs_raw.toString());

    await insert_values(context, kvs_raw_read);
}   

main().catch(console.error).finally(() => process.exit()); 