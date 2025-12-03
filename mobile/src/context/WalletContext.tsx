/**
 * Chameleon Wallet - Wallet Context Provider
 */

import React, { createContext, useContext, useReducer, useEffect, ReactNode } from 'react';
import { walletService } from '@/services/wallet';
import { AppStorage } from '@/services/storage';
import { DEFAULT_NETWORK } from '@/constants';
import type {
  AppState,
  Wallet,
  Balance,
  Transaction,
  Validator,
  StakingInfo,
  LiquidityPosition,
  SecuritySettings,
  NetworkConfig,
} from '@/types';

// Action types
type WalletAction =
  | { type: 'SET_INITIALIZED'; payload: boolean }
  | { type: 'SET_WALLET'; payload: Wallet | null }
  | { type: 'SET_LOCKED'; payload: boolean }
  | { type: 'SET_BALANCE'; payload: Balance | null }
  | { type: 'SET_TRANSACTIONS'; payload: Transaction[] }
  | { type: 'ADD_TRANSACTION'; payload: Transaction }
  | { type: 'UPDATE_TRANSACTION'; payload: { hash: string; updates: Partial<Transaction> } }
  | { type: 'SET_VALIDATORS'; payload: Validator[] }
  | { type: 'SET_STAKING_INFO'; payload: StakingInfo | null }
  | { type: 'SET_LIQUIDITY_POSITIONS'; payload: LiquidityPosition[] }
  | { type: 'SET_SECURITY_SETTINGS'; payload: SecuritySettings }
  | { type: 'SET_NETWORK'; payload: NetworkConfig }
  | { type: 'SET_ERROR'; payload: string | null }
  | { type: 'SET_LOADING'; payload: boolean };

// Initial state
const initialState: AppState = {
  isInitialized: false,
  hasWallet: false,
  isLocked: true,
  currentNetwork: DEFAULT_NETWORK,
  selectedWallet: undefined,
  balance: undefined,
  transactions: [],
  validators: [],
  stakingInfo: undefined,
  liquidityPositions: [],
  securitySettings: {
    biometric: {
      isAvailable: false,
      isEnabled: false,
      type: 'None',
    },
    pinEnabled: false,
    autoLockMinutes: 5,
    screenshotBlocked: true,
  },
};

// Reducer function
function walletReducer(state: AppState, action: WalletAction): AppState {
  switch (action.type) {
    case 'SET_INITIALIZED':
      return { ...state, isInitialized: action.payload };
    
    case 'SET_WALLET':
      return {
        ...state,
        selectedWallet: action.payload || undefined,
        hasWallet: action.payload !== null,
      };
    
    case 'SET_LOCKED':
      return { ...state, isLocked: action.payload };
    
    case 'SET_BALANCE':
      return { ...state, balance: action.payload || undefined };
    
    case 'SET_TRANSACTIONS':
      return { ...state, transactions: action.payload };
    
    case 'ADD_TRANSACTION':
      return {
        ...state,
        transactions: [action.payload, ...state.transactions],
      };
    
    case 'UPDATE_TRANSACTION':
      return {
        ...state,
        transactions: state.transactions.map(tx =>
          tx.hash === action.payload.hash
            ? { ...tx, ...action.payload.updates }
            : tx
        ),
      };
    
    case 'SET_VALIDATORS':
      return { ...state, validators: action.payload };
    
    case 'SET_STAKING_INFO':
      return { ...state, stakingInfo: action.payload || undefined };
    
    case 'SET_LIQUIDITY_POSITIONS':
      return { ...state, liquidityPositions: action.payload };
    
    case 'SET_SECURITY_SETTINGS':
      return { ...state, securitySettings: action.payload };
    
    case 'SET_NETWORK':
      return { ...state, currentNetwork: action.payload };
    
    default:
      return state;
  }
}

// Context type
interface WalletContextType {
  state: AppState;
  dispatch: React.Dispatch<WalletAction>;
  
  // Wallet actions
  initializeWallet: () => Promise<void>;
  createWallet: (name: string, seedPhrase?: string) => Promise<Wallet>;
  importWallet: (seedPhrase: string, name: string) => Promise<Wallet>;
  deleteWallet: () => Promise<void>;
  lockWallet: () => void;
  unlockWallet: () => void;
  
  // Balance actions
  refreshBalance: () => Promise<void>;
  
  // Transaction actions
  sendTransaction: (to: string, amount: string, isPrivate: boolean, memo?: string) => Promise<string>;
  shieldTokens: (amount: string) => Promise<string>;
  unshieldTokens: (amount: string) => Promise<string>;
  refreshTransactions: () => Promise<void>;
  
