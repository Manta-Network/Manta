import { ApiPromise, WsProvider } from '@polkadot/api';

// Construct
const wsProvider = new WsProvider('ws://127.0.0.1:9800');
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
                checkpoint: 'Checkpoint',
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
    const payload = await (api.rpc as any).mantaPay.pull_ledger_diff({receiver_index: new Array(256).fill(0), sender_index: 0});
    console.log(payload);
}

main().catch(console.error).finally(() => process.exit());
