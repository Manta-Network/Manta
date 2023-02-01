import { Keyring } from "@polkadot/keyring";
import { execute_with_root_via_governance } from "../mantapay/manta_pay";
import { createPromiseApi, readChainSpec, delay } from "../utils/utils";
import { assert } from "chai";
import minimist, { ParsedArgs } from "minimist";
import { blake2AsHex } from "@polkadot/util-crypto";
import * as fs from "fs";

const test_config = {
  mnemonic:
    "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice",
  timeout: 2000000,
};

describe("Node RPC Test", () => {
  it("Check RPC result", async () => {
    // create api
    let nodeAddress = "";
    const args: ParsedArgs = minimist(process.argv.slice(2));
    if (args["address"] == null) {
      const chainSpec = await readChainSpec();
      const wsPort = chainSpec.parachains[0].nodes[0].wsPort;
      nodeAddress = "ws://127.0.0.1:" + wsPort;
    } else {
      nodeAddress = args["address"];
    }
    const api = await createPromiseApi(nodeAddress);

    const keyring = new Keyring({ type: "sr25519" });
    const aliceKeyPair = keyring.addFromMnemonic(test_config.mnemonic);

    const oldRuntimeVersion = await api.rpc.state.getRuntimeVersion();
    const oldSpecVersion = oldRuntimeVersion["specVersion"];

    const code = fs.readFileSync("calamari.wasm").toString("hex");
    let codeHash = blake2AsHex(`0x${code}`);
    const authorizeUpgradeCallData =
      api.tx.parachainSystem.authorizeUpgrade(codeHash);
    var referendumIndexObject = { referendumIndex: 0 };
    execute_with_root_via_governance(
      api,
      aliceKeyPair,
      authorizeUpgradeCallData,
      referendumIndexObject
    );
    await delay(60000);
    api.tx.parachainSystem
      .enactAuthorizedUpgrade(`0x${code}`)
      .signAndSend(aliceKeyPair, { nonce: -1 });
    await delay(120000);

    let newRuntimeVersions = await api.rpc.state.getRuntimeVersion();
    const newSpecVersion = newRuntimeVersions["specVersion"];
    assert(newSpecVersion > oldSpecVersion);

    let blockNow = await api.rpc.chain.getBlock();
    let blockNumberNow = blockNow.block.header.number;
    await delay(60000);
    let blockLater = await api.rpc.chain.getBlock();
    let blockNumberLater = blockLater.block.header.number;
    assert(blockNumberLater > blockNumberNow);

    api.disconnect();
  }).timeout(test_config.timeout);
});
