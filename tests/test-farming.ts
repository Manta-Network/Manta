import { ApiPromise, WsProvider } from '@polkadot/api';
import { BN } from '@polkadot/util';
import { Keyring } from '@polkadot/keyring';
import { farming_rpc_api, farming_types } from './types';
import { expect } from 'chai';
import minimist, { ParsedArgs } from 'minimist';
import {execute_transaction, execute_via_governance,timer } from "./chain-util";
import {
    LP_USDT_USDC_METADATA,
    MANDEX_METADATA,
    USDC_LOCATION,
    USDC_METADATA,
    USDT_LOCATION,
    USDT_METADATA
} from "./constants";

const test_config = {
    ws_address: "ws://127.0.0.1:9800",
    timeout: 2000000
}

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
        console.log(new Date() + " parachain:" + parachainId);

        // register asset 8(decimal:6)
        let callData = api.tx.assetManager.registerAsset(USDT_LOCATION, USDT_METADATA);
        await execute_via_governance(api, alice, callData);

        let state: any = await api.query.assetManager.assetIdMetadata(8);
        while(state.isNone) {
            state = await api.query.assetManager.assetIdMetadata(8);
            await timer(3000);
        }
        console.log(new Date() + " Register Asset8:" + JSON.stringify(state));

        // 9(decimal:10)
        callData = api.tx.assetManager.registerAsset(USDC_LOCATION, USDC_METADATA);
        await execute_via_governance(api, alice, callData);
        state = await api.query.assetManager.assetIdMetadata(9);
        while(state.isNone) {
            state = await api.query.assetManager.assetIdMetadata(9);
            await timer(3000);
        }
        console.log(new Date() + " Register Asset9:" + JSON.stringify(state));

        // 10(decimal:18)
        callData = api.tx.assetManager.registerAsset(local_asset(parachainId, "MANDEX"), MANDEX_METADATA);
        await execute_via_governance(api, alice, callData);
        let mandexId = 10;
        state = await api.query.assetManager.assetIdMetadata(mandexId);
        while(state.isNone) {
            state = await api.query.assetManager.assetIdMetadata(mandexId);
            await timer(3000);
        }
        console.log(new Date() + " Register Asset10:" + JSON.stringify(state));

        // register lp asset 11(decimal:12)
        callData = api.tx.assetManager.registerLpAsset(8, 9, LP_USDT_USDC_METADATA);
        await execute_via_governance(api, alice, callData);
        let lpAssetId = 11;
        state = await api.query.assetManager.assetIdMetadata(lpAssetId);
        while(state.isNone) {
            state = await api.query.assetManager.assetIdMetadata(lpAssetId);
            await timer(3000);
        }
        console.log(new Date() + " Register LP Asset11:" + JSON.stringify(state));

        console.log(new Date() + " Register LP Asset block:" + Number(await api.query.system.number()));

        // create dex pair
        callData = api.tx.zenlinkProtocol.createPair([parachainId,2,8], [parachainId,2,9]);
        await execute_via_governance(api, alice, callData);

        console.log(new Date() + " Create Pair block:" + Number(await api.query.system.number()));

        callData = api.tx.assetManager.mintAsset(8, alice.address, new BN("20000000000000"));
        await execute_via_governance(api, alice, callData);
        callData = api.tx.assetManager.mintAsset(9, alice.address, new BN("200000000000000000"));
        await execute_via_governance(api, alice, callData);
        callData = api.tx.assetManager.mintAsset(mandexId, alice.address, new BN("1000000000000000000000000"));
        await execute_via_governance(api, alice, callData);

        console.log(new Date() + " Mint Asset block:" + Number(await api.query.system.number()));

        state = await api.query.zenlinkProtocol.pairStatuses([[parachainId,2,8], [parachainId,2,9]]);
        console.log(new Date() + " Pair status0:" + JSON.stringify(state));

        // add liquidity to dex
        callData = api.tx.zenlinkProtocol.addLiquidity([parachainId,2,8], [parachainId,2,9],
            new BN("10000000000000"), new BN("100000000000000000"), new BN("10000000000000"), new BN("100000000000000000"), 1000);
        await execute_transaction(api, alice, callData, false);

        await timer(13000);

        console.log(new Date() + " Add Liquidity block:" + Number(await api.query.system.number()));

        // create farming pool: stake 11(LP), reward 10(MANDEX)
        callData = api.tx.farming.createFarmingPool([[lpAssetId, 1000000000]], [[mandexId, new BN("1000000000000000000")]], null, 10000000000000, 1, 0, 0, 2);
        await execute_via_governance(api, alice, callData);

        state = await api.query.farming.poolInfos(0);
        while(state.isNone) {
            state = await api.query.farming.poolInfos(0);
            await timer(3000);
        }
        console.log(new Date() + " Query farming pool0:" + JSON.stringify(state));

        // charge reward token to farming pool account
        callData = api.tx.farming.charge(0, [[mandexId, new BN("1000000000000000000000")]]);
        await execute_transaction(api, alice, callData, false);

        let json = JSON.parse(JSON.stringify(state));
        let stateString = json.state.toString();
        while(stateString != "Charged") {
            state = await api.query.farming.poolInfos(0);
            json = JSON.parse(JSON.stringify(state));
            stateString = json.state.toString();
            await timer(3000);
        }
        console.log(new Date() + " Query farming pool1:" + JSON.stringify(state));
        expect(json.state).to.deep.equal("Charged");

        // mock new block
        callData = api.tx.system.remark("0x00");
        await execute_transaction(api, alice, callData, false);

        state = await api.query.zenlinkProtocol.pairStatuses([[parachainId,2,8], [parachainId,2,9]]);
        json = JSON.parse(JSON.stringify(state));
        console.log(new Date() + " After AddLiquidity Pair status1:" + JSON.stringify(state));
        expect(new BN(json.trading["totalSupply"].toString())).to.deep.equal(new BN("1000000000000000"));

        let block1 = Number(await api.query.system.number());

        // user deposit lp token to farming pool
        callData = api.tx.farming.deposit(0, new BN("10000000000000"), null);
        await execute_transaction(api, alice, callData, false);

        // mock new block
        let block2 = Number(await api.query.system.number());
        while(block1 == block2) {
            callData = api.tx.system.remark("0x00");
            await execute_transaction(api, alice, callData, false);
            await timer(3000);
            block2 = Number(await api.query.system.number());
        }
        console.log(new Date() + " farming deposit before:" + block1 + ",after:" + block2);

        state = await api.query.farming.poolInfos(0);
        json = JSON.parse(JSON.stringify(state));
        stateString = json.state.toString();
        while(stateString != "Ongoing") {
            state = await api.query.farming.poolInfos(0);
            json = JSON.parse(JSON.stringify(state));
            stateString = json.state.toString();
            await timer(3000);
        }
        console.log(new Date() + " Query farming pool2:" + JSON.stringify(state));
        expect(json.state).to.deep.equal("Ongoing");

        state = await api.query.farming.sharesAndWithdrawnRewards(0, alice.address);
        console.log(new Date() + " share info:" + JSON.stringify(state));
        expect(new BN(JSON.parse(JSON.stringify(state)).share.toString())).to.deep.equal(new BN("10000000000000"));

        block1 = Number(await api.query.system.number());
        block2 = Number(await api.query.system.number());
        while(block1 == block2) {
            callData = api.tx.system.remark("0x00");
            await execute_transaction(api, alice, callData, false);
            await timer(3000);
            block2 = Number(await api.query.system.number());
        }
        console.log(new Date() + " mock new block before:" + block1 + ",after:" + block2);

        // get farming reward
        let response = await (api.rpc as any).farming.getFarmingRewards(alice.address, 0);
        console.log(new Date() + " farming reward0:" + JSON.stringify(response));
        expect(new BN(response[0][1].toString())).to.deep.equal(new BN("1000000000000000000"));

        block1 = Number(await api.query.system.number());
        block2 = Number(await api.query.system.number());
        while(block1 == block2) {
            callData = api.tx.system.remark("0x00");
            await execute_transaction(api, alice, callData, false);
            await timer(3000);
            block2 = Number(await api.query.system.number());
        }
        console.log(new Date() + " mock new block before:" + block1 + ",after:" + block2);

        response = await (api.rpc as any).farming.getFarmingRewards(alice.address, 0);
        console.log(new Date() + " farming reward1:" + JSON.stringify(response));
        expect(new BN(response[0][1].toString())).to.deep.equal(new BN("2000000000000000000"));

        api.disconnect();
    }).timeout(test_config.timeout);
});