  // Staking actions
  stakeTokens: (validatorAddress: string, amount: string) => Promise<string>;
  unstakeTokens: (amount: string) => Promise<string>;
  refreshStaking: () => Promise<void>;
  
  // Settings actions
  updateSecuritySettings: (settings: Partial<SecuritySettings>) => Promise<void>;
  switchNetwork: (network: NetworkConfig) => Promise<void>;
}

// Create context
const WalletContext = createContext<WalletContextType | undefined>(undefined);

// Provider component
interface WalletProviderProps {
  children: ReactNode;
}

export function WalletProvider({ children }: WalletProviderProps) {
  const [state, dispatch] = useReducer(walletReducer, initialState);

  // Initialize wallet service
  const initializeWallet = async (): Promise<void> => {
    try {
      await walletService.initialize();
      
      const currentWallet = walletService.getCurrentWallet();
      dispatch({ type: 'SET_WALLET', payload: currentWallet });
      
      if (currentWallet) {
        // Load initial data
        await Promise.all([
          refreshBalance(),
          refreshTransactions(),
          refreshStaking(),
        ]);
      }
      
      // Load security settings
      const securitySettings = await AppStorage.getSecuritySettings();
      if (securitySettings) {
        dispatch({ type: 'SET_SECURITY_SETTINGS', payload: securitySettings });
      }
      
      // Load network config
      const networkConfig = await AppStorage.getNetworkConfig();
      if (networkConfig) {
        dispatch({ type: 'SET_NETWORK', payload: networkConfig });
      }
      
      dispatch({ type: 'SET_INITIALIZED', payload: true });
    } catch (error) {
      console.error('Failed to initialize wallet:', error);
      dispatch({ type: 'SET_INITIALIZED', payload: true }); // Still mark as initialized
    }
  };

  // Create wallet
  const createWallet = async (name: string, seedPhrase?: string): Promise<Wallet> => {
    const wallet = await walletService.createWallet(name, seedPhrase);
    dispatch({ type: 'SET_WALLET', payload: wallet });
    dispatch({ type: 'SET_LOCKED', payload: false });
    
    // Load initial balance
    await refreshBalance();
    
    return wallet;
  };

  // Import wallet
  const importWallet = async (seedPhrase: string, name: string): Promise<Wallet> => {
    const wallet = await walletService.importWallet(seedPhrase, name);
    dispatch({ type: 'SET_WALLET', payload: wallet });
    dispatch({ type: 'SET_LOCKED', payload: false });
    
    // Load initial data
    await Promise.all([
      refreshBalance(),
      refreshTransactions(),
    ]);
    
    return wallet;
  };

  // Delete wallet
  const deleteWallet = async (): Promise<void> => {
    await walletService.deleteWallet();
    dispatch({ type: 'SET_WALLET', payload: null });
    dispatch({ type: 'SET_BALANCE', payload: null });
    dispatch({ type: 'SET_TRANSACTIONS', payload: [] });
    dispatch({ type: 'SET_STAKING_INFO', payload: null });
  };

  // Lock wallet
  const lockWallet = (): void => {
    dispatch({ type: 'SET_LOCKED', payload: true });
  };

  // Unlock wallet
  const unlockWallet = (): void => {
    dispatch({ type: 'SET_LOCKED', payload: false });
  };

  // Refresh balance
  const refreshBalance = async (): Promise<void> => {
    try {
      const balance = await walletService.getBalance();
      dispatch({ type: 'SET_BALANCE', payload: balance });
    } catch (error) {
      console.error('Failed to refresh balance:', error);
    }
  };

  // Send transaction
  const sendTransaction = async (
    to: string,
    amount: string,
    isPrivate: boolean,
    memo?: string
  ): Promise<string> => {
    const txHash = await walletService.sendTransaction(to, amount, isPrivate, memo);
    
    // Add pending transaction to state
    const transaction: Transaction = {
      hash: txHash,
      type: 'send',
      amount,
      timestamp: Date.now(),
      status: 'pending',
      isPrivate,
      from: state.selectedWallet?.address,
      to,
      memo,
    };
    
    dispatch({ type: 'ADD_TRANSACTION', payload: transaction });
    
    return txHash;
  };

  // Shield tokens
  const shieldTokens = async (amount: string): Promise<string> => {
    const txHash = await walletService.shieldTokens(amount);
    
    const transaction: Transaction = {
      hash: txHash,
      type: 'shield',
      amount,
      timestamp: Date.now(),
      status: 'pending',
      isPrivate: true,
      from: state.selectedWallet?.address,
      to: state.selectedWallet?.address,
    };
    
    dispatch({ type: 'ADD_TRANSACTION', payload: transaction });
    
    return txHash;
  };

  // Unshield tokens
  const unshieldTokens = async (amount: string): Promise<string> => {
    const txHash = await walletService.unshieldTokens(amount);
    
    const transaction: Transaction = {
      hash: txHash,
      type: 'unshield',
      amount,
      timestamp: Date.now(),
      status: 'pending',
      isPrivate: false,
      from: state.selectedWallet?.address,
      to: state.selectedWallet?.address,
    };
    
    dispatch({ type: 'ADD_TRANSACTION', payload: transaction });
    
    return txHash;
  };

  // Refresh transactions
  const refreshTransactions = async (): Promise<void> => {
    try {
      const transactions = await walletService.getTransactionHistory();
      dispatch({ type: 'SET_TRANSACTIONS', payload: transactions });
    } catch (error) {
      console.error('Failed to refresh transactions:', error);
    }
  };

  // Stake tokens
  const stakeTokens = async (validatorAddress: string, amount: string): Promise<string> => {
    const txHash = await walletService.stakeTokens(validatorAddress, amount);
    
    const transaction: Transaction = {
      hash: txHash,
      type: 'stake',
      amount,
      timestamp: Date.now(),
      status: 'pending',
      isPrivate: false,
      from: state.selectedWallet?.address,
      to: validatorAddress,
    };
    
    dispatch({ type: 'ADD_TRANSACTION', payload: transaction });
    
    return txHash;
  };

  // Unstake tokens
  const unstakeTokens = async (amount: string): Promise<string> => {
    const txHash = await walletService.unstakeTokens(amount);
    
    const transaction: Transaction = {
      hash: txHash,
      type: 'unstake',
      amount,
      timestamp: Date.now(),
      status: 'pending',
      isPrivate: false,
      from: state.selectedWallet?.address,
    };
    
    dispatch({ type: 'ADD_TRANSACTION', payload: transaction });
    
    return txHash;
  };

  // Refresh staking info
  const refreshStaking = async (): Promise<void> => {
    try {
      // This would be implemented when staking is available
      // const stakingInfo = await rpcService.getStakingInfo(state.selectedWallet?.address);
      // dispatch({ type: 'SET_STAKING_INFO', payload: stakingInfo });
    } catch (error) {
      console.error('Failed to refresh staking info:', error);
    }
  };

  // Update security settings
  const updateSecuritySettings = async (settings: Partial<SecuritySettings>): Promise<void> => {
    const updatedSettings = { ...state.securitySettings, ...settings };
    await AppStorage.storeSecuritySettings(updatedSettings);
    dispatch({ type: 'SET_SECURITY_SETTINGS', payload: updatedSettings });
  };

  // Switch network
  const switchNetwork = async (network: NetworkConfig): Promise<void> => {
    await AppStorage.storeNetworkConfig(network);
    dispatch({ type: 'SET_NETWORK', payload: network });
    
    // Reconnect to new network and refresh data
    if (state.selectedWallet) {
      await Promise.all([
        refreshBalance(),
        refreshTransactions(),
        refreshStaking(),
      ]);
    }
  };

  // Initialize on mount
  useEffect(() => {
    initializeWallet();
  }, []);

  // Subscribe to balance updates
  useEffect(() => {
    if (!state.selectedWallet || state.isLocked) return;

    let unsubscribe: (() => void) | null = null;

    try {
      unsubscribe = walletService.subscribeToBalance((balance) => {
        dispatch({ type: 'SET_BALANCE', payload: balance });
      });
    } catch (error) {
      console.error('Failed to subscribe to balance updates:', error);
    }

    return () => {
      if (unsubscribe) {
        unsubscribe();
      }
    };
  }, [state.selectedWallet, state.isLocked]);

  const contextValue: WalletContextType = {
    state,
    dispatch,
    initializeWallet,
    createWallet,
    importWallet,
    deleteWallet,
    lockWallet,
    unlockWallet,
    refreshBalance,
    sendTransaction,
    shieldTokens,
    unshieldTokens,
    refreshTransactions,
    stakeTokens,
    unstakeTokens,
    refreshStaking,
    updateSecuritySettings,
    switchNetwork,
  };

  return (
    <WalletContext.Provider value={contextValue}>
      {children}
    </WalletContext.Provider>
  );
}

// Hook to use wallet context
export function useWalletContext(): WalletContextType {
  const context = useContext(WalletContext);
  if (context === undefined) {
    throw new Error('useWalletContext must be used within a WalletProvider');
  }
  return context;
}
