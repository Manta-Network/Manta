import { expect } from "chai";
import { step } from "mocha-steps";
import {describeWithManta, executeTx, remark} from "./util";
import '@polkadot/api-augment';
import {USDT_LOCATION, USDT_METADATA} from "../constants";

describeWithManta("Manta RPC (Assets)", (context) => {
    step("asset manager register asset should work", async function () {
        const parachainId = Number(await context.api.query.parachainInfo.parachainId());
        console.log(new Date() + " parachain:" + parachainId);

        let callData = await context.api.tx.assetManager.registerAsset(USDT_LOCATION, USDT_METADATA);
        await executeTx(context, callData, true);

        let state: any = await context.api.query.assetManager.assetIdMetadata(8);
        console.log(new Date() + " Register Asset8:" + JSON.stringify(state));
    });
});