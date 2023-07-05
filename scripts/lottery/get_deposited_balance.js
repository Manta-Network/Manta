const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");

const keyring = new Keyring({ type: "sr25519" });

// Create a promise API instance of the passed in node address.
async function createPromiseApi(nodeAddress) {
  const wsProvider = new WsProvider(nodeAddress);
  const api = await ApiPromise.create({ provider: wsProvider });
  await api.isReady;
  return api;
}

async function main() {
  const nodeAddress = "wss://crispy.baikal.testnet.calamari.systems:443";
  const api = await createPromiseApi(nodeAddress);

  const aliceKey = keyring.addFromMnemonic(
    "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice"
  );
  const lotteryAccount = "dmwQify2zjqgMQfDhDQRppgEeBFVvJaX3bq3xqtFmM94PKfgY";

  const totalPot = await api.query.lottery.totalPot();
  console.log(`tokens in pot: ${totalPot}`);
}
main().catch(console.error);
