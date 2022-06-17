import { xxhashAsU8a } from '@polkadot/util-crypto';
import type { HexString } from '@polkadot/util/types';
import { u8aToHex, u8aToBigInt, numberToU8a, nToU8a} from '@polkadot/util';

export enum HashType {
    Identity
};

// delay sometime (usually used for wait a block confirmation)
export async function delay(ms: number) {
    return new Promise( resolve => setTimeout(resolve, ms) );
}

function twox_128(data: HexString | Buffer | Uint8Array | string): Uint8Array {
    return xxhashAsU8a(data, 128);
}

// only support Identity hash type for now
export function single_map_storage_key(
    module_name: string, 
    variable_name: string, 
    key: number, 
    bitLength: number, 
    hash_type: HashType 
) : String {
    if(hash_type !== HashType.Identity){
        throw Error("HashType can only be Identity");
    }
    let binary = new Uint8Array([
        ...twox_128(module_name), 
        ...twox_128(variable_name),
        ...numberToU8a(key, bitLength).reverse()]);
    return u8aToHex(binary);
}

// only support Identity hash type for now
export function double_map_storage_key(
    module_name: string,
    variable_name: string,
    key_1: number,
    bitLength_1: number,
    hash_type_1: HashType,
    key_2: number,
    bitLength_2: number,
    hash_type_2: HashType
): String {
    if(hash_type_1 !== HashType.Identity || hash_type_2 !== HashType.Identity){
        throw Error("HashType can only be Identity");
    }
    let binary = new Uint8Array([
        ...twox_128(module_name),
        ...twox_128(variable_name),
        ...numberToU8a(key_1, bitLength_1).reverse(),
        ...numberToU8a(key_2, bitLength_2).reverse()
    ]);
    return u8aToHex(binary);
}

export function bigintToMadEncoding(input: BigInt): BigInt {
    return u8aToBigInt(nToU8a(input.valueOf()).reverse())
}