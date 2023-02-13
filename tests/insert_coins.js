const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');

const keyring = new Keyring({ type: 'sr25519' });

const fs = require('fs');

// Create a promise API instance of the passed in node address.
async function createPromiseApi(nodeAddress) {
    const wsProvider = new WsProvider(nodeAddress);
    const api = await new ApiPromise({ 
        provider: wsProvider
    });
    await api.isReady;
    return api;
}
const mints = [];

async function main() {
    let nodeAddress = "ws://127.0.0.1:9801";
    
    const api = await createPromiseApi(nodeAddress);
    let sender = keyring.addFromMnemonic("bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice");

    const mints_file = '/home/georgi/Desktop/workspace/Manta/precomputed-15k-iterations/precomputed_mints_v3-5';
    const transfers_file = '/home/georgi/Desktop/workspace/Manta/precomputed-15k-iterations/precomputed_transfers_v3-5';
    const reclaims_file = '/home/georgi/Desktop/workspace/Manta/precomputed-15k-iterations/precomputed_reclaims_v3-5';

    let mints_offset = 2;
    let transfers_offset = 4;
    let reclaims_offset = 4;
    let total_iterations = 15000;

    let mint_size = 552;

    const mints_buffer = await fs.promises.readFile(mints_file);
    const transfers_buffer = await fs.promises.readFile(transfers_file);
    const reclaims_buffer = await fs.promises.readFile(reclaims_file);

    let transactions = [];
    for(let i = 0; i < 14000; ++i){
        let mints_start = mints_offset + i * mint_size;
        const mint = await api.tx.mantaPay.toPrivate(mints_buffer.subarray(mints_start, mint_size + mints_start));
        transactions.push(mint);

        let transfers_start = transfers_offset + i * (2 * mint_size + transfer_size);
        const transfer_mint_1 = await api.tx.mantaPay.toPrivate(transfers_buffer.subarray(transfers_start, transfers_start + mint_size));
        const transfer_mint_2 = await api.tx.mantaPay.toPrivate(transfers_buffer.subarray(transfers_start + mint_size, transfers_start + 2 * mint_size));
        const transfer = await api.tx.mantaPay.privateTransfer(transfers_buffer.subarray(transfers_start + 2 * mint_size, transfers_start + 2 * mint_size + transfer_size));
        transactions.push(transfer_mint_1);
        transactions.push(transfer_mint_2);
        transactions.push(transfer);

        await api.tx.utility.forceBatch(transactions).signAndSend(sender, {nonce: -1}, ({ events = [], status }) => {
            if (status.isFinalized) {
                console.log("tx %i success.", status.nonce);
            }
        if (status.isDropped) {
            console.log("tx %i dropped.", status.nonce);
            unsub();
        }
        if (status.isUsurped) {
            console.log("tx %i usurped.", status.nonce);
            unsub();
        }
        if (status.isFinalityTimeout) {
            console.log("tx %i finality timeout.", status.nonce);
            unsub();
        }
        if (status.isRetracted) {
            console.log("tx %i retracted.", status.nonce);
            unsub();
        }
        });
        await new Promise(resolve => setTimeout(resolve, 10000));
        transactions = [];

        let reclaims_start = reclaims_offset + i * (2 * mint_size + reclaim_size);
        const reclaim_mint_1 = await api.tx.mantaPay.toPrivate(reclaims_buffer.subarray(reclaims_start, reclaims_start + mint_size));
        const reclaim_mint_2 = await api.tx.mantaPay.toPrivate(reclaims_buffer.subarray(reclaims_start + mint_size, reclaims_start + 2 * mint_size));
        const reclaim = await api.tx.mantaPay.toPublic(reclaims_buffer.subarray(reclaims_start + 2 * mint_size, reclaims_start + 2 * mint_size + reclaim_size));
        transactions.push(reclaim_mint_1);
        transactions.push(reclaim_mint_2);
        transactions.push(reclaim);
        await api.tx.utility.forceBatch(transactions).signAndSend(sender, {nonce: -1}, ({ events = [], status }) => {
            if (status.isFinalized) {
                console.log("tx %i success.", status.nonce);
            }
        if (status.isDropped) {
            console.log("tx %i dropped.", status.nonce);
            unsub();
        }
        if (status.isUsurped) {
            console.log("tx %i usurped.", status.nonce);
            unsub();
        }
        if (status.isFinalityTimeout) {
            console.log("tx %i finality timeout.", status.nonce);
            unsub();
        }
        if (status.isRetracted) {
            console.log("tx %i retracted.", status.nonce);
            unsub();
        }
        });
        batches_sent++;
        console.log("\n Batches sent: ", batches_sent);
        if (batches_sent % 1 == 0) {
            await new Promise(resolve => setTimeout(resolve, 10000));
        }
        transactions = [];
    }
}

main().catch(console.error);
