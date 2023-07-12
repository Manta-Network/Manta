const fs = require('fs').promises;
const { Keyring } = require('@polkadot/keyring');
const { ApiPromise, WsProvider } = require('@polkadot/api');

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
    const sender = keyring.addFromMnemonic("bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice");

    const mints_file = './data/precomputed_mints';
    const transfers_file = './data/precomputed_transfers';
    const reclaims_file = './data/precomputed_reclaims';

    const mints_buffer = await fs.readFile(mints_file);
    const transfers_buffer = await fs.readFile(transfers_file);
    const reclaims_buffer = await fs.readFile(reclaims_file);

    const mints_offset = 2;
    const transfers_offset = 4;
    const reclaims_offset = 4;
    const total_iterations = 14000;
    const mint_size = 553;
    const transfer_size = 1291;
    const reclaim_size = 1001;

    let batches_sent = 0;
    const transactions = [];

    for (let i = 0; i < total_iterations; i++) {
        const mints_start = mints_offset + i * mint_size;
        const mint = api.tx.mantaPay.toPrivate(mints_buffer.subarray(mints_start, mints_start + mint_size));
        transactions.push(mint);

        const transfers_start = transfers_offset + i * (2 * mint_size + transfer_size);
        const transfer_mint_1 = api.tx.mantaPay.toPrivate(transfers_buffer.subarray(transfers_start, transfers_start + mint_size));
        const transfer_mint_2 = api.tx.mantaPay.toPrivate(transfers_buffer.subarray(transfers_start + mint_size, transfers_start + 2 * mint_size));
        const transfer = api.tx.mantaPay.privateTransfer(transfers_buffer.subarray(transfers_start + 2 * mint_size, transfers_start + 2 * mint_size + transfer_size));
        transactions.push(transfer_mint_1);
        transactions.push(transfer_mint_2);
        transactions.push(transfer);

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
                console.log(`tx %i ${status.type}.`, status.nonce);
            }
        });

        await new Promise(resolve => setTimeout(resolve, 12000));
        transactions.length = 0;

        const reclaims_start = reclaims_offset + i * (2 * mint_size + reclaim_size);
        const reclaim_mint_1 = api.tx.mantaPay.toPrivate(reclaims_buffer.subarray(reclaims_start, reclaims_start + mint_size));
        const reclaim_mint_2 = api.tx.mantaPay.toPrivate(reclaims_buffer.subarray(reclaims_start + mint_size, reclaims_start + 2 * mint_size));
        const reclaim = api.tx.mantaPay.toPublic(reclaims_buffer.subarray(reclaims_start + 2 * mint_size, reclaims_start + 2 * mint_size + reclaim_size));
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
                console.log(`tx %i ${status.type}.`, status.nonce);
            }
        });

        batches_sent++;
        console.log("Batches sent: ", batches_sent);
        await new Promise(resolve => setTimeout(resolve, 12000));
        transactions.length = 0;
    }
}

main().catch(console.error);
