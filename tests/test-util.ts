import { xxhashAsU8a } from '@polkadot/util-crypto';
import type { HexString } from '@polkadot/util/types';
import { u8aToHex, u8aToBigInt, numberToU8a, nToU8a} from '@polkadot/util';

export enum HashType {
    Identity,
    TwoxConcat
}

/**
 * delay sometime (usually used for wait a block confirmation).
 * @param ms number of millisec to be delayed
 * @returns 
 */
export async function delay(ms: number) {
    return new Promise( resolve => setTimeout(resolve, ms) );
}

// emojis
export const emojis = {
    post: String.fromCodePoint(0x1F4EE),
    pending: String.fromCodePoint(0x1F69A),
    write: String.fromCodePoint(0x1F4DD),
}

/**
 * twox_128 hash.
 * @param data input data.
 * @returns hash value.
 */
function twox_128(data: HexString | Buffer | Uint8Array | string): Uint8Array {
    return xxhashAsU8a(data, 128);
}

/**
 * generate storage key for a single map storage item.
 * @param module_name name of the module, e.g. `MantaPay`.
 * @param variable_name storage variable name.
 * @param key key of the single map.
 * @param bitLength bit length.
 * @param hash_type type of hash.
 * @returns storage key.
 */
export function single_map_storage_key(
    module_name: string, 
    variable_name: string, 
    key: number, 
    bitLength: number, 
    hash_type: HashType 
) : string {
    const binary = new Uint8Array([
        ...twox_128(module_name), 
        ...twox_128(variable_name),
        ...hash_number(key, bitLength, hash_type)]);
    return u8aToHex(binary);
}

/**
 * hash a number to bytes according to `hash_type`.
 * @param data data to be hashed.
 * @param bit_length bit length.
 * @param hash_type type of hash.
 * @returns hash value.
 */
function hash_number(data: number, bit_length: number, hash_type: HashType): Uint8Array {
    switch (hash_type) {
        case HashType.Identity:
            return numberToU8a(data, bit_length).reverse();
        case HashType.TwoxConcat:
            return new Uint8Array([
                ...xxhashAsU8a(numberToU8a(data, bit_length).reverse(), 64),
                ...numberToU8a(data, bit_length).reverse()
            ])
    }
}

/**
 * generate storage key for a single map storage item.
 * @param module_name name of the module, e.g. `MantaPay`.
 * @param variable_name storage variable name.
 * @param key_1 first key of the double map.
 * @param bitLength_1 bit length of the first key.
 * @param hash_type_1 hash type of the first key.
 * @param key_2 second key for the double map.
 * @param bitLength_2 bit length of the second key.
 * @param hash_type_2 hash type of the second key.
 * @returns storage key of the item.
 */
export function double_map_storage_key(
    module_name: string,
    variable_name: string,
    key_1: number,
    bitLength_1: number,
    hash_type_1: HashType,
    key_2: number,
    bitLength_2: number,
    hash_type_2: HashType
): string {
    
    const binary = new Uint8Array([
        ...twox_128(module_name),
        ...twox_128(variable_name),
        ...hash_number(key_1, bitLength_1, hash_type_1),
        ...hash_number(key_2, bitLength_2, hash_type_2),
    ]);
    return u8aToHex(binary);
}
