import { Keyring } from "@polkadot/keyring";
import {
  createPromiseApi,
  delay,
  createAssetMetadata,
  createMultiLocationWithParaId,
} from "../utils/utils";
import { BN } from "@polkadot/util";
import { nodeAddress, signer } from "../config/config.json";

async function main() {
  const nodeApi = await createPromiseApi(nodeAddress);

  const keyring = new Keyring({ type: "sr25519", ss58Format: 78 });
  const alice = keyring.addFromUri(signer);

  const amount = 100000000;
  const decimal = nodeApi.registry.chainDecimals;
  const toMint = new BN(amount).mul(new BN(decimal));

  let createAssetsCalls = [];
  let mintAssetsCalls = [];
  const symbols = ["KMA", "MANTA", "DOL"];
  const assetIds = [8, 9, 10];
  for (let i = 0; i < 3; i++) {
    const assetLocaltion = createMultiLocationWithParaId(2000 + i);
    const assetMetadata = createAssetMetadata(symbols[i], symbols[i], 12);
    const createAssetsCall = nodeApi.tx.assetManager.registerAsset(
      assetLocaltion,
      assetMetadata
    );
    const sudoCall = nodeApi.tx.sudo.sudo(createAssetsCall);
    createAssetsCalls.push(sudoCall);

    const _mintAssetsCall = nodeApi.tx.assetManager.mintAsset(
      assetIds[i],
      alice.address,
      toMint
    );
    const mintAssetsCall = nodeApi.tx.sudo.sudo(_mintAssetsCall);
    mintAssetsCalls.push(mintAssetsCall);
  }

  // initialize create three assets
  const unsub1 = await nodeApi.tx.utility
    .batch(createAssetsCalls)
    .signAndSend(alice, { nonce: -1 }, (result) => {
      console.log(`Current status is ${result.status}`);
      result.events.forEach(({ phase, event: { data, method, section } }) => {
        console.log(`\t' ${phase}: ${section}.${method}:: ${data}`);
      });
      if (result.status.isInBlock) {
        console.log(
          `Transaction included at blockHash ${result.status.asInBlock}`
        );
      } else if (result.status.isFinalized) {
        console.log(
          `Transaction finalized at blockHash ${result.status.asFinalized}`
        );
        unsub1();
      }
    });

  const unsub2 = await nodeApi.tx.utility
    .batch(mintAssetsCalls)
    .signAndSend(alice, { nonce: -1 }, (result) => {
      console.log(`Current status is ${result.status}`);
      result.events.forEach(({ phase, event: { data, method, section } }) => {
        console.log(`\t' ${phase}: ${section}.${method}:: ${data}`);
      });
      if (result.status.isInBlock) {
        console.log(
          `Transaction included at blockHash ${result.status.asInBlock}`
        );
      } else if (result.status.isFinalized) {
        console.log(
          `Transaction finalized at blockHash ${result.status.asFinalized}`
        );
        unsub2();
      }
    });
  await delay(12000);

  await nodeApi.disconnect();
}

main().catch(console.error);
