import { assert } from "chai";
import { Keyring } from "@polkadot/keyring";
import { ApiPromise } from "@polkadot/api";
import "@polkadot/api-augment";
import { nodeAddress, signer } from "../config/config.json";
import { createPromiseApi, delay } from "../utils/utils";
import { base64Decode } from "@polkadot/util-crypto";
import { $Receivers, $Senders } from "../types";

describe("Relaying non subscription rpc methods", function () {
  let fullNodeApi: ApiPromise;

  before(async function () {
    // ensure node is health
    fullNodeApi = await createPromiseApi(nodeAddress);
    const fullNodeHealth = await fullNodeApi.rpc.system.health();
    assert.isNotTrue(fullNodeHealth.toJSON().isSyncing);
  });

  it("Check pull response", async function () {
    const keyring = new Keyring({ type: "sr25519", ss58Format: 78 });
    const alice = keyring.addFromUri(signer);

    // there're only 40 private extrinsic sent, so 128 is big enough to get all Utxos.
    const totalReceivers = 128;
    const totalSenders = 128;

    const shardNumber = 256;
    let checkPoint = {
      receiver_index: new Array<number>(shardNumber).fill(0),
      sender_index: 0,
    };

    const pullResponse = await (fullNodeApi.rpc as any).mantaPay.pull_ledger_diff(
      checkPoint,
      BigInt(totalReceivers),
      BigInt(totalSenders)
    );
    
    const densePullResponse = await (fullNodeApi.rpc as any).mantaPay.dense_pull_ledger_diff(
      checkPoint,
      BigInt(totalReceivers),
      BigInt(totalSenders)
    );
      
    // decode densePullResponse, ensure which is equal to pullResponse
    assert.isNotTrue(pullResponse.should_continue);
    assert.isNotTrue(densePullResponse.should_continue);

    const decodedRecievers = $Receivers.decode(base64Decode(densePullResponse.receivers.toString()));
    const decodedSenders = $Senders.decode(base64Decode(densePullResponse.senders.toString()));

    // assert.equal(decodedRecievers[0], pullResponse.receivers[0]);
    assert.equal(decodedSenders, pullResponse.senders);

    assert.equal(densePullResponse.senders_receivers_total, pullResponse.senders_receivers_total);
  });

  after(async function () {
    // Exit the mocha process.
    // If not, the process will pend there.
    await fullNodeApi.disconnect();
  });
});

