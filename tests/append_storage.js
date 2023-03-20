const fs = require('fs');

// MantaPay, AssetManager and Assets storage items
let prefixes = [
'0x682a59d51ab9e48a8c8cc418ff9708d2',
'0x4ae7e256f92e5888372d72f3e4db1003',
'0xa66d1aecfdbd14d785a4d1d8723b4beb'
];

const { Keyring } = require('@polkadot/keyring');
const { ApiPromise, WsProvider } = require('@polkadot/api');

const keyring = new Keyring({
    type: 'sr25519'
});
const nodeAddress = "wss://ws.calamari.systems";

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
let ind = 3;
  const blockHash = '0x44d0ab61506166043072f0be3fd5f634f4d2f22dfdbbd89beb0bb3908efaa016';
  const { block } = await api.rpc.chain.getBlock(blockHash);
  console.log('extrinsic:', JSON.stringify(block.extrinsics[ind].toHuman(), null, 2));
  const queryFeeDetails = await api.rpc.payment.queryFeeDetails(block.extrinsics[ind].toHex(), blockHash);
  console.log('queryFeeDetails:', JSON.stringify(queryFeeDetails.toHuman(), null, 2));
  const queryInfo = await api.rpc.payment.queryInfo(block.extrinsics[ind].toHex(), blockHash);
  console.log('queryInfo:', JSON.stringify(queryInfo.toHuman(), null, 2));

  // let storagePath = './data/storage.json';
  // let forkPath = './data/fork.json';

  // let storage = JSON.parse(fs.readFileSync(storagePath, 'utf8'));
  // let forkedSpec = JSON.parse(fs.readFileSync(forkPath, 'utf8'));

  // // Grab the items to be moved, then iterate through and insert into storage
  // storage
  // .filter((i) => prefixes.some((prefix) => i[0].startsWith(prefix)))
  // .forEach(([key, value]) => (forkedSpec.genesis.raw.top[key] = value));

  // fs.writeFileSync(forkPath, JSON.stringify(forkedSpec, null, 4));

  process.exit();
}

main().catch(console.error);
