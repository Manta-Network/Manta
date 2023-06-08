import { ApiPromise, WsProvider } from '@polkadot/api';
import '@polkadot/api-augment';
import '@polkadot/api-augment/kusama';
import '@polkadot/api-augment/polkadot';
import { assert } from 'chai';

const TEST_TIMEOUT = 300000; // 5 minutes

export async function createPromiseApi(nodeAddress: string) {
  const wsProvider = new WsProvider(nodeAddress);

  const api = new ApiPromise({
    provider: wsProvider,
  });
  await api.isReady;
  console.log(`${nodeAddress} has been started`);
  return api;
}

describe('Check Parachain_Lease_Expiration', () => {
    it('Check Manta Parachain Lease', async () => {
        const polkadotNode = 'wss://1rpc.io/dot';
        const polkadotApi = await createPromiseApi(polkadotNode);
        const mantaParaId = 2104;
        const oneDay = 3600 * 24;

        const relaychainBlockTime = polkadotApi.consts.babe.expectedBlockTime.toNumber() / 1000;
        const bestNumber = (await polkadotApi.derive.chain.bestNumber()).toNumber();
        const startNumber = bestNumber - polkadotApi.consts.slots.leaseOffset.toNumber();
        const leasePeriod = polkadotApi.consts.slots.leasePeriod.toNumber();
        const currentPeriod = Math.floor(startNumber / leasePeriod);

        const remainingLeaseLength = await polkadotApi.query.slots.leases(mantaParaId);
        const endPeriod = currentPeriod + remainingLeaseLength.length - 1;

        await polkadotApi.disconnect();
        assert(remainingLeaseLength.length >= 2, 'Manta lease is going to expire, please do auction.');
        console.log(`Remaining lease period for manta: ${currentPeriod}-${endPeriod}`);

        const firstBlock = startNumber % leasePeriod;
        if (firstBlock === 0) {
            const remainingDays = leasePeriod * relaychainBlockTime * remainingLeaseLength.length / oneDay;
            console.log(`Manta parachain lease has ${remainingDays} days remaining.`);
        } else {
            const currentLeaseRemainedBlocks = leasePeriod - firstBlock;
            const remainingDays = leasePeriod * relaychainBlockTime * (remainingLeaseLength.length - 1) / oneDay + (currentLeaseRemainedBlocks * relaychainBlockTime / oneDay);
            console.log(`Manta parachain lease has ${remainingDays} days remaining.`);
        }
    }).timeout(TEST_TIMEOUT);

    it('Check Calamari Parachain Lease', async () => {
        const kusamaNode = 'wss://kusama.api.onfinality.io/public-ws';
        const kusamaApi = await createPromiseApi(kusamaNode);
        const calamariParaId = 2084;
        const oneDay = 3600 * 24;

        const relaychainBlockTime = kusamaApi.consts.babe.expectedBlockTime.toNumber() / 1000;
        const bestNumber = (await kusamaApi.derive.chain.bestNumber()).toNumber();
        const startNumber = bestNumber - kusamaApi.consts.slots.leaseOffset.toNumber();
        const leasePeriod = kusamaApi.consts.slots.leasePeriod.toNumber();
        const currentPeriod = Math.floor(startNumber / leasePeriod);

        const remainingLeaseLength = await kusamaApi.query.slots.leases(calamariParaId);
        const endPeriod = currentPeriod + remainingLeaseLength.length - 1;

        await kusamaApi.disconnect();
        assert(remainingLeaseLength.length >= 2, 'Calamari lease is going to expire, please do auction.');
        console.log(`Remaining lease period for calamari: ${currentPeriod}-${endPeriod}`);

        const firstBlock = startNumber % leasePeriod;
        if (firstBlock === 0) {
            const remainingDays = leasePeriod * relaychainBlockTime * remainingLeaseLength.length / oneDay;
            console.log(`Calamari parachain lease has ${remainingDays} days remaining.`);
        } else {
            const currentLeaseRemainedBlocks = leasePeriod - firstBlock;
            const remainingDays = leasePeriod * relaychainBlockTime * (remainingLeaseLength.length - 1) / oneDay + (currentLeaseRemainedBlocks * relaychainBlockTime / oneDay);
            console.log(`Calamari parachain lease has ${remainingDays} days remaining.`);
        }
    }).timeout(TEST_TIMEOUT);
});
