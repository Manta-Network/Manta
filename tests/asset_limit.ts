/* eslint-disable @typescript-eslint/no-unsafe-return */
import axios from 'axios';
import moment from 'moment';
import * as fs from 'fs';
import { ApiPromise, WsProvider } from '@polkadot/api';

// curl -s -X 'GET' 'https://api.coingecko.com/api/v3/coins/acala/history?date=09-11-2022&localization=en' -H 'accept: application/json' | jq '.market_data.current_price.usd'
const axios_api = axios.create({
    baseURL: 'https://api.coingecko.com/',
    headers: {
      'Content-Type': 'application/json',
      'X-API-KEY': '02dfd68d52654337a82519c4e17422a0',
      timeout: '10000',
      'User-Agent': 'curl/7.64.1',
    },
  });

function contains(arr: [{"token": string}], val: string) {
  for (var i = 0; i < arr.length; i++) {
    if (arr[i]["token"] === val) return true;
  }
  return false;
}

const default_limit = 2000; // asset limit with usd.
const path = "/tmp/price.json";
const tokens = ["calamari-network", "moonriver", "kusama", "karura"];
let asset_map = new Map([
  ["calamari-network", {
    "symbol": "KMA",
    "decimal": 12,
    "limit": default_limit
  }],
  ["moonriver", {
    "symbol": "MOVR",
    "decimal": 18,
    "limit": default_limit
  }],
  ["kusama", {
    "symbol": "KSM",
    "decimal": 12,
    "limit": default_limit
  }],
  ["liquid-ksm", {
    "symbol": "LKSM",
    "decimal": 12,
    "limit": default_limit
  }],
  ["phala-network", {
    "symbol": "PHA",
    "decimal": 12,
    "limit": default_limit
  }],
  ["karura", {
    "symbol": "KAR",
    "decimal": 12,
    "limit": default_limit
  }],
  ["acala-dollar-acala", {
    "symbol": "AUSD",
    "decimal": 12,
    "limit": default_limit
  }],
  ["acala-dollar-karura", {
    "symbol": "KUSD",
    "decimal": 12,
    "limit": default_limit
  }],
]); 

// Due to rate limit of coingecko, run multi time: `yarn asset_price_test` until all tokens processed.
describe('Asset Price test', () => { 
  it('Asset Price test', async () => {
    var obj = [];
    var current_token: string = tokens[0];
    const history_count = 30;

    // read file and get next processing token.
    if (fs.existsSync(path)) {
      var data = fs.readFileSync(path, 'utf8');
      if (data != "") {
        obj = JSON.parse(data);
        if(obj.length == tokens.length) {
          return console.log("All token processed, QUIT.");
        }
        for(const token of tokens) {
          const token_is_exist = contains(obj, token);
          if(token_is_exist != true) {
            current_token = token;
            break;
          } else {
            console.log("token:" + token + " data existed. ✅");
          }
        }
      }
    }
    console.log("processing token:" + current_token + " ...");

    // fetch price of token, and calculate average price.
    const fetch_usd_price = async (token: string, date: string) => {
      const data = await axios_api.get('/api/v3/coins/'+token+'/history?date='+date+'&localization=en');
      return data.data.market_data.current_price.usd;
    }
    const asset_prices = async (token: string) => {
      let arr = Array.from(Array(history_count), (n, index) => index);
      return Promise.all(arr.map(async (i) => {
        const date_i = moment().subtract(i, 'days').format("DD-MM-YYYY");
        const res = await fetch_usd_price(token, date_i);
        return res;
      }));
    }
    const prices = await asset_prices(current_token);
    const total = prices.reduce((sum, cur) => {
      return sum + cur;
    }, 0);
    const average = total / history_count;
    console.log("token:" + current_token + ", average:" + average);

    // write result to file
    if (obj.length == 0) {
      obj = [{
        "token": current_token,
        "average": average            
      }]
    } else {
      obj.push({
        "token": current_token,
        "average": average
      });
    }
    var json = JSON.stringify(obj);
    fs.writeFile(path, json, 'utf8', function(err) {});
  }).timeout(30000);
});

// After all token price processed, run `yarn asset_limit_test` to get asset limit amount.
describe('Asset Limit test', () => { 
  it('Test asset limit', async () => {
    if (!fs.existsSync(path)) {
      return console.log("file not exist!");
    }
    var data = fs.readFileSync(path, 'utf8');
    const table_data = [];
    if (data === "") {
      return console.log("file is empty!");
    }
    var obj = JSON.parse(data);
    if(obj.length != tokens.length) {
      return console.log("Not all asset processed.");
    }

    const asset_price_maps = new Map();
    // wss://ws.calamari.systems/
    // wss://ws.rococo.dolphin.engineering
    const provider = new WsProvider('wss://ws.calamari.systems');
    const api = await ApiPromise.create({ provider });
    const assets = await api.query.assetManager.assetIdMetadata.entries();
    assets.forEach(([{ args: [assetId] }, value]) => {
      const json = JSON.stringify(value.toHuman());
      const jsonObj = JSON.parse(json);
      asset_price_maps.set(jsonObj["symbol"], assetId);
    });

    for(var i=0;i<obj.length;i++) {
      const token = obj[i]["token"];
      const price = obj[i]["average"];

      const asset_symbol = asset_map.get(token)?.symbol;
      const asset_id = asset_price_maps.get(asset_symbol);

      const limit = asset_map.get(token)?.limit as number;
      const units = asset_map.get(token)?.decimal as number;
      const amount = limit / price;
      const amounts = amount * Math.pow(10, units);

      table_data.push({
        asset_symbol: asset_symbol,
        asset_id: asset_id.toNumber(),
        decimal: units,
        price: price,
        amounts: Math.round(amounts)
      });
    }
    console.table(table_data);
  }).timeout(60000);
});
