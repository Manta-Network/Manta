import { Keyring } from "@polkadot/keyring";
import { createPromiseApi, delay, readChainSpec } from "../utils/utils";
import { readFile } from "fs/promises";
import { signer } from "../config/config.json";

async function main() {
  const chainSpec = await readChainSpec();
  const wsPort = chainSpec.parachains[0].nodes[0].wsPort;
  const nodeAddress = "ws://127.0.0.1:" + wsPort;
  const nodeApi = await createPromiseApi(nodeAddress);

  const keyring = new Keyring({ type: "sr25519", ss58Format: 78 });
  const alice = keyring.addFromUri(signer);

  const offSet = 1;
  const coinSize = 552; // each coin size is 552.
  const coinsCount = 40;
  const batchSize = 5;
  const content = await readFile("data/init-utxo/initialize-utxo");
  const buffer = content.subarray(
    offSet + 0 * batchSize * coinSize,
    offSet + 0 * batchSize * coinSize + coinSize * coinsCount
  );
  let start = 0;
  let end = start + coinSize;
  for (let k = 0; k < coinsCount / batchSize; ++k) {
    let mintTxs = [];
    for (let i = 0; i < batchSize; ++i) {
      const mint = nodeApi.tx.mantaPay.toPrivate(buffer.subarray(start));
      mintTxs.push(mint);
      start = end;
      end += coinSize;
    }

    const unsub = await nodeApi.tx.utility
      .batch(mintTxs)
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
          unsub();
        }
      });
    await delay(12000); // wait 12s to ensure transactions are included.
  }

  console.log(`${coinsCount} transactions have been sent.`);
  await nodeApi.disconnect();
}

main().catch(console.error);
