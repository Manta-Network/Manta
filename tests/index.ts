import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { U128, u128, u8 } from '@polkadot/types';
import { xxhashAsHex } from '@polkadot/util-crypto';


// Construct
const wsProvider = new WsProvider('ws://127.0.0.1:9800');

function twox_128(input: any) {
    return xxhashAsHex(input, 128);
}

function shard_prefix(shard_index: u8, element_index: u128){
    return twox_128('MantaPay') + twox_128('Shards') + shard_index.toString(16) + element_index.toString(16);
}

async function main(){
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
    const keyring = new Keyring();
    const sudo_key_pair = keyring.createFromUri('//Alice');
    
    var shard_index: U8 = 0;
    var element_index: U128 = 0;
    const zero_shard_prefix = shard_prefix(shard_index, element_index);
    console.log(zero_shard_prefix);
    const content = await api.rpc.state.getStorage(zero_shard_prefix);
    console.log(content);
    const proposal =  await api.tx.system.setStorage([
        [ zero_shard_prefix,
          `0x83590b405cf760cb1660fc295f7810d428fb27d946f2bba38cb9ca5b7d4ed643c5e56ae65158f96c93573210b6a0f36eadf01166b77dbe49247947f669daa1225e11b47dd076bf70568bd8d9ceb93a90e49ba1ce0a651f2a0107364da1d2f018776494b592a8eb26b8af06fb56e681e3efadd4d23f12eedac960fdeb455f66fbeb0967bf`]
      ]);
    console.log(proposal);
    
    const payload = await (api.rpc as any).mantaPay.pull_ledger_diff({receiver_index: new Array(256).fill(0), sender_index: 0});
    console.log(payload);
}

main().catch(console.error).finally(() => process.exit());
