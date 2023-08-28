const fs = require('fs');
const bigJson = require('big-json');

// MantaPay, AssetManager and Assets storage items
let prefixes = [
'0x682a59d51ab9e48a8c8cc418ff9708d2',
'0x4ae7e256f92e5888372d72f3e4db1003',
'0xa66d1aecfdbd14d785a4d1d8723b4beb'
];

// mantasbt, merkle trie prefix
const mantaSbtPrefix = ['0xee3a0abfdb3bbd4914c7ac9d04e5f843'];

async function main() {
  let storagePath = './data/storage.json';
  let mantaSbtStoragePath = './data/mantaSbtStorage.json';
  let forkPath = './data/fork.json';

  let storage = JSON.parse(fs.readFileSync(storagePath, 'utf8'));
  let forkedSpec = JSON.parse(fs.readFileSync(forkPath, 'utf8'));
  
  // Grab the items to be moved, then iterate through and insert into storage
  storage
  .filter((i) => prefixes.some((prefix) => i[0].startsWith(prefix)))
  .forEach(([key, value]) => (forkedSpec.genesis.raw.top[key] = value));
  
  // insert mantasbt's storages into chain spec
  if (fs.existsSync(mantaSbtStoragePath)) {
    let mantaSbtStorage = JSON.parse(fs.readFileSync(mantaSbtStoragePath, 'utf8'));
    mantaSbtStorage
    .filter((i) => mantaSbtPrefix.some((prefix) => i[0].startsWith(prefix)))
    .forEach(([key, value]) => (forkedSpec.genesis.raw.top[key] = value));
  }

  await new Promise((resolve, reject) => {
    bigJson.createStringifyStream({ body: forkedSpec })
      .pipe(fs.createWriteStream(forkPath)
        .on('end', function() {
          resolve();
        }));
  });

  process.exit();
}

main().catch(console.error);
