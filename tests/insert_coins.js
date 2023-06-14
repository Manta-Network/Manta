const fs = require('fs').promises;
const { Keyring } = require('@polkadot/keyring');
const { ApiPromise, WsProvider } = require('@polkadot/api');

const keyring = new Keyring({
    type: 'sr25519'
});
const nodeAddress = "wss://c1.manta.systems";

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

    const blockHash = '0x8c62a343e109c23a5445fc61f98a6d90ee6ec91c884f0c3d5b8d7bb8b2dd3891';
    const { block } = await api.rpc.chain.getBlock(blockHash);
    console.log('extrinsic:', JSON.stringify(block.extrinsics[3].toHuman(), null, 2));
    console.log('extrinsic:', JSON.stringify(block.extrinsics[3].length, null, 2));
    // const queryFeeDetails = await api.rpc.payment.queryFeeDetails(block.extrinsics[3].toHex(), blockHash);
    // console.log('queryFeeDetails:', JSON.stringify(queryFeeDetails.toHuman(), null, 2));
    // const queryInfo = await api.rpc.payment.queryInfo(block.extrinsics[3].toHex(), blockHash);
    // console.log('queryInfo:', JSON.stringify(queryInfo.toHuman(), null, 2));
}

main().catch(console.error);
