const fs = require('fs');

// MantaPay, AssetManager and Assets storage items
let prefixes = [
'0x682a59d51ab9e48a8c8cc418ff9708d2',
'0x4ae7e256f92e5888372d72f3e4db1003',
'0xa66d1aecfdbd14d785a4d1d8723b4beb'
];

async function main() {
  let storagePath = './data/storage.json';
  let forkPath = './data/fork.json';

  let storage = JSON.parse(fs.readFileSync(storagePath, 'utf8'));
  let forkedSpec = JSON.parse(fs.readFileSync(forkPath, 'utf8'));

  // Grab the items to be moved, then iterate through and insert into storage
  storage
  .filter((i) => prefixes.some((prefix) => i[0].startsWith(prefix)))
  .forEach(([key, value]) => (forkedSpec.genesis.raw.top[key] = value));

  fs.writeFileSync(forkPath, JSON.stringify(forkedSpec, null, 4));

  process.exit();
}

main().catch(console.error);
