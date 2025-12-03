/**
 * Chameleon Network - Constants and Configuration
 * Based on official Chameleon Network specifications
 */

import { NetworkConfig } from '@/types';

// Chameleon Token Constants
export const CHAMELEON = {
  TOKEN_SYMBOL: 'CHML',
  TOKEN_NAME: 'Chameleon Network Token',
  DECIMALS: 18,
  TOTAL_SUPPLY: '100000000', // 100M CHML
  MIN_VALIDATOR_STAKE: '1750', // 1,750 CHML
  EXISTENTIAL_DEPOSIT: '0.001', // 0.001 CHML
} as const;

// Network Configuration
export const NETWORKS: Record<string, NetworkConfig> = {
  mainnet: {
    name: 'Chameleon Mainnet',
    rpcUrl: 'wss://rpc.chameleon.network',
    ss58Prefix: 99,
    tokenSymbol: 'CHML',
    tokenDecimals: 18,
    isTestnet: false,
  },
  testnet: {
    name: 'Chameleon Testnet',
    rpcUrl: 'wss://testnet.chameleon.network',
    ss58Prefix: 99,
    tokenSymbol: 'CHMLTest',
    tokenDecimals: 18,
    isTestnet: true,
  },
  local: {
    name: 'Local Devnet',
    rpcUrl: 'ws://localhost:9944',
    ss58Prefix: 99,
    tokenSymbol: 'CHML',
    tokenDecimals: 18,
    isTestnet: true,
  },
} as const;

// Default network (testnet for development)
export const DEFAULT_NETWORK = NETWORKS.testnet;

// Fee Structure (based on Chameleon specs)
export const FEES = {
  BASE_GAS: '0.1',         // 0.1 CHML base gas fee
  SHIELD_MIN: '0.1',       // 0.1 CHML minimum shield fee
  UNSHIELD_MIN: '0.1',     // 0.1 CHML minimum unshield fee
  SHIELD_PERCENT: 0.0002,  // 0.02% shield fee
  UNSHIELD_PERCENT: 0.0005, // 0.05% unshield fee
  PDEX_SWAP: 0.0025,       // 0.25% pDEX swap fee
} as const;

// Time Constants
export const TIME = {
  BLOCK_TIME: 6, // 6 seconds per block
  BLOCKS_PER_MINUTE: 10,
  BLOCKS_PER_HOUR: 600,
  BLOCKS_PER_DAY: 14400,
  UNBONDING_PERIOD_DAYS: 14,
  UNBONDING_PERIOD_BLOCKS: 201600, // 14 days * 14400 blocks/day
  SESSION_LENGTH_HOURS: 4,
  SESSION_LENGTH_BLOCKS: 2400, // 4 hours * 600 blocks/hour
} as const;

// Staking Constants
export const STAKING = {
  MIN_VALIDATOR_STAKE: '1750', // 1,750 CHML
  MAX_VALIDATORS: 200,
  MIN_VALIDATORS: 5,
  MAX_DELEGATIONS_PER_DELEGATOR: 100,
  MAX_DELEGATORS_PER_VALIDATOR: 500,
  SLASHING_DOWNTIME: 0.001,    // 0.1%
  SLASHING_DOUBLE_SIGN: 0.05,  // 5.0%
} as const;

// Privacy Constants
export const PRIVACY = {
  DEFAULT_RING_SIZE: 11,       // Ring signature size
  MIN_SHIELD_AMOUNT: '0.1',    // Minimum amount to shield
  MIN_UNSHIELD_AMOUNT: '0.1',  // Minimum amount to unshield
  PROOF_GENERATION_TIMEOUT: 30000, // 30 seconds
  PROOF_VERIFICATION_TIMEOUT: 5000,  // 5 seconds
} as const;

// App Configuration
export const APP_CONFIG = {
  VERSION: '1.0.0',
  BUILD_NUMBER: 1,
  MIN_PIN_LENGTH: 6,
  MAX_PIN_LENGTH: 8,
  AUTO_LOCK_OPTIONS: [1, 5, 15, 30, 60], // Minutes
  DEFAULT_AUTO_LOCK: 5, // 5 minutes
  TRANSACTION_HISTORY_LIMIT: 100,
  BALANCE_REFRESH_INTERVAL: 30000, // 30 seconds
  PRICE_REFRESH_INTERVAL: 60000,   // 1 minute
} as const;

