const fs = require('fs');

async function main() {
  let storage = JSON.parse(fs.readFileSync('./storage.json', 'utf8'));
  let forkedSpec = JSON.parse(fs.readFileSync('./fork.json', 'utf8'));
  // Grab the items to be moved, then iterate through and insert into storage
  storage
  .filter((i) => prefixes.some((prefix) => i[0].startsWith(prefix)))
  .forEach(([key, value]) => (forkedSpec.genesis.raw.top[key] = value));
}

main().catch(console.error);
