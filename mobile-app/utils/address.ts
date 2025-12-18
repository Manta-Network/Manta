/**
 * Address validation and formatting utilities
 */

import { decodeAddress, encodeAddress, isAddress } from '@polkadot/util-crypto';
import { NETWORK_CONFIG } from '../config/network';

/**
 * Validate if a string is a valid Substrate address
 */
export function isValidAddress(address: string): boolean {
  try {
    return isAddress(address);
  } catch {
    return false;
  }
}

/**
 * Format address for the current network's SS58 prefix
 */
export function formatAddress(address: string): string {
  try {
    if (!isValidAddress(address)) {
      throw new Error('Invalid address');
    }
    
    const decoded = decodeAddress(address);
    return encodeAddress(decoded, NETWORK_CONFIG.ss58Prefix);
  } catch (error) {
    console.error('Error formatting address:', error);
    return address; // Return original if formatting fails
  }
}

/**
 * Truncate address for display (e.g., "5GrwvaEF...KutQY")
 */
export function truncateAddress(address: string, startLength = 8, endLength = 6): string {
  if (!address || address.length <= startLength + endLength) {
    return address;
  }
  
  return `${address.substring(0, startLength)}...${address.substring(address.length - endLength)}`;
}

/**
 * Generate a random test address (for development)
 */
export function generateTestAddress(): string {
  // This is Alice's address on Substrate networks
  return '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
}

/**
 * Common test addresses for development
 */
export const TEST_ADDRESSES = {
  alice: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
  bob: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
  charlie: '5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y',
  dave: '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy',
  eve: '5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw',
};
