const fs = require('fs').promises;
const { Keyring } = require('@polkadot/keyring');
const { ApiPromise, WsProvider } = require('@polkadot/api');

const keyring = new Keyring({
    type: 'sr25519'
});
const nodeAddress = "ws://127.0.0.1:9988";

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

    const mints_file = './data/mantaSbt_mints';

    const mints_buffer = await fs.readFile(mints_file);

    const mints_offset = 4;
    const total_iterations = 20;
    const mint_size = 553;

    const transactions = [];

    // forceToPrivate
    // set next id to how many coins
    for (let i = 0; i < total_iterations; i++) {
        const mints_start = mints_offset + i * mint_size;
        const mint = api.tx.mantaSbt.forceToPrivate(
            mints_buffer.subarray(mints_start, mints_start + mint_size),
            0,
            '123',
        'dmyLrsKhggNctvRPNWWyAgAhvtr8JrnsRBdUwoKzLkdyc2cza');
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
                console.log(`tx %i ${status.type}.`, status.nonce);
            }
        });

        await new Promise(resolve => setTimeout(resolve, 12000));
        transactions.length = 0;
    }

    // to_private
    // reserve_account, and after 4 mint, do it again
    for (let i = 0; i < total_iterations; i++) {
        const mints_start = mints_offset + i * mint_size;
        const mint = api.tx.mantaSbt.toPrivate(
            null,
            null,
            null,
            mints_buffer.subarray(mints_start, mints_start + mint_size),
            '123',
        );
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
                console.log(`tx %i ${status.type}.`, status.nonce);
            }
        });

        await new Promise(resolve => setTimeout(resolve, 12000));
        transactions.length = 0;
    }

    // mint_sbt_eth
    // allowlist account
    for (let i = 0; i < total_iterations; i++) {
        const mints_start = mints_offset + i * mint_size;
        const mint = api.tx.mantaSbt.mintSbtEth(
            mints_buffer.subarray(mints_start, mints_start + mint_size),
            0,
            'sig',
            0,
            0,
            0,
            '123',
        );
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
                console.log(`tx %i ${status.type}.`, status.nonce);
            }
        });

        await new Promise(resolve => setTimeout(resolve, 12000));
        transactions.length = 0;
    }

    // force_mint_sbt_eth
    for (let i = 0; i < total_iterations; i++) {
        const mints_start = mints_offset + i * mint_size;
        const mint = api.tx.mantaSbt.forceMintSbtEth(
            mints_buffer.subarray(mints_start, mints_start + mint_size),
            0,
            'sig',
            0,
            0,
            0,
            '123',
        );
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
                console.log(`tx %i ${status.type}.`, status.nonce);
            }
        });

        await new Promise(resolve => setTimeout(resolve, 12000));
        transactions.length = 0;
    }

    await api.disconnect();
}

main().catch(console.error);
