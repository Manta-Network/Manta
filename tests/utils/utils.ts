import { ApiPromise, WsProvider } from "@polkadot/api";
import { decodeAddress, encodeAddress } from "@polkadot/keyring";
import { hexToU8a, isHex } from "@polkadot/util";
import { rpc_api, manta_pay_types } from "../types";
import { readFile } from "fs/promises";

export function isValidAddress(address: string) {
  try {
    encodeAddress(isHex(address) ? hexToU8a(address) : decodeAddress(address));
    return true;
  } catch (error) {
    return false;
  }
}

export function createAssetMetadata(
  name: string,
  symbol: string,
  decimals: number
) {
  const assetMetadata = {
    metadata: {
      name,
      symbol,
      decimals,
      isFrozen: false,
    },
    minBalance: 1,
    isSufficient: true,
  };
  return assetMetadata;
}

export function createMultiLocationWithParaId(paraId: number) {
  const assetLocaltion = {
    V1: {
      parent: 1,
      interior: { X1: { Parachain: { parachain: paraId } } },
    },
  };
  return assetLocaltion;
}

export async function createPromiseApi(nodeAddress: string) {
  const wsProvider = new WsProvider(nodeAddress);

  const api = new ApiPromise({
    provider: wsProvider,
    types: manta_pay_types,
    rpc: rpc_api,
  });
  await api.isReady;
  console.log(`${nodeAddress} has been started`);
  return api;
}

export async function delay(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export default createPromiseApi;

export async function readChainSpec() {
  const content = await readFile(
    "../.github/resources/config-for-integration-test.json",
    "utf8"
  );
  return JSON.parse(content);
}
