import { spawn, ChildProcess } from "child_process";
import chaiAsPromised from "chai-as-promised";
import chai from "chai";
import getPort from 'get-port';
import {ApiPromise, WsProvider} from '@polkadot/api';
import {Keyring} from "@polkadot/keyring";
import {KeyringPair} from "@polkadot/keyring/types";
import {cryptoWaitReady} from '@polkadot/util-crypto';

export interface TestContext {
    api: ApiPromise;
    alice: KeyringPair;
    bob: KeyringPair;
}

chai.use(chaiAsPromised);

export const DISPLAY_LOG = process.env.MANTA_LOG || false;
export const MANTA_LOG = process.env.MANTA_LOG || "info";
export const MANTA_BUILD = process.env.MANTA_BUILD || "release";

export const BINARY_PATH = process.env.MANTA_BINARY || `../target/${MANTA_BUILD}/manta`;
export const SPAWNING_TIME = 120000;

export async function startMantaNode(): Promise<{ binary: ChildProcess; } & TestContext> {
    const P2P_PORT = await getPort({ port: getPort.makeRange(19931, 22000) });
    const RPC_PORT = await getPort({ port: getPort.makeRange(19931, 22000) });
    const WS_PORT = await getPort({ port: getPort.makeRange(19931, 22000) });

    console.log(`P2P: ${P2P_PORT}, RPC: ${RPC_PORT}, WS: ${WS_PORT}`);

    const cmd = BINARY_PATH;
    const args = [
        `--chain=manta-localdev`,
        `--alice`,
        `-lruntime=debug`,
        `--execution=native`, // Faster execution using native
        `--no-telemetry`,
        `--no-prometheus`,
        `--port=${P2P_PORT}`,
        `--rpc-port=${RPC_PORT}`,
        `--rpc-external`,
        `--ws-port=${WS_PORT}`,
        `--ws-external`,
        `--rpc-cors=all`,
        `--rpc-methods=unsafe`,
        `--pruning=archive`,
        `--keep-blocks=archive`,
        `--tmp`,
    ];
    const binary = spawn(cmd, args);

    binary.on("error", (err) => {
        if ((err as any).errno == "ENOENT") {
            console.error(
                `\x1b[31mMissing Acala binary (${BINARY_PATH}).\nPlease compile the Acala project:\nmake test-ts\x1b[0m`
            );
        } else {
            console.error(err);
        }
        process.exit(1);
    });

    const binaryLogs = [] as any;
    const { api, alice, bob } = await new Promise<TestContext>((resolve, reject) => {
        const timer = setTimeout(() => {
            console.error(`\x1b[31m Failed to start Acala Node.\x1b[0m`);
            console.error(`Command: ${cmd} ${args.join(" ")}`);
            console.error(`Logs:`);
            console.error(binaryLogs.map((chunk: any) => chunk.toString()).join("\n"));
            process.exit(1);
        }, SPAWNING_TIME - 2000);

        const onData = async (chunk: any) => {
            if (DISPLAY_LOG) {
                console.log(chunk.toString());
            }
            binaryLogs.push(chunk);
            if (chunk.toString().match(/best: #0/)) {
                try {
                    const { api, alice, bob } = await getTestUtils(`ws://127.0.0.1:${WS_PORT}`);

                    clearTimeout(timer);
                    if (!DISPLAY_LOG) {
                        binary.stderr.off("data", onData);
                        binary.stdout.off("data", onData);
                    }
                    resolve({ api, alice, bob });
                } catch(e) {
                    binary.kill();
                    reject(e);
                }
            }
        };
        binary.stderr.on("data", onData);
        binary.stdout.on("data", onData);
    });

    return { api, alice, bob, binary };
}

export function describeWithManta(title: string, cb: (context: TestContext) => void) {
    let context = {} as TestContext;

    describe(title, () => {
        let binary: ChildProcess;
        before("Starting Manta Test Node", async function () {
            console.log('starting manta node ...')
            this.timeout(SPAWNING_TIME);

            const init = await startMantaNode();

            context.api = init.api;
            context.alice = init.alice;
            context.bob = init.bob;
            binary = init.binary;

            console.log('manta node started!')
        });

        after(async function () {
            console.log(`\x1b[31m Killing RPC\x1b[0m`);
            await context.api.disconnect();
            binary.kill();
        });

        cb(context);
    });
}

export const getTestUtils = async (
    url = 'ws://localhost:9944',
): Promise<{
    api: ApiPromise;
    alice: KeyringPair;
    bob: KeyringPair;
}> => {
    const provider = new WsProvider(url);
    const api = await ApiPromise.create({provider: provider});
    await cryptoWaitReady();
    await api.isReady;
    
    const alice = new Keyring({type: 'sr25519'}).addFromUri("//Alice");
    const bob = new Keyring({type: 'sr25519'}).addFromUri("//Bob");

    return {
        api,
        alice,
        bob
    };
};

export const remark = async(context: TestContext) => {
    const alice = context.alice;

    const tx = await context.api.tx.system.remark("0x00");
    await tx.signAndSend(alice, {nonce: -1}, async ({ events = [], status, txHash, dispatchError }) => {
        if (dispatchError) {
            console.log(`extrinsic has error: ${dispatchError.toString()}, hex:${tx.toHex()}`);
        }
    });
    await new Promise(res => setTimeout(res, 200))
}

export const executeTx = async(
    context: TestContext,
    tx: any,
    sudo: boolean = false,
    useAlice: boolean = true
) => {
    let account;
    if(useAlice) {
        account = context.alice;
    } else {
        account = context.bob;
    }

    if (sudo) {
        const rootCall = context.api.tx.sudo.sudo(tx);
        await rootCall.signAndSend(context.alice, {nonce: -1}, async ({ events = [], status, txHash, dispatchError }) => {
            if (dispatchError) {
                console.log(`root extrinsic has error: ${dispatchError.toString()}, hex:${tx.toHex()}`);
            }
        });
    } else {
        // @ts-ignore
        await tx.signAndSend(account, {nonce: -1}, async ({ events = [], status, txHash, dispatchError }) => {
            if (dispatchError) {
                console.log(`extrinsic has error: ${dispatchError.toString()}, hex:${tx.toHex()}`);
            }
        });
    }
    await new Promise(res => setTimeout(res, 200))
}
