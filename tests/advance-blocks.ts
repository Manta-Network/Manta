import { ApiPromise, WsProvider } from '@polkadot/api';
import '@polkadot/api-augment';

async function createPromiseApi(nodeAddress: string) {
  const wsProvider = new WsProvider(nodeAddress);

  const api = new ApiPromise({
    provider: wsProvider,
  });
  await api.isReady;
  console.log(`${nodeAddress} has been started`);
  return api;
}

async function delay(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function advance() {
    const mantaEndpoint = 'ws://127.0.0.1:8000';
    const dotEndpoint = 'ws://127.0.0.1:8001';
    const mantaApi = await createPromiseApi(mantaEndpoint);
    const dotApi = await createPromiseApi(dotEndpoint);
    await mantaApi.rpc('dev_newBlock', { count: 30 });
    await dotApi.rpc('dev_newBlock', { count: 60 });
    await mantaApi.disconnect();
    await dotApi.disconnect();
}

async function main() {
    await advance();
  }
  
  main().catch(console.error);
  