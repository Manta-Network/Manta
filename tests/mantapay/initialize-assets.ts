import { Keyring } from "@polkadot/keyring";
import {
  createPromiseApi,
  delay,
  createAssetMetadata,
  createMultiLocationWithParaId,
} from "../utils/utils";
import { BN } from "@polkadot/util";
import { nodeAddress, signer, mnemonic } from "../config/config.json";
import { execute_with_root_via_governance } from "./manta_pay";

async function main() {
  const nodeApi = await createPromiseApi(nodeAddress);

  const keyring = new Keyring({ type: "sr25519", ss58Format: 78 });
  const alice = keyring.addFromUri(mnemonic);

  const amount = 100000000;
  const decimal = nodeApi.registry.chainDecimals;
  const toMint = new BN(amount).mul(new BN(decimal));

  let createAssetsCalls = [];
  let mintAssetsCalls = [];
  const symbols = ["KMA", "MANTA", "DOL"];
  const assetIds = [8, 9, 10];
  let referendumIndex = (
    await nodeApi.query.democracy.referendumCount()
  ).toNumber();
  for (let i = 0; i < 3; i++) {
    const assetLocaltion = createMultiLocationWithParaId(2000 + i);
    const assetMetadata = createAssetMetadata(symbols[i], symbols[i], 12);
    const createAssetsCall = nodeApi.tx.assetManager.registerAsset(
      assetLocaltion,
      assetMetadata
    );

    const _mintAssetsCall = nodeApi.tx.assetManager.mintAsset(
      assetIds[i],
      alice.address,
      toMint
    );

    await execute_with_root_via_governance(nodeApi, alice, createAssetsCall, {
      referendumIndex: referendumIndex,
    });
    referendumIndex += 1;
    await execute_with_root_via_governance(nodeApi, alice, _mintAssetsCall, {
      referendumIndex: referendumIndex,
    });
    referendumIndex += 1;
  }

  await delay(60000);
  await nodeApi.disconnect();
}

main().catch(console.error);
