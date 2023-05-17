const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');

const keyring = new Keyring({ type: 'sr25519' });

// Create a promise API instance of the passed in node address.
async function createPromiseApi(nodeAddress) {
  const wsProvider = new WsProvider(nodeAddress);
  const api = await ApiPromise.create({ provider: wsProvider });
  await api.isReady;
  return api;
}

async function main() {
  const nodeAddress = 'wss://crispy.baikal.testnet.calamari.systems:443';
  const api = await createPromiseApi(nodeAddress);

  const aliceKey = keyring.addFromMnemonic("bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice");
  const lotteryAccount = 'dmwQify2zjqgMQfDhDQRppgEeBFVvJaX3bq3xqtFmM94PKfgY';

  // fund lottery account with gas money
  const id = new Uint8Array(32);
  id[0] = 1;
  const amount = new Uint8Array(16); // roughly 50k KMA
  amount[7] = 1;
  const transferTx = api.tx.mantaPay.publicTransfer({id: id,value:amount},lotteryAccount);
  await transferTx.signAndSend(aliceKey, {nonce:-1});
  console.log(`Transferred ${amount} KMA from Alice to ${lotteryAccount}`);

  // deposit from alice to lottery
  const depositTx = api.tx.lottery.deposit(amount);
  await depositTx.signAndSend(aliceKey, {nonce:-1});
  console.log(`Deposited ${amount} KMA from Alice to ${lotteryAccount}`);
  
  // start lottery
  const startLotteryTx = api.tx.lottery.startLottery();
  const proposalLength = 1000;
  const threshold = 1;
  const proposal = api.tx.council.propose(threshold, startLotteryTx, proposalLength);
  // Sign and send the council proposal
  await proposal.signAndSend(aliceKey, {nonce:-1});
  const proposalHash = proposal.method.hash.toHex();
  console.log(`Lottery start proposal submitted by Alice. Proposal Hash: ${proposalHash}`);
}
main().catch(console.error);