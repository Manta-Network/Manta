const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');

const keyring = new Keyring({ type: 'sr25519' });

const fs = require('fs');
const { readFile } = require('fs/promises');

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
    let receiver = keyring.addFromMnemonic("bottom drive obey lake curtain smoke basket hold race lonely fit walk//Bob");

    let mints_offset = 2;
    let transfers_offset = 4;
    let reclaims_offset = 4;
    let total_iterations = 15000;

    let mint_size = 552;
    const mints_content = await readFile("./precomputed_mints");
    const mints_buffer = mints_content.subarray(
      mints_offset,
      mint_size * total_iterations
    );
    let transfer_size = 1290;
    let full_transfer_size = (mint_size * 2 + transfer_size);
    const transfers_content = await readFile("./precomputed_transfers");
    const transfers_buffer = transfers_content.subarray(
      transfers_offset,
      full_transfer_size * total_iterations
    );
    let reclaim_size = 968;
    let full_reclaim_size = (mint_size * 2 + reclaim_size);
    const reclaims_content = await readFile("./precomputed_reclaims");
    const reclaims_buffer = reclaims_content.subarray(
      reclaims_offset,
      full_reclaim_size * total_iterations
    );

    let txs_sent = 0;
    for(let i = 0; i < total_iterations; ++i){
        await api.tx.mantaPay.toPrivate(mints_buffer.subarray(mint_size * i, mint_size * (i + 1))).signAndSend(sender, {nonce: -1});

        let transfers_start = i * (2 * mint_size + transfer_size);
        await api.tx.mantaPay.toPrivate(transfers_buffer.subarray(transfers_start, transfers_start + mint_size)).signAndSend(sender, {nonce: -1});
        await api.tx.mantaPay.toPrivate(transfers_buffer.subarray(transfers_start + mint_size, transfers_start + 2 * mint_size)).signAndSend(sender, {nonce: -1});
        await api.tx.mantaPay.privateTransfer(transfers_buffer.subarray(transfers_start + 2 * mint_size, transfers_start + 2 * mint_size + transfer_size)).signAndSend(sender, {nonce: -1});
        await new Promise(resolve => setTimeout(resolve, 12000));

        let reclaims_start = i * (2 * mint_size + reclaim_size);
        await api.tx.mantaPay.toPrivate(reclaims_buffer.subarray(reclaims_start, reclaims_start + mint_size)).signAndSend(sender, {nonce: -1});
        await api.tx.mantaPay.toPrivate(reclaims_buffer.subarray(reclaims_start + mint_size, reclaims_start + 2 * mint_size)).signAndSend(sender, {nonce: -1});
        await api.tx.mantaPay.toPublic(reclaims_buffer.subarray(reclaims_start + 2 * mint_size, reclaims_start + 2 * mint_size + reclaim_size)).signAndSend(sender, {nonce: -1});
        await new Promise(resolve => setTimeout(resolve, 12000));

        txs_sent += 7;
        console.log("\n Transactions sent: ", txs_sent);
    }
}

main().catch(console.error);
