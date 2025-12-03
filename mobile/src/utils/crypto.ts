/**
 * Chameleon Wallet - Cryptographic Utilities
 */

import { mnemonicGenerate, mnemonicValidate, mnemonicToMiniSecret, cryptoWaitReady } from '@polkadot/util-crypto';
import { Keyring } from '@polkadot/keyring';
import { u8aToHex, hexToU8a } from '@polkadot/util';
import { NETWORKS, DEFAULT_NETWORK } from '@/constants';
import type { NetworkConfig } from '@/types';

// Initialize crypto libraries
let cryptoReady = false;

/**
 * Initialize cryptographic libraries
 * Must be called before using any crypto functions
 */
export async function initializeCrypto(): Promise<void> {
  if (!cryptoReady) {
    await cryptoWaitReady();
    cryptoReady = true;
  }
}

/**
 * Generate a new BIP39 mnemonic seed phrase
 * @param wordCount Number of words (12 or 24)
 * @returns Generated mnemonic seed phrase
 */
export function generateSeedPhrase(wordCount: 12 | 24 = 12): string {
  if (!cryptoReady) {
    throw new Error('Crypto not initialized. Call initializeCrypto() first.');
  }
  
  return mnemonicGenerate(wordCount);
}

/**
 * Validate a BIP39 mnemonic seed phrase
 * @param seedPhrase Seed phrase to validate
 * @returns True if valid, false otherwise
 */
export function validateSeedPhrase(seedPhrase: string): boolean {
  if (!cryptoReady) {
    throw new Error('Crypto not initialized. Call initializeCrypto() first.');
  }
  
  try {
    return mnemonicValidate(seedPhrase.trim());
  } catch {
    return false;
  }
}

/**
 * Create a keyring for the specified network
 * @param network Network configuration
 * @returns Configured keyring
 */
export function createKeyring(network: NetworkConfig = DEFAULT_NETWORK): Keyring {
  if (!cryptoReady) {
    throw new Error('Crypto not initialized. Call initializeCrypto() first.');
  }
  
  return new Keyring({
    type: 'sr25519',
    ss58Format: network.ss58Prefix,
  });
}

/**
 * Derive a keypair from seed phrase
 * @param seedPhrase BIP39 seed phrase
 * @param derivationPath HD derivation path (optional)
 * @param network Network configuration
 * @returns Keypair with address and public key
 */
export function deriveKeypairFromSeed(
  seedPhrase: string,
  derivationPath?: string,
  network: NetworkConfig = DEFAULT_NETWORK
): { address: string; publicKey: string; keypair: any } {
  if (!validateSeedPhrase(seedPhrase)) {
    throw new Error('Invalid seed phrase');
  }
  
  const keyring = createKeyring(network);
  const keypair = keyring.addFromMnemonic(seedPhrase, {}, 'sr25519');
  
  return {
    address: keypair.address,
    publicKey: u8aToHex(keypair.publicKey),
    keypair,
  };
}

/**
 * Generate HD wallet addresses from seed
 * @param seedPhrase BIP39 seed phrase
 * @param count Number of addresses to generate
 * @param network Network configuration
 * @returns Array of derived addresses
 */
export function generateHDAddresses(
  seedPhrase: string,
  count: number = 5,
  network: NetworkConfig = DEFAULT_NETWORK
): Array<{ address: string; publicKey: string; derivationPath: string }> {
  if (!validateSeedPhrase(seedPhrase)) {
    throw new Error('Invalid seed phrase');
  }
  
  const keyring = createKeyring(network);
  const addresses = [];
  
  for (let i = 0; i < count; i++) {
    const derivationPath = `//chameleon//wallet//${i}`;
    const keypair = keyring.addFromMnemonic(`${seedPhrase}${derivationPath}`, {}, 'sr25519');
    
    addresses.push({
      address: keypair.address,
      publicKey: u8aToHex(keypair.publicKey),
      derivationPath,
    });
  }
  
  return addresses;
}

/**
 * Sign a message with the keypair
 * @param message Message to sign
 * @param seedPhrase Seed phrase
 * @param derivationPath HD derivation path (optional)
 * @param network Network configuration
 * @returns Signature as hex string
 */
export function signMessage(
  message: string | Uint8Array,
  seedPhrase: string,
  derivationPath?: string,
  network: NetworkConfig = DEFAULT_NETWORK
): string {
  const { keypair } = deriveKeypairFromSeed(seedPhrase, derivationPath, network);
  
  const messageU8a = typeof message === 'string' ? new TextEncoder().encode(message) : message;
  const signature = keypair.sign(messageU8a);
  
  return u8aToHex(signature);
}

/**
 * Verify a signature
 * @param message Original message
 * @param signature Signature to verify
 * @param publicKey Public key of signer
 * @returns True if signature is valid
 */
export function verifySignature(
  message: string | Uint8Array,
  signature: string,
  publicKey: string
): boolean {
  try {
    const keyring = createKeyring();
    const messageU8a = typeof message === 'string' ? new TextEncoder().encode(message) : message;
    const signatureU8a = hexToU8a(signature);
    const publicKeyU8a = hexToU8a(publicKey);
    
    return keyring.verify(messageU8a, signatureU8a, publicKeyU8a);
  } catch {
    return false;
  }
}

