import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { KeyringPair } from '@polkadot/keyring/types';
import { u8aToHex }from '@polkadot/util';
import { single_map_storage_key, double_map_storage_key, delay, emojis, HashType} from './test-util';

// number of shards at MantaPay
const SHARD_NUMBER: number = 256;
const PULL_MAX_SENDER_UPDATE_SIZE: number = 1024;
const PULL_MAX_PER_SHARD_UPDATE_SIZE: number = 128;

// generate utxo deterministically using shard_idx and utxo_idx
function generate_utxo(shard_idx: number, utxo_idx: number): Uint8Array {
    const utxo = new Uint8Array(32+32+68);
    let mod = (shard_idx + utxo_idx) % 256;
    // we need to avoid fill all 0s, since all 0s means empty
    var element = ( mod === 0 ? 42 : mod );
    utxo.fill(element);
    return utxo;
}

// generate void number deterministically using index
function generate_void_number(index: number): Uint8Array {
    const void_number = new Uint8Array(32);
    void_number.fill(index % 256);
    return void_number;
}

// move checkpoint to next 
function next_checkpoint(
    per_shard_amount: number, 
    checkpoint: Array<number> ): Array<number>
{
    if (checkpoint.length !== SHARD_NUMBER) {
        throw "Checkpoint of wrong size";
    }

    const new_checkpoint = new Array<number>(SHARD_NUMBER);
    for(var i = 0; i < SHARD_NUMBER; ++i){
        new_checkpoint[i] = checkpoint[i] + per_shard_amount;
    }

    return new_checkpoint;
}

// generate a batch of utxo insertions from a checkpoint
function generate_utxo_data(per_shard_amount: number, checkpoint: Array<number>){
    var data = [];
    for(var shard_idx = 0; shard_idx < SHARD_NUMBER; ++ shard_idx) {
        for(var utxo_idx  = checkpoint[shard_idx]; utxo_idx < checkpoint[shard_idx] + per_shard_amount; ++ utxo_idx) {
            const shards_storage_key = double_map_storage_key(
                "MantaPay", "Shards", shard_idx, 8, HashType.Identity, utxo_idx, 64, HashType.Identity);
            let value_str = u8aToHex(generate_utxo(shard_idx, utxo_idx));
            data.push([shards_storage_key, value_str]);
        }
    }
    const new_checkpoint = next_checkpoint(per_shard_amount, checkpoint); 
    return {data: data, checkpoint: new_checkpoint};
}

