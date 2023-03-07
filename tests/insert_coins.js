const fs = require('fs').promises;
const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');

const keyring = new Keyring({
    type: 'sr25519'
});
const nodeAddress = "ws://127.0.0.1:9801";

async function createPromiseApi(nodeAddress) {
    const wsProvider = new WsProvider(nodeAddress);
    const api = await ApiPromise.create({
        provider: wsProvider
    });
    await api.isReady;
    return api;
}

async function main() {
    const api = await createPromiseApi(nodeAddress);
    const sender = keyring.addFromMnemonic("bottom drive obey lake curtain smoke basket hold race lonely fit walk//Charlie");

    const mints_file = '/Users/ghz/workspace/Manta-Network/Manta/precomputed-1-iterations/precomputed_mints_v6';
    // const transfers_file = '/Users/ghz/workspace/Manta-Network/Manta/precomputed-1-iterations/precomputed_transfers_v6';
    const reclaims_file = '/Users/ghz/workspace/Manta-Network/Manta/precomputed-1-iterations/precomputed_reclaims_v6';

    const mints_buffer = await fs.readFile(mints_file);
    // const transfers_buffer = await fs.readFile(transfers_file);
    const reclaims_buffer = await fs.readFile(reclaims_file);

    const mints_offset = 1;
    const transfers_offset = 4;
    const reclaims_offset = 1;
    const total_iterations = 1;
    const mint_size = 553;
    const transfer_size = 100;
    const reclaims_size = 1001;

    let batches_sent = 0;
    const transactions = [];

    for (let i = 0; i < total_iterations; i++) {

        // const reclaims_start = reclaims_offset + i * (2 * mint_size + transfer_size);
        const reclaims_start = reclaims_offset;
        const reclaim_mint_1 = api.tx.mantaPay.toPrivate(reclaims_buffer.subarray(reclaims_start, reclaims_start + mint_size));
        const reclaim_mint_2 = api.tx.mantaPay.toPrivate(reclaims_buffer.subarray(reclaims_start + mint_size, reclaims_start + 2 * mint_size));
        const reclaim = api.tx.mantaPay.toPublic(reclaims_buffer.subarray(reclaims_start + 2 * mint_size, reclaims_start + 2 * mint_size + reclaims_size));
        transactions.push(reclaim_mint_1);
        transactions.push(reclaim_mint_2);
        transactions.push(reclaim);

        await api.tx.utility.forceBatch(transactions).signAndSend(sender, {
            nonce: -1
        }, ({
            events = [],
            status
        }) => {
            if (status.isFinalized) {
                console.log("tx %i success.", status.nonce);
            }
            if (status.isDropped || status.isUsurped || status.isFinalityTimeout || status.isRetracted) {
                console.err(`tx %i ${status.type}.`, status.nonce);
            }
        });

        await new Promise(resolve => setTimeout(resolve, 10000));
        transactions.length = 0;

        const mints_start = mints_offset + i * mint_size;
        const mint = api.tx.mantaPay.toPrivate(mints_buffer.subarray(mints_start, mint_size + mints_start));
        transactions.push(mint);

        await api.tx.utility.forceBatch(transactions).signAndSend(sender, {
            nonce: -1
        }, ({
            events = [],
            status
        }) => {
            if (status.isFinalized) {
                console.log("tx %i success.", status.nonce);
            }
            if (status.isDropped || status.isUsurped || status.isFinalityTimeout || status.isRetracted) {
                console.err(`tx %i ${status.type}.`, status.nonce);
            }
        });

        await new Promise(resolve => setTimeout(resolve, 10000));
        transactions.length = 0;
    }
}

main().catch(console.error);