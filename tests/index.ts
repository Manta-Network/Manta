import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { U128, u128, u8 } from '@polkadot/types';
import { xxhashAsHex } from '@polkadot/util-crypto';

function twox_128(input: any) {
    return xxhashAsHex(input, 128);
}

const changeEndianness = (string: String) => {
    const result = [];
    let len = string.length - 2;
    while (len >= 0) {
      var substr = string.substr(len, 2);
      if (substr == '') {
          substr += '0';
      }
      result.push(string.substr(len, 2));
      len -= 2;
    }
    return result.join('');
}

function vn_insertion_order_set_storage_key(element_index: bigint) {
    const element_index_hex_len = 16;

    var element_index_hex = element_index.toString(16);
    const element_index_actual = element_index_hex.length;

    for (var i = 0; i < element_index_hex_len - element_index_actual; ++i) {
        element_index_hex = '0' + element_index_hex;
    }

    element_index_hex = changeEndianness(element_index_hex);
    
    return twox_128('MantaPay') + twox_128('VoidNumberSetInsertionOrder').slice(2) + element_index_hex;
}

function shard_storage_key(shard_index: u8, element_index: bigint) {
    const shard_index_hex_len = 2;
    const element_index_hex_len = 16;

    var shard_index_hex = shard_index.toString(16);
    const shard_index_actual = shard_index_hex.length;
    var element_index_hex = element_index.toString(16);
    const element_index_actual = element_index_hex.length;

    for (var i = 0; i < shard_index_hex_len - shard_index_actual; ++i) {
        shard_index_hex = '0' + shard_index_hex;
    }
    for (var i = 0; i < element_index_hex_len - element_index_actual; ++i) {
        element_index_hex = '0' + element_index_hex;
    }

    shard_index_hex = changeEndianness(shard_index_hex);
    element_index_hex = changeEndianness(element_index_hex);
    
    return twox_128('MantaPay') + twox_128('Shards').slice(2) + shard_index_hex + element_index_hex;
}

function wait(ms: number){
    var start = new Date().getTime();
    var end = start;
    while(end < start + ms) {
      end = new Date().getTime();
   }
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

    const keyring = new Keyring({ type: 'sr25519' });
    const sudo_key_pair = keyring.addFromMnemonic('bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice');
    
    const shards_amount = 3;
    const utxo_per_shard_amount = 3;

    for(var i = 0; i < shards_amount; ++i) {
        for(var j = 0; j < utxo_per_shard_amount; ++j) {
            var shard_index: u8 = (i as unknown) as u8;
            var element_in_shard_index: bigint = (j as unknown) as bigint;
            const shards_storage_key = shard_storage_key(shard_index, element_in_shard_index);
            console.log('Shards storage key for shard_index: ' + shard_index + ' and element_index: ' + element_in_shard_index + ' is: ' + shards_storage_key);
    
            const shards_call = api.tx.system.setStorage([
                [ 
                    shards_storage_key,
                    // Insert some random but correct value:
                    '0x83590b405cf760cb1660fc295f7810d428fb27d946f2bba38cb9ca5b7d4ed643c5e56ae65158f96c93573210b6a0f36eadf01166b77dbe49247947f669daa1225e11b47dd076bf70568bd8d9ceb93a90e49ba1ce0a651f2a0107364da1d2f018776494b592a8eb26b8af06fb56e681e3efadd4d23f12eedac960fdeb455f66fbeb0967bf']
                ]
            );
    
            await api.tx.sudo.sudo(shards_call).signAndSend(sudo_key_pair, ({ events = [], status }) => {
                console.log('Call status:', status.type);
                if (status.isInBlock) {
                    console.error('You have just set storage for MantaPay Shards on your chain');
        
                    console.log('Included at block hash', status.asInBlock.toHex());
                    console.log('Events:');
        
                    console.log(JSON.stringify(events, null, 2));
                } else if (status.isFinalized) {
                    console.log('Finalized block hash', status.asFinalized.toHex());
                }
            });
    
            wait(20000);
        }
    }

    const all_entries = shards_amount * utxo_per_shard_amount;
    for(var i = 0; i < all_entries; ++i) {
        var vsio_element_index: bigint = (i as unknown) as bigint;
        const vsio_storage_key = vn_insertion_order_set_storage_key(vsio_element_index);
        console.log('VoidNumberSetInsertionOrder storage key for index: ' + vsio_element_index + ' is: ' + vsio_storage_key);
    
        // Insert some random but correct value:
        const call = api.tx.system.setStorage([[vsio_storage_key, '0xefe34cfd4418c9b1c04e555965e479d00ec4814ed0cd94641df1a8c6f9fa1071']]);
        await api.tx.sudo.sudo(call).signAndSend(sudo_key_pair, ({ events = [], status }) => {
            console.log('Call status:', status.type);
            if (status.isInBlock) {
                console.error('You have just set storage of VoidNumberSetInsertionOrder on your chain');
        
                console.log('Included at block hash', status.asInBlock.toHex());
                console.log('Events:');
        
                console.log(JSON.stringify(events, null, 2));
            } else if (status.isFinalized) {
                console.log('Finalized block hash', status.asFinalized.toHex());
            }
        });
        
        wait(20000);    
    }

    wait(60000);

    const payload = await (api.rpc as any).mantaPay.pull_ledger_diff({receiver_index: new Array(256).fill(0), sender_index: 0});
    console.log(payload.toHuman());
}

main().catch(console.error).finally(() => process.exit());