/**
 * Generate a random salt for encryption
 * @param length Salt length in bytes
 * @returns Random salt as hex string
 */
export function generateSalt(length: number = 32): string {
  const array = new Uint8Array(length);
  crypto.getRandomValues(array);
  return u8aToHex(array);
}

/**
 * Hash a password with salt (for PIN storage)
 * @param password Password to hash
 * @param salt Salt for hashing
 * @returns Hashed password as hex string
 */
export async function hashPassword(password: string, salt?: string): Promise<{ hash: string; salt: string }> {
  const actualSalt = salt || generateSalt();
  const encoder = new TextEncoder();
  const data = encoder.encode(password + actualSalt);
  
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hash = u8aToHex(new Uint8Array(hashBuffer));
  
  return { hash, salt: actualSalt };
}

/**
 * Verify a password against its hash
 * @param password Password to verify
 * @param hash Stored hash
 * @param salt Salt used for hashing
 * @returns True if password matches
 */
export async function verifyPassword(password: string, hash: string, salt: string): Promise<boolean> {
  const { hash: computedHash } = await hashPassword(password, salt);
  return computedHash === hash;
}

/**
 * Encrypt data using AES-GCM
 * @param data Data to encrypt
 * @param password Password for encryption
 * @returns Encrypted data with IV
 */
export async function encryptData(data: string, password: string): Promise<{ encrypted: string; iv: string }> {
  const encoder = new TextEncoder();
  const dataBuffer = encoder.encode(data);
  
  // Generate key from password
  const passwordBuffer = encoder.encode(password);
  const keyMaterial = await crypto.subtle.importKey(
    'raw',
    passwordBuffer,
    { name: 'PBKDF2' },
    false,
    ['deriveKey']
  );
  
  const salt = crypto.getRandomValues(new Uint8Array(16));
  const key = await crypto.subtle.deriveKey(
    {
      name: 'PBKDF2',
      salt,
      iterations: 100000,
      hash: 'SHA-256',
    },
    keyMaterial,
    { name: 'AES-GCM', length: 256 },
    false,
    ['encrypt']
  );
  
  // Encrypt data
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const encryptedBuffer = await crypto.subtle.encrypt(
    { name: 'AES-GCM', iv },
    key,
    dataBuffer
  );
  
  return {
    encrypted: u8aToHex(new Uint8Array(encryptedBuffer)),
    iv: u8aToHex(iv),
  };
}

/**
 * Decrypt data using AES-GCM
 * @param encryptedData Encrypted data
 * @param iv Initialization vector
 * @param password Password for decryption
 * @returns Decrypted data
 */
export async function decryptData(encryptedData: string, iv: string, password: string): Promise<string> {
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();
  
  // Generate key from password
  const passwordBuffer = encoder.encode(password);
  const keyMaterial = await crypto.subtle.importKey(
    'raw',
    passwordBuffer,
    { name: 'PBKDF2' },
    false,
    ['deriveKey']
  );
  
  const salt = crypto.getRandomValues(new Uint8Array(16));
  const key = await crypto.subtle.deriveKey(
    {
      name: 'PBKDF2',
      salt,
      iterations: 100000,
      hash: 'SHA-256',
    },
    keyMaterial,
    { name: 'AES-GCM', length: 256 },
    false,
    ['decrypt']
  );
  
  // Decrypt data
  const encryptedBuffer = hexToU8a(encryptedData);
  const ivBuffer = hexToU8a(iv);
  
  const decryptedBuffer = await crypto.subtle.decrypt(
    { name: 'AES-GCM', iv: ivBuffer },
    key,
    encryptedBuffer
  );
  
  return decoder.decode(decryptedBuffer);
}

/**
 * Generate a secure random PIN
 * @param length PIN length
 * @returns Random PIN as string
 */
export function generateSecurePin(length: number = 6): string {
  const digits = '0123456789';
  let pin = '';
  
  for (let i = 0; i < length; i++) {
    const randomIndex = Math.floor(Math.random() * digits.length);
    pin += digits[randomIndex];
  }
  
  return pin;
}

/**
 * Validate address format
 * @param address Address to validate
 * @param network Network configuration
 * @returns True if address is valid
 */
export function validateAddress(address: string, network: NetworkConfig = DEFAULT_NETWORK): boolean {
  try {
    const keyring = createKeyring(network);
    keyring.decodeAddress(address);
    return true;
  } catch {
    return false;
  }
}

/**
 * Convert address between different SS58 formats
 * @param address Address to convert
 * @param targetNetwork Target network configuration
 * @param sourceNetwork Source network configuration
 * @returns Converted address
 */
export function convertAddress(
  address: string,
  targetNetwork: NetworkConfig,
  sourceNetwork: NetworkConfig = DEFAULT_NETWORK
): string {
  const sourceKeyring = createKeyring(sourceNetwork);
  const targetKeyring = createKeyring(targetNetwork);
  
  const publicKey = sourceKeyring.decodeAddress(address);
  return targetKeyring.encodeAddress(publicKey);
}
