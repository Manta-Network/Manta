import { ApiPromise, WsProvider } from '@polkadot/api';
import { BN } from '@polkadot/util';
import { Keyring } from '@polkadot/keyring';
import { farming_rpc_api, farming_types } from './types';
import { expect } from 'chai';
import minimist, { ParsedArgs } from 'minimist';
import {execute_transaction, execute_via_governance,timer } from "./chain-util";

// ./target/release/manta --chain=manta-localdev --alice --ws-port 9800 --rpc-cors all --execution=native
const test_config = {
    ws_address: "ws://127.0.0.1:9800",
    timeout: 2000000
}
const location = {
    V1: {
        parents: 1,
        interior: {
            X3: [
                {
                    Parachain: 1000
                },
                {
                    PalletInstance: 50
                },
                {
                    GeneralIndex: 1984
                }
            ]
        }
    }
};
const location2 = {
    V1: {
        parents: 1,
        interior: {
            X3: [
                {
                    Parachain: 1000
                },
                {
                    PalletInstance: 50
                },
                {
                    GeneralIndex: 1985
                }
            ]
        }
    }
};
const metadata = {
    metadata: {
        name: "Tether USD",
        symbol: "USDT",
        decimals: 6,
        isFrozen: false
    },
    minBalance: 1,
    isSufficient: true
};
const metadata2 = {
    metadata: {
        name: "USDC",
        symbol: "USDC",
        decimals: 10,
        isFrozen: false
    },
    minBalance: 1,
    isSufficient: true
};
const metadata3 = {
    metadata: {
        name: "MANDEX",
        symbol: "MANDEX",
        decimals: 18,
        isFrozen: false
    },
    minBalance: 1,
    isSufficient: true
};
const lp_metadata = {
    metadata: {
        name: "LP-USDC-USDT",
        symbol: "LP",
        decimals: 12,
        isFrozen: false
    },
    minBalance: 1,
    isSufficient: true
};
var referendumIndexObject = { referendumIndex: 0 };

function local_asset(parachainId: number, generalKey: string) {
    let location = {
        V1: {
            parents: 1,
            interior: {
                X2: [
                    {
                        Parachain: parachainId
                    },
                    {
                        GeneralKey: generalKey
                    }
                ]
            }
        }
    };
    return location;
}

