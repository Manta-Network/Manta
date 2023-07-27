import { expect } from "chai";
import { step } from "mocha-steps";
import {describeWithManta, executeTx, remark} from "./util";
import '@polkadot/api-augment';

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

describeWithManta("Manta RPC (Assets)", (context) => {
    step("asset manager", async function () {
        const parachainId = Number(await context.api.query.parachainInfo.parachainId());
        console.log(new Date() + " parachain:" + parachainId);

        let callData = await context.api.tx.assetManager.registerAsset(location, metadata);
        await executeTx(context, callData, true);

        let state: any = await context.api.query.assetManager.assetIdMetadata(8);
        console.log(new Date() + " Register Asset8:" + JSON.stringify(state));
    });
});