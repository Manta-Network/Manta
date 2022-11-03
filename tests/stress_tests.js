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
    // pub const COINS_SIZE: usize = 48802; // 100 v1
    // pub const COINS_SIZE: usize = 488002; // 1000 v1
    // pub const COINS_SIZE: usize = 4880002; // 10k v1
    // pub const COINS_SIZE: usize = 48800004; // 100k v1
    // 424002 v2 s
    // 4240002 v2 10k
    // 349 v0
    // 488 v1
    // 424 v2
    const api = await createPromiseApi(nodeAddress);
    let sender = keyring.addFromMnemonic("bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice");
    let receiver = keyring.addFromMnemonic("bottom drive obey lake curtain smoke basket hold race lonely fit walk//Bob");
    let offset = 2;
    let coin_size = 349;
    let coins_count = 1000;
    let coins_count_per_batch = 30;
    var buffer = Buffer.alloc(coin_size * coins_count);
    fs.open('/home/georgi/Desktop/workspace/Manta-Network/Manta/precomputed_mints_v2_10000', 'r',  async  function(status, fd) {
        fs.read(fd, buffer, 0, coin_size * coins_count, offset + 0 * coins_count_per_batch * coin_size, async function(err, num) {
        });
        let batches_sent = 0;
        let start = 0;
        let end = start + coin_size;
        for(let k = 0; k < coins_count / coins_count_per_batch; ++k){
            let mints = [];
            for(let i = 0; i < coins_count_per_batch; ++i){
                const mint = await api.tx.mantaPay.toPrivate(buffer.subarray(start,end));
                mints.push(mint);
                start = end;
                end += coin_size;
            }

            const batch = await api.tx.utility.forceBatch(mints);
            const unsub = await batch.signAndSend(sender, {nonce: -1});
            batches_sent++;
            console.log("\n Batches sent: ",batches_sent);
            if (batches_sent % 25 == 0) {
                await new Promise(resolve => setTimeout(resolve, 180000));
            }
        }

        console.log("\n Batches sent: ",batches_sent);

        if (status) {
            console.log(status.message);
            return;
        }
    });
}

main().catch(console.error);
