import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { KeyringPair } from '@polkadot/keyring/types';
import { u8aToHex }from '@polkadot/util';
import { single_map_storage_key, double_map_storage_key, delay, HashType} from './test-util';

// number of shards at MantaPay
const SHARD_NUMBER: number = 256;

// delay time: 30 sec to be safe
const BLOCK_TIME = 30000;

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

    // generate uxto data
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
    const new_checkpoint = new Array<number>(SHARD_NUMBER);
    for(var i = 0; i < SHARD_NUMBER; ++i){
        new_checkpoint[i] = checkpoint[i] + per_shard_amount;
    }
    return new_checkpoint;
}

// insert a certain amount of void numbers
async function insert_void_numbers(
    api:ApiPromise, keyring: KeyringPair, amount: number, start_index: number): Promise<number>{
   var data = [];
   for(var i = start_index; i < amount; i++) {
        const vn_storage_key = single_map_storage_key("MantaPay", "VoidNumberSetInsertionOrder", i, 64, HashType.Identity);
        data.push([vn_storage_key, u8aToHex(generate_void_number(i))]);
   }
   let call_data = api.tx.system.setStorage(data);
   await api.tx.sudo.sudo(call_data).signAndSend(keyring, ({ events = [], status }) => {});
   return start_index + amount;
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

    const keyring = new Keyring({ type: 'sr25519' });
    const sudo_key_pair = keyring.addFromMnemonic('bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice');
    
    const batch_number = 32;
    const batch_amount = 16;
    
    console.log(">>>> Inserting UTXOs: %i per shard, %i batchs", batch_amount, batch_number);
    var checkpoint = new Array<number>(SHARD_NUMBER);
    checkpoint.fill(0);
    for(var batch_idx = 0; batch_idx < batch_number; batch_idx ++){
        checkpoint = await insert_utxos(api, sudo_key_pair, batch_amount, checkpoint);
        console.log(checkpoint);
        console.log(">>>> batch %i done.", batch_idx);
        await delay(BLOCK_TIME);
    }
    console.log(">>>> Complete inserting UTXOs")
    
    await delay(BLOCK_TIME);

    const vn_amount = 1024;
    console.log(">>>> Inserting void nubmers: %i in total", vn_amount);
    await insert_void_numbers(api, sudo_key_pair, vn_amount, 0);
    console.log(">>>> Complete inserting void numbers");

    const before_rpc = performance.now();
    await (api.rpc as any).mantaPay.pull_ledger_diff({receiver_index: new Array(256).fill(0), sender_index: 0});
    const after_rpc = performance.now();
    console.log("rpc time: %i ms", after_rpc - before_rpc);
}

main().catch(console.error).finally(() => process.exit());
