/**
 * Chameleon Wallet - TypeScript Type Definitions
 */

// Wallet Types
export interface Wallet {
  address: string;
  publicKey: string;
  name: string;
  createdAt: number;
  isHD: boolean; // Hierarchical Deterministic
  derivationPath?: string;
}

// Balance Information
export interface Balance {
  public: string;      // Public CHML balance (in base units)
  shielded: string;    // Shielded (private) balance (in base units)
  usdValue: string;    // USD equivalent
  lastUpdated: number; // Timestamp
}

// Transaction Types
export type TransactionType = 'send' | 'receive' | 'shield' | 'unshield' | 'stake' | 'unstake' | 'swap';
export type TransactionStatus = 'pending' | 'confirmed' | 'failed' | 'cancelled';

export interface Transaction {
  hash: string;
  type: TransactionType;
  amount: string;      // In base units
  timestamp: number;
  status: TransactionStatus;
  isPrivate: boolean;
  from?: string;       // Sender address
  to?: string;         // Recipient address
  fee?: string;        // Transaction fee
  memo?: string;       // Optional memo
  blockNumber?: number;
  confirmations?: number;
}

// Network Configuration
export interface NetworkConfig {
  name: string;
  rpcUrl: string;
  ss58Prefix: number;
  tokenSymbol: string;
  tokenDecimals: number;
  isTestnet: boolean;
}

// Validator Information
export interface Validator {
  address: string;
  name?: string;
  commission: string;  // Commission rate (0-100)
  totalStake: string;  // Total staked amount
  ownStake: string;    // Validator's own stake
  apy: string;         // Annual percentage yield
  isActive: boolean;
  uptime: string;      // Uptime percentage
  delegatorCount: number;
}

// Staking Information
export interface StakingInfo {
  validatorAddress: string;
  stakedAmount: string;
  rewards: string;
  unbondingAmount?: string;
  unbondingBlocks?: number; // Blocks remaining for unbonding
  isActive: boolean;
}

// DEX Types
export interface TokenPair {
  token0: string;      // Token symbol (e.g., 'CHML')
  token1: string;      // Token symbol (e.g., 'ETH')
  reserve0: string;    // Reserve of token0
  reserve1: string;    // Reserve of token1
  totalSupply: string; // LP token total supply
  fee: string;         // Trading fee (e.g., '0.003' for 0.3%)
}

export interface LiquidityPosition {
  pairAddress: string;
  token0: string;
  token1: string;
  lpTokens: string;    // LP tokens owned
  token0Amount: string; // Underlying token0 amount
  token1Amount: string; // Underlying token1 amount
  sharePercent: string; // Share of pool (0-100)
  rewards: string;     // Claimable rewards
}

// Swap Quote
export interface SwapQuote {
  inputAmount: string;
  outputAmount: string;
  priceImpact: string; // Price impact percentage
  fee: string;         // Trading fee
  route: string[];     // Token route for swap
  slippage: string;    // Slippage tolerance
}

// Security & Authentication
export interface BiometricConfig {
  isAvailable: boolean;
  isEnabled: boolean;
  type: 'FaceID' | 'TouchID' | 'Fingerprint' | 'None';
}

export interface SecuritySettings {
  biometric: BiometricConfig;
  pinEnabled: boolean;
  autoLockMinutes: number;
  screenshotBlocked: boolean;
}

// App State
export interface AppState {
  isInitialized: boolean;
  hasWallet: boolean;
  isLocked: boolean;
  currentNetwork: NetworkConfig;
  selectedWallet?: Wallet;
  balance?: Balance;
  transactions: Transaction[];
  validators: Validator[];
  stakingInfo?: StakingInfo;
  liquidityPositions: LiquidityPosition[];
  securitySettings: SecuritySettings;
}

// API Response Types
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp: number;
}

// RPC Types
export interface RpcCall {
  method: string;
  params: any[];
  id: number;
}

export interface RpcResponse<T = any> {
  jsonrpc: string;
  id: number;
  result?: T;
  error?: {
    code: number;
    message: string;
    data?: any;
  };
}

// Storage Types
export interface StoredWallet {
  address: string;
  name: string;
  createdAt: number;
  isHD: boolean;
  derivationPath?: string;
  // Note: Private keys are stored separately in secure keychain
}

export interface StoredTransaction {
  hash: string;
  type: TransactionType;
  amount: string;
  timestamp: number;
  status: TransactionStatus;
  isPrivate: boolean;
  from?: string;
  to?: string;
  fee?: string;
  memo?: string;
}

// Navigation Types
export type RootStackParamList = {
  Loading: undefined;
  Welcome: undefined;
  CreateWallet: undefined;
  ImportWallet: undefined;
  BackupSeed: { seed: string };
  ConfirmSeed: { seed: string };
  SetupPin: undefined;
  SetupBiometric: undefined;
  Unlock: undefined;
  Main: undefined;
  Send: { recipient?: string; amount?: string };
  Receive: undefined;
  TransactionDetail: { transaction: Transaction };
  ValidatorDetail: { validator: Validator };
  Stake: { validator?: Validator };
  Swap: { inputToken?: string; outputToken?: string };
  LiquidityAdd: { pair?: TokenPair };
  LiquidityRemove: { position: LiquidityPosition };
  Settings: undefined;
  Security: undefined;
  Network: undefined;
  About: undefined;
};

export type TabParamList = {
  Home: undefined;
  Staking: undefined;
  DEX: undefined;
  Settings: undefined;
};

// Hook Return Types
export interface UseWalletReturn {
  wallet: Wallet | null;
  balance: Balance | null;
  isLoading: boolean;
  error: string | null;
  createWallet: (name: string, seed?: string) => Promise<Wallet>;
  importWallet: (seed: string, name: string) => Promise<Wallet>;
  deleteWallet: () => Promise<void>;
  refreshBalance: () => Promise<void>;
}

export interface UseTransactionsReturn {
  transactions: Transaction[];
  isLoading: boolean;
  error: string | null;
  sendTransaction: (to: string, amount: string, isPrivate: boolean, memo?: string) => Promise<string>;
  shieldTokens: (amount: string) => Promise<string>;
  unshieldTokens: (amount: string) => Promise<string>;
  refreshTransactions: () => Promise<void>;
}

export interface UseStakingReturn {
  validators: Validator[];
  stakingInfo: StakingInfo | null;
  isLoading: boolean;
  error: string | null;
  stake: (validatorAddress: string, amount: string) => Promise<string>;
  unstake: (amount: string) => Promise<string>;
  claimRewards: () => Promise<string>;
  refreshStaking: () => Promise<void>;
}

// Utility Types
export type Prettify<T> = {
  [K in keyof T]: T[K];
} & {};

export type Optional<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;

export type RequiredFields<T, K extends keyof T> = T & Required<Pick<T, K>>;
