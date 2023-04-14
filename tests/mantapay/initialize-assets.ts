import { Keyring } from "@polkadot/keyring";
import {
  createPromiseApi,
  delay,
  createAssetMetadata,
  createMultiLocationWithParaId,
  readChainSpec,
} from "../utils/utils";
import { BN } from "@polkadot/util";
import { mnemonic } from "../config/config.json";
import { execute_with_root_via_governance } from "./manta_pay";

async function main() {
  const chainSpec = await readChainSpec();
  const wsPort = chainSpec.parachains[0].nodes[0].wsPort;
  const nodeAddress = "ws://127.0.0.1:" + wsPort;

  const nodeApi = await createPromiseApi(nodeAddress);

  const keyring = new Keyring({ type: "sr25519", ss58Format: 78 });
  const alice = keyring.addFromUri(mnemonic);

  const amount = 100000000;
  const decimal = nodeApi.registry.chainDecimals;
  const factor = new BN(10).pow(new BN(decimal));
  const toMint = new BN(amount).mul(factor);

  const symbols = ["KMA", "MANTA", "DOL"];
  const assetIds = [8, 9, 10];
  let referendumIndex = parseInt(
    (await nodeApi.query.democracy.referendumCount()).toString()
  );
  for (let i = 0; i < 3; i++) {
    const assetLocaltion = createMultiLocationWithParaId(2000 + i);
    const assetMetadata = createAssetMetadata(symbols[i], symbols[i], 12);
    const createAssetsCall = nodeApi.tx.assetManager.registerAsset(
      assetLocaltion,
      assetMetadata
    );

    const mintAssetsCall = nodeApi.tx.assetManager.mintAsset(
      assetIds[i],
      alice.address,
      toMint
    );

    await execute_with_root_via_governance(nodeApi, alice, createAssetsCall, {
      referendumIndex: referendumIndex,
    });
    referendumIndex += 1;
    await execute_with_root_via_governance(nodeApi, alice, mintAssetsCall, {
      referendumIndex: referendumIndex,
    });
    referendumIndex += 1;
  }

  await delay(60000);
  await nodeApi.disconnect();
}

main().catch(console.error);
