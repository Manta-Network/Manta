const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const util = require("util");

const keyring = new Keyring({ type: "sr25519" });

// Create a promise API instance of the passed in node address.
async function createPromiseApi(nodeAddress) {
  const wsProvider = new WsProvider(nodeAddress);
  const api = await ApiPromise.create({ provider: wsProvider });
  await api.isReady;
  return api;
}

async function main() {
  // const nodeAddress = 'wss://crispy.baikal.testnet.calamari.systems:443'
  const nodeAddress = "ws://127.0.0.1:9111";
  const api = await createPromiseApi(nodeAddress);

  console.log(
    util.inspect(api, { showHidden: false, depth: null, colors: true })
  );
}
main().catch(console.error);