describe('Node RPC Test', () => {
    it('Check RPC result', async () => {

        let nodeAddress = "";
        const args: ParsedArgs = minimist(process.argv.slice(2));
        if (args["address"] == null) {
            nodeAddress = test_config.ws_address;
        } else {
            nodeAddress = args["address"];
        }
        console.log("using address %s", nodeAddress);

        const wsProvider = new WsProvider(nodeAddress);
        const api = await ApiPromise.create({provider: wsProvider,
            types: farming_types,
            rpc: farming_rpc_api
        });
        const alice = new Keyring({type: 'sr25519'}).addFromUri("//Alice");

        const parachainId = Number(await api.query.parachainInfo.parachainId());
        console.log("parachain:" + parachainId);

        // register asset 8(decimal:6)
        let callData = api.tx.assetManager.registerAsset(location, metadata);
        await execute_via_governance(api, alice, callData, referendumIndexObject);

        let state: any = await api.query.assetManager.assetIdMetadata(8);
        while(state.isNone) {
            state = await api.query.assetManager.assetIdMetadata(8);
            await timer(3000);
        }
        console.log(new Date() + " Register Asset8:" + JSON.stringify(state));

        // 9(decimal:10)
        callData = api.tx.assetManager.registerAsset(location2, metadata2);
        await execute_via_governance(api, alice, callData, referendumIndexObject);
        state = await api.query.assetManager.assetIdMetadata(9);
        while(state.isNone) {
            state = await api.query.assetManager.assetIdMetadata(9);
            await timer(3000);
        }
        console.log(new Date() + " Register Asset9:" + JSON.stringify(state));

        // 10(decimal:18)
        callData = api.tx.assetManager.registerAsset(local_asset(parachainId, "MANDEX"), metadata3);
        await execute_via_governance(api, alice, callData, referendumIndexObject);
        state = await api.query.assetManager.assetIdMetadata(10);
        while(state.isNone) {
            state = await api.query.assetManager.assetIdMetadata(10);
            await timer(3000);
        }
        console.log(new Date() + " Register Asset10:" + JSON.stringify(state));

        // register lp asset 11(decimal:12)
        callData = api.tx.assetManager.registerLpAsset(8, 9, lp_metadata);
        await execute_via_governance(api, alice, callData, referendumIndexObject);
        state = await api.query.assetManager.assetIdMetadata(11);
        while(state.isNone) {
            state = await api.query.assetManager.assetIdMetadata(11);
            await timer(3000);
        }
        console.log(new Date() + " Register LP Asset11:" + JSON.stringify(state));

        console.log(new Date() + " Register LP Asset block:" + Number(await api.query.system.number()));

        // create dex pair
        callData = api.tx.zenlinkProtocol.createPair([parachainId,2,8], [parachainId,2,9]);
        await execute_via_governance(api, alice, callData, referendumIndexObject);

        console.log(new Date() + " Create Pair block:" + Number(await api.query.system.number()));

        state = await api.query.zenlinkProtocol.pairStatuses([[parachainId,2,8], [parachainId,2,9]]);
        console.log(new Date() + " Create Pair status0:" + JSON.stringify(state));

        callData = api.tx.assetManager.mintAsset(8, alice.address, new BN("20000000000000"));
        await execute_via_governance(api, alice, callData, referendumIndexObject);
        callData = api.tx.assetManager.mintAsset(9, alice.address, new BN("200000000000000000"));
        await execute_via_governance(api, alice, callData, referendumIndexObject);
        callData = api.tx.assetManager.mintAsset(10, alice.address, new BN("1000000000000000000000000"));
        await execute_via_governance(api, alice, callData, referendumIndexObject);

        console.log(new Date() + " Mint Asset block:" + Number(await api.query.system.number()));

        // add liquidity to dex
        callData = api.tx.zenlinkProtocol.addLiquidity([2104,2,8], [2104,2,9],
            new BN("10000000000000"), new BN("100000000000000000"), new BN("10000000000000"), new BN("100000000000000000"), 1000);
        await execute_transaction(api, alice, callData, false);

        console.log(new Date() + " Add Liquidity block:" + Number(await api.query.system.number()));

        // create farming pool: stake 11(LP), reward 10(MANDEX)
        callData = api.tx.farming.createFarmingPool([[11, 1000000000]], [[10, new BN("1000000000000000000")]], null, 10000000000000, 1, 0, 0, 2);
        await execute_via_governance(api, alice, callData, referendumIndexObject);

        // charge reward token to farming pool account
        callData = api.tx.farming.charge(0, [[10, new BN("1000000000000000000000")]]);
        await execute_transaction(api, alice, callData, false);

        // mock new block
        callData = api.tx.system.remark("0x00");
        await execute_transaction(api, alice, callData, false);

        callData = api.tx.system.remark("0x00");
        await execute_transaction(api, alice, callData, false);

        await timer(1000);
        state = await api.query.zenlinkProtocol.pairStatuses([[parachainId,2,8], [parachainId,2,9]]);
        let json = JSON.parse(JSON.stringify(state));
        console.log(new Date() + " After AddLiquidity Pair status1:" + JSON.stringify(state));
        expect(new BN(json.trading["totalSupply"].toString())).to.deep.equal(new BN("1000000000000000"));

        state = await api.query.farming.poolInfos(0);
        json = JSON.parse(JSON.stringify(state));
        expect(json.state).to.deep.equal("Charged");

        // user deposit lp token to farming pool
        callData = api.tx.farming.deposit(0, new BN("10000000000000"), null);
        await execute_transaction(api, alice, callData, false);

        // mock new block
        callData = api.tx.system.remark("0x00");
        await execute_transaction(api, alice, callData, false);
        callData = api.tx.system.remark("0x00");
        await execute_transaction(api, alice, callData, false);

        await timer(1000);
        state = await api.query.farming.poolInfos(0);
        json = JSON.parse(JSON.stringify(state));
        expect(json.state).to.deep.equal("Ongoing");

        state = await api.query.farming.sharesAndWithdrawnRewards(0, alice.address);
        expect(new BN(JSON.parse(JSON.stringify(state)).share.toString())).to.deep.equal(new BN("10000000000000"));

        // get farming reward
        let response = await (api.rpc as any).farming.getFarmingRewards(alice.address, 0);
        expect(new BN(response[0][1].toString())).to.deep.equal(new BN("1000000000000000000"));

        callData = api.tx.system.remark("0x00");
        await execute_transaction(api, alice, callData, false);

        await timer(1000);
        response = await (api.rpc as any).farming.getFarmingRewards(alice.address, 0);
        expect(new BN(response[0][1].toString())).to.deep.equal(new BN("2000000000000000000"));

        api.disconnect();
    }).timeout(test_config.timeout);
});
