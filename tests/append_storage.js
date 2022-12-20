const fs = require('fs');
const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { xxhashAsHex } = require('@polkadot/util-crypto');

let prefixes = ['0x26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9' /* System.Account */];
const skippedModulesPrefix = ['System', 'Session', 'Authorship', 'CollatorSelection', 'ParachainSystem', 'Timestamp','ParachainInfo', 
'TransactionPause', 'TransactionPayment', 'Democracy', 'Council', 'CouncilMembership','TechnicalCommittee', 'AuthorInherent', 'AuraAuthorFilter',
'Aura', 'Treasury', 'XcmpQueue', 'PolkadotXcm', 'CumulusXcm', 'DmpQueue', 'XTokens', 'Utility', 'Multisig', 'Preimage', 'Scheduler', 'Sudo', 
'TechnicalMembership', 'AuraExt', 'ParachainStaking', 'AuthorInherent', 'AuraAuthorFilter', 'CalamariVesting', 'Balances',]

async function main() {
  let nodeAddress = "ws://127.0.0.1:9801";

  const wsProvider = new WsProvider(nodeAddress);
  const api = await ApiPromise.create({ 
      provider: wsProvider});

  const metadata = await api.rpc.state.getMetadata();
  // Populate the prefixes array
  const modules = metadata.asLatest.pallets;
  modules.forEach((module) => {
    if (module.storage) {
      if (!skippedModulesPrefix.includes(module.name.toHuman())) {
        prefixes.push(xxhashAsHex(module.name, 128));
        console.log("\n module.name: ", module.name);
      }
    }
  });
  console.log("\n prefixes: ", prefixes);

  let storage = JSON.parse(fs.readFileSync('./storage.json', 'utf8'));
  let forkedSpec = JSON.parse(fs.readFileSync('./fork.json', 'utf8'));

  // Grab the items to be moved, then iterate through and insert into storage
  storage
  .filter((i) => prefixes.some((prefix) => i[0].startsWith(prefix)))
  .forEach(([key, value]) => (forkedSpec.genesis.raw.top[key] = value));

  fs.writeFileSync('./fork.json', JSON.stringify(forkedSpec, null, 4));

  process.exit();
}

main().catch(console.error);
