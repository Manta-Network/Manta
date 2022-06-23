import { xxhashAsU8a } from '@polkadot/util-crypto';
import type { HexString } from '@polkadot/util/types';
import { u8aToHex, u8aToBigInt, numberToU8a, nToU8a} from '@polkadot/util';

export enum HashType {
    Identity,
    TwoxConcat
};

// delay sometime (usually used for wait a block confirmation)
export async function delay(ms: number) {
    return new Promise( resolve => setTimeout(resolve, ms) );
}

// emojis
export const emojis = {
    post: String.fromCodePoint(0x1F4EE),
    pending: String.fromCodePoint(0x1F69A),
    write: String.fromCodePoint(0x1F4DD),
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
    let binary = new Uint8Array([
        ...twox_128(module_name), 
        ...twox_128(variable_name),
        ...hash_number(key, bitLength, hash_type)]);
    return u8aToHex(binary);
}

function hash_number(key: number, bit_length: number, hash_type: HashType): Uint8Array {
    switch (hash_type) {
        case HashType.Identity:
            return numberToU8a(key, bit_length).reverse();
        case HashType.TwoxConcat:
            return new Uint8Array([
                ...xxhashAsU8a(numberToU8a(key, bit_length).reverse(), 64),
                ...numberToU8a(key, bit_length).reverse()
            ])
    }
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
    
    let binary = new Uint8Array([
        ...twox_128(module_name),
        ...twox_128(variable_name),
        ...hash_number(key_1, bitLength_1, hash_type_1),
        ...hash_number(key_2, bitLength_2, hash_type_2),
    ]);
    return u8aToHex(binary);
}

export function bigintToMadEncoding(input: BigInt): BigInt {
    return u8aToBigInt(nToU8a(input.valueOf()).reverse())
}