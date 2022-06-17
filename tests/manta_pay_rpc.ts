import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { KeyringPair } from '@polkadot/keyring/types';
import { u8aToHex, hexToU8a, u8aToBigInt }from '@polkadot/util';
import { single_map_storage_key, double_map_storage_key, delay, HashType} from './test-util';

// number of shards at MantaPay
const SHARD_NUMBER: number = 256;
const PULL_MAX_SENDER_UPDATE_SIZE: number = 1024;
const PULL_MAX_PER_SHARD_UPDATE_SIZE: number = 128;

// delay time: 40 sec to be safe
const TX_DELAY_TIME = 40000;
const performance = require('perf_hooks').performance;

// generate utxo deterministically using shard_idx and utxo_idx
function generate_utxo(shard_idx: number, utxo_idx: number): Uint8Array {
    const utxo = new Uint8Array(32+32+68);
    utxo.fill((shard_idx + utxo_idx) % 256);
    return utxo;
}

// generate void number deterministically using index
function generate_void_number(index: number): Uint8Array {
    const void_number = new Uint8Array(32);
    void_number.fill(index % 256);
    return void_number;
}

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

// insert certain amount utxos per shard, return a checkpoint 
async function insert_utxos(
    api: ApiPromise, 
    keyring: KeyringPair,
    per_shard_amount: number, 
    checkpoint: Array<number> ): Promise<Array<number>>
{
    if (checkpoint.length !== SHARD_NUMBER) {
        throw "Checkpoint of wrong size";
    }

    // generate utxo data
    var data = [];
    for(var shard_idx = 0; shard_idx < SHARD_NUMBER; ++ shard_idx) {
        for(var utxo_idx  = checkpoint[shard_idx]; utxo_idx < per_shard_amount; ++ utxo_idx) {
            const shards_storage_key = double_map_storage_key(
                "MantaPay", "Shards", shard_idx, 8, HashType.Identity, utxo_idx, 64, HashType.Identity);
            let value_str = u8aToHex(generate_utxo(shard_idx, utxo_idx));
            data.push([shards_storage_key, value_str]);
        }
    }
    let call_data = api.tx.system.setStorage(data);
    await api.tx.sudo.sudo(call_data).signAndSend(keyring, ({ events = [], status }) => {});
    return next_checkpoint(per_shard_amount, checkpoint);
}

// insert a certain amount of void numbers
async function insert_void_numbers(
    api:ApiPromise, keyring: KeyringPair, amount: number, start_index: number): Promise<number>{
    if (amount % PULL_MAX_SENDER_UPDATE_SIZE != 0) {
        throw "Senders amount must be a multiple of PULL_MAX_SENDER_UPDATE_SIZE(1024)";
    }

   var data = [];
   for(var sender_idx = start_index; sender_idx < start_index + amount; ++sender_idx) {
        const vn_storage_key = single_map_storage_key("MantaPay", "VoidNumberSetInsertionOrder", sender_idx, 64, HashType.Identity);
        data.push([vn_storage_key, u8aToHex(generate_void_number(sender_idx))]);
   }
   let call_data = api.tx.system.setStorage(data);
   await api.tx.sudo.sudo(call_data).signAndSend(keyring, ({ events = [], status }) => {});
   return start_index + amount;
}

async function check_full_sync_order_and_performance(api:ApiPromise) {
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

async function setup_storage(api:ApiPromise) {
    const keyring = new Keyring({ type: 'sr25519' });
    const sudo_key_pair = keyring.addFromMnemonic('bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice');
    
    const batch_number = 32;
    const batch_amount = 16;
    
    console.log(">>>> Inserting UTXOs: %i per shard, %i batches", batch_amount, batch_number);
    var receiver_checkpoint = new Array<number>(SHARD_NUMBER);
    receiver_checkpoint.fill(0);
    for(var batch_idx = 0; batch_idx < batch_number; batch_idx ++){
        receiver_checkpoint = await insert_utxos(api, sudo_key_pair, batch_amount, receiver_checkpoint);
        console.log(receiver_checkpoint);
        console.log(">>>> batch %i done.", batch_idx);
        await delay(TX_DELAY_TIME);
    }
    console.log(">>>> Complete inserting UTXOs")

    const vn_amount = PULL_MAX_SENDER_UPDATE_SIZE * batch_number; // Same amount as UTXOs
    for(var batch_idx = 0; batch_idx < batch_number; ++batch_idx) {
        console.log(">>>> Inserting void numbers: %i in total, batch: %i ", PULL_MAX_SENDER_UPDATE_SIZE, batch_idx);
        await insert_void_numbers(api, sudo_key_pair, PULL_MAX_SENDER_UPDATE_SIZE, batch_idx * PULL_MAX_SENDER_UPDATE_SIZE);
        await delay(TX_DELAY_TIME);
    }
    console.log(">>>> Complete inserting void numbers");
}

async function check_single_pull_performance(api:ApiPromise) {
    const before_rpc = performance.now();
    var receiver_checkpoint = new Array<number>(SHARD_NUMBER);
    receiver_checkpoint.fill(0);
    let payload = await (api.rpc as any).mantaPay.pull_ledger_diff({receiver_index: new Array<number>(SHARD_NUMBER).fill(0), sender_index: 0});
    const after_rpc = performance.now();
    console.log("single rpc sync time: %i ms", after_rpc - before_rpc);
}

async function main(){
    let nodeAddress = "ws://127.0.0.1:9801";
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

    
    //await setup_storage(api);
    await check_single_pull_performance(api);
    await check_full_sync_order_and_performance(api);

    console.log("Success!");
}

main().catch(console.error).finally(() => process.exit());
