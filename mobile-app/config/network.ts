/**
 * Network configuration for Chameleon Network
 */

export const NETWORK_CONFIG = {
  name: 'Chameleon Devnet',
  isTestnet: true,
  httpEndpoint: 'http://64.23.233.36:9933',
  wsEndpoint: 'ws://64.23.233.36:9944',
  tokenSymbol: 'CHML',
  tokenDecimals: 18,
  ss58Prefix: 42, // Default Substrate prefix
};

// Connection settings
export const CONNECTION_CONFIG = {
  reconnectAttempts: 5,
  reconnectDelay: 3000, // 3 seconds
  connectionTimeout: 10000, // 10 seconds
};

// Chain constants
export const CHAIN_CONSTANTS = {
  blockTime: 6000, // 6 seconds in milliseconds
  existentialDeposit: '1000000000000000000', // 1 CHML in planck units
};