// Storage Keys
export const STORAGE_KEYS = {
  WALLET_DATA: 'chameleon_wallet_data',
  TRANSACTIONS: 'chameleon_transactions',
  SETTINGS: 'chameleon_settings',
  SECURITY: 'chameleon_security',
  NETWORK: 'chameleon_network',
  PRICE_CACHE: 'chameleon_price_cache',
} as const;

// Keychain Keys (for secure storage)
export const KEYCHAIN_KEYS = {
  SEED_PHRASE: 'chameleon_seed_phrase',
  PRIVATE_KEY: 'chameleon_private_key',
  PIN_HASH: 'chameleon_pin_hash',
  BIOMETRIC_KEY: 'chameleon_biometric_key',
} as const;

// API Endpoints
export const API_ENDPOINTS = {
  PRICE_API: 'https://api.coingecko.com/api/v3/simple/price',
  CHAMELEON_API: 'https://api.chameleon.network/v1',
  TESTNET_FAUCET: 'https://faucet.chameleon.network',
} as const;

// UI Constants
export const UI = {
  ANIMATION_DURATION: 300,
  HAPTIC_FEEDBACK: true,
  THEME_TRANSITION_DURATION: 200,
  LOADING_TIMEOUT: 30000, // 30 seconds
  ERROR_DISPLAY_DURATION: 5000, // 5 seconds
} as const;

// Validation Rules
export const VALIDATION = {
  MIN_SEND_AMOUNT: '0.001',    // Minimum send amount
  MAX_SEND_AMOUNT: '1000000',  // Maximum send amount (1M CHML)
  ADDRESS_LENGTH: 48,          // SS58 address length
  SEED_PHRASE_WORDS: [12, 24], // Supported seed phrase lengths
  PIN_ATTEMPTS: 5,             // Max PIN attempts before lock
  BIOMETRIC_ATTEMPTS: 3,       // Max biometric attempts
} as const;

// Error Messages
export const ERROR_MESSAGES = {
  NETWORK_ERROR: 'Network connection failed. Please check your internet connection.',
  INVALID_ADDRESS: 'Invalid wallet address format.',
  INSUFFICIENT_BALANCE: 'Insufficient balance for this transaction.',
  INVALID_AMOUNT: 'Invalid transaction amount.',
  TRANSACTION_FAILED: 'Transaction failed. Please try again.',
  SEED_PHRASE_INVALID: 'Invalid seed phrase. Please check and try again.',
  PIN_INCORRECT: 'Incorrect PIN. Please try again.',
  BIOMETRIC_FAILED: 'Biometric authentication failed.',
  WALLET_LOCKED: 'Wallet is locked. Please unlock to continue.',
  PROOF_GENERATION_FAILED: 'Privacy proof generation failed. Please try again.',
} as const;

// Success Messages
export const SUCCESS_MESSAGES = {
  WALLET_CREATED: 'Wallet created successfully!',
  WALLET_IMPORTED: 'Wallet imported successfully!',
  TRANSACTION_SENT: 'Transaction sent successfully!',
  BACKUP_COMPLETED: 'Seed phrase backup completed.',
  SETTINGS_SAVED: 'Settings saved successfully.',
  STAKE_SUCCESSFUL: 'Staking transaction successful!',
  UNSTAKE_SUCCESSFUL: 'Unstaking transaction successful!',
} as const;

// Feature Flags (for gradual rollout)
export const FEATURE_FLAGS = {
  BIOMETRIC_AUTH: true,
  PRIVACY_TRANSACTIONS: true,
  STAKING: true,
  PDEX: true,
  CROSS_CHAIN_BRIDGE: false, // Coming in Phase 2
  GOVERNANCE: false,         // Coming in Phase 3
  NFT_SUPPORT: false,        // Future feature
} as const;

// Development/Debug
export const DEBUG = {
  ENABLE_LOGS: __DEV__,
  LOG_LEVEL: 'info' as 'debug' | 'info' | 'warn' | 'error',
  MOCK_BIOMETRICS: __DEV__,
  SKIP_ONBOARDING: false,
  TESTNET_ONLY: true, // Force testnet in development
} as const;