// insert certain amount utxos per shard
// return successfully inserted batch number
async function insert_utxos(
    api: ApiPromise, 
    keyring: KeyringPair,
    batch_number: number,
    per_shard_amount: number, 
    init_checkpoint: Array<number>,
    max_wait_time_sec: number ): Promise<number>
{
    if (init_checkpoint.length !== SHARD_NUMBER) {
        throw "Checkpoint of wrong size";
    }

    var success_batch: number = 0;
    var cur_checkpoint = init_checkpoint;
    for (var batch_idx = 0; batch_idx < batch_number; batch_idx ++){
        var {data, checkpoint} = generate_utxo_data(per_shard_amount, cur_checkpoint);
        cur_checkpoint = checkpoint;
        let call_data = api.tx.system.setStorage(data);
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
    for(var i =0; i < max_wait_time_sec; i ++){
        await delay(1000);
        if (success_batch === batch_number) {
            console.log("total wait: %i sec.", i + 1);
            return success_batch;
        }
    }
    return success_batch;
}

// generate void number data
function generate_vn_data(
    start_index: number,
    amount_per_batch: number): String[][] {
    var data = [];    
    for (var idx = start_index; idx < start_index + amount_per_batch; idx ++){
        const vn_storage_key = single_map_storage_key(
            "MantaPay", "VoidNumberSetInsertionOrder", idx, 64, HashType.Identity);
        data.push([vn_storage_key, u8aToHex(generate_void_number(idx))]);
    }
    return data;
}

// insert certain amount of void numbers in batches
// returns number of success batch
async function insert_void_numbers(
    api:ApiPromise, 
    keyring: KeyringPair, 
    amount_per_batch: number,
    batch_number: number, 
    start_index: number,
    max_wait_time_sec: number): Promise<number>{

    var success_batch = 0;
    var sender_idx = start_index; 
    for(var batch_idx = 0; batch_idx < batch_number; batch_idx ++) {
        console.log("start vn batch %i", batch_idx);
        const data = generate_vn_data(sender_idx, amount_per_batch);
        let call_data = api.tx.system.setStorage(data);
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
    for(var i =0; i < max_wait_time_sec; i++){
        await delay(1000);
        if (success_batch === batch_number) { 
            console.log("total wait: %i sec.", i + 1); 
            return success_batch;
        }
    }
    return success_batch;
}

async function full_sync_performance(api:ApiPromise) {
    const before_rpc = performance.now();

    var should_pull = true;
    let sender_checkpoint = 0;
    var receiver_checkpoint = new Array<number>(SHARD_NUMBER);
    receiver_checkpoint.fill(0);
    while(should_pull) {
        let payload = await (api.rpc as any).mantaPay.pull_ledger_diff({receiver_index: receiver_checkpoint, sender_index: sender_checkpoint});
        let [should_continue, receivers, senders] = payload;

        if(should_continue[1].toString() == 'true') {
            should_pull = true;
        } else {
            should_pull = false;
        }
        
        for(var receiver_index = 0; receiver_index < receivers[1].length; receiver_index++) {
            let [ephemeral_public_key, cipher_text] = receivers[1][receiver_index][1];
            let shard_index = ~~(receiver_index / PULL_MAX_PER_SHARD_UPDATE_SIZE); // integer division
            let utxo_index = receiver_checkpoint[shard_index] + (receiver_index % PULL_MAX_PER_SHARD_UPDATE_SIZE);
            let supposed_next = (shard_index + utxo_index) % 256;
            let actual_next = cipher_text[1][0];
            if(cipher_text[1][0] != supposed_next) {
                throw "UTXOs not in expected order";
            }
        }

        for(var sender_index = sender_checkpoint; sender_index < senders[1].length; sender_index++) {
            let supposed_next = sender_index % 256;
            if(senders[1][sender_index][0] != supposed_next) {
                throw "Void numbers not in expected order";
            }
        }

        receiver_checkpoint = next_checkpoint(PULL_MAX_PER_SHARD_UPDATE_SIZE, receiver_checkpoint);
        sender_checkpoint += PULL_MAX_SENDER_UPDATE_SIZE;
    }

    const after_rpc = performance.now();
    console.log("full rpc sync time: %i ms", after_rpc - before_rpc);
}

async function setup_storage(api:ApiPromise, init_utxo_idx: number) {
    const keyring = new Keyring({ type: 'sr25519' });
    const sudo_key_pair = keyring.addFromMnemonic('bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice');
    
    const utxo_big_batch_number = 1;
    const utxo_batch_number = 4;
    const utxo_per_shard = 16;
    const vn_batch_number = 1;
    const vn_batch_size = 1024;
    
    var receiver_checkpoint = new Array<number>(SHARD_NUMBER);
    var check_idx = init_utxo_idx;
    console.log(">>>>>>>>> UTXO INSERT START >>>>>>>>");
    for (var big_batch_idx = 0; big_batch_idx < utxo_big_batch_number; big_batch_idx ++) {
        console.log(">>>> Inserting %i big batch UTXOs", big_batch_idx + 1);
        receiver_checkpoint.fill(check_idx);
        console.log("starting utxo idx: %i", receiver_checkpoint[0]);
        const utxo_batch_done = await insert_utxos(api, sudo_key_pair, utxo_batch_number, utxo_per_shard, receiver_checkpoint, 1000);
        console.log(">>>> Complete %i big batch with %i UTXOs", big_batch_idx + 1 , utxo_batch_done * utxo_per_shard * SHARD_NUMBER);
    }
    console.log(">>>>>>>>> UTXO INSERT DONE >>>>>>>>");

    console.log(">>>> Inserting void numbers: %i per batch, %i batchs", vn_batch_size, vn_batch_number);
    const vn_batch_done = await insert_void_numbers(api, sudo_key_pair, vn_batch_size, vn_batch_number, 0, 1000);
    console.log(">>>> Complete inserting %i void numbers", vn_batch_done * vn_batch_size);
}

async function single_rpc_performance(api:ApiPromise) {
    const before_rpc = performance.now();
    var receiver_checkpoint = new Array<number>(SHARD_NUMBER);
    receiver_checkpoint.fill(0);
    const data = await (api.rpc as any).mantaPay.pull_ledger_diff({receiver_index: new Array<number>(SHARD_NUMBER).fill(0), sender_index: 0});
    const after_rpc = performance.now();
    console.log("ledger diff receiver size: %i", data.receivers.length);
    console.log("ledger diff void number size: %i", data.senders.length);
    console.log("single rpc sync time: %i ms", after_rpc - before_rpc);
}

async function main(){
    let nodeAddress = "ws://127.0.0.1:9800";
    const args = require('minimist')(process.argv.slice(2))
    if (args.hasOwnProperty('address')) {
      nodeAddress = args['address'];
      console.log("Using passed parameter address: " + nodeAddress);
    }

    const wsProvider = new WsProvider(nodeAddress);

    const api = await ApiPromise.create({ 
        provider: wsProvider,
        types: {
            Checkpoint: {
                receiver_index: '[u64; 256]',
                sender_index: 'u64'
            },
            EncryptedNote: {
                ephemeral_public_key: '[u8; 32]',
                ciphertext: '[u8; 68]'
            },
            PullResponse: {
                should_continue: 'bool',
                receivers: 'Vec<([u8; 32], EncryptedNote)>',
                senders: 'Vec<[u8; 32]>',
            }
        },
        rpc: {
            mantaPay: {
                pull_ledger_diff: {
                    description: 'pull from mantaPay',
                    params: [
                        {
                            name: 'checkpoint',
                            type: 'Checkpoint'
                        }
                    ],
                    type: 'PullResponse'
                }
            }
        }});

    //await setup_storage(api, 64);
    //const block_hash = await api.rpc.chain.getBlockHash()
    //let shards: any[][] = await (api.query as any).mantaPay.shards.entriesAt(block_hash);
    //console.log("shards size: %i", shards.length);
    //await single_rpc_performance(api);
    const block_hash = await api.rpc.chain.getBlockHash();
    var before = performance.now();
    let shards = await ((await api.at(block_hash)).query as any).mantaPay.shards.entries();
    //let vns = await (api.query as any).mantaPay.voidNumberSetInsertionOrder.entries();
    var after = performance.now();
    console.log("shard size: %i", shards.length);
    //console.log("vn size: %i", vns.length);
    console.log("entires time: %i ms", after - before);
    //await check_full_sync_order_and_performance(api);

    //console.log("Success!");
}

main().catch(console.error).finally(() => process.exit());
