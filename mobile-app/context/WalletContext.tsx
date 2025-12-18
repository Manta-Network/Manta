/**
 * React Context for wallet state management
 */

import React, { createContext, useContext, useEffect, useState, useCallback, ReactNode } from 'react';
import { walletService, type WalletState } from '../services/wallet';
import { storageService } from '../services/storage';

interface WalletContextType {
  wallet: WalletState | null;
  isLoading: boolean;
  error: string | null;
  createWallet: (mnemonic: string, name?: string) => Promise<void>;
  importWallet: (mnemonic: string, name?: string) => Promise<void>;
  importDevAccount: (accountName: 'alice' | 'bob' | 'charlie' | 'dave' | 'eve') => Promise<void>;
  logout: () => Promise<void>;
  clearError: () => void;
}

const WalletContext = createContext<WalletContextType | undefined>(undefined);

interface WalletProviderProps {
  children: ReactNode;
}

export function WalletProvider({ children }: WalletProviderProps) {
  const [wallet, setWallet] = useState<WalletState | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Initialize wallet from storage on app start
  useEffect(() => {
    initializeWallet();
  }, []);

  // Subscribe to wallet state changes
  useEffect(() => {
    const unsubscribe = walletService.onWalletStateChange((state) => {
      setWallet(state);
    });

    return unsubscribe;
  }, []);

  const initializeWallet = async () => {
    setIsLoading(true);
    setError(null);

    try {
      // Check if wallet exists in storage
      const hasWallet = await storageService.hasWallet();
      
      if (hasWallet) {
        // Get wallet metadata
        const metadata = await storageService.getWalletMetadata();
        
        if (metadata) {
          // Set wallet state as locked (user needs to unlock with seed)
          setWallet({
            address: metadata.address,
            name: metadata.name,
            isLocked: true,
          });
        }
      }
    } catch (err) {
      console.error('Error initializing wallet:', err);
      setError('Failed to initialize wallet');
    } finally {
      setIsLoading(false);
    }
  };

  const createWallet = useCallback(async (mnemonic: string, name: string = 'My Wallet') => {
    setIsLoading(true);
    setError(null);

    try {
      // Create wallet with wallet service
      const walletState = await walletService.createWallet(mnemonic, name);
      
      // Save to secure storage
      await storageService.saveEncryptedSeed(mnemonic);
      await storageService.saveWalletMetadata(name, walletState.address);
      
      setWallet(walletState);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to create wallet';
      setError(errorMessage);
      console.error('Error creating wallet:', err);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const importWallet = useCallback(async (mnemonic: string, name: string = 'Imported Wallet') => {
    setIsLoading(true);
    setError(null);

    try {
      // Import wallet with wallet service
      const walletState = await walletService.importWallet(mnemonic, name);
      
      // Save to secure storage
      await storageService.saveEncryptedSeed(mnemonic);
      await storageService.saveWalletMetadata(name, walletState.address);
      
      setWallet(walletState);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to import wallet';
      setError(errorMessage);
      console.error('Error importing wallet:', err);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const importDevAccount = useCallback(async (accountName: 'alice' | 'bob' | 'charlie' | 'dave' | 'eve') => {
    setIsLoading(true);
    setError(null);

    try {
      // Import dev account
      const walletState = await walletService.importDevAccount(accountName);
      
      // Get the mnemonic for the dev account
      const mnemonic = DEV_ACCOUNTS[accountName];
      
      // Save to secure storage
      await storageService.saveEncryptedSeed(mnemonic);
      await storageService.saveWalletMetadata(walletState.name, walletState.address);
      
      setWallet(walletState);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to import dev account';
      setError(errorMessage);
      console.error('Error importing dev account:', err);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const logout = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      // Clear wallet service state
      walletService.clearWallet();
      
      // Clear secure storage
      await storageService.deleteWallet();
      
      setWallet(null);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to logout';
      setError(errorMessage);
      console.error('Error during logout:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  const value: WalletContextType = {
    wallet,
    isLoading,
    error,
    createWallet,
    importWallet,
    importDevAccount,
    logout,
    clearError,
  };

  return (
    <WalletContext.Provider value={value}>
      {children}
    </WalletContext.Provider>
  );
}

export function useWallet(): WalletContextType {
  const context = useContext(WalletContext);
  if (context === undefined) {
    throw new Error('useWallet must be used within a WalletProvider');
  }
  return context;
}
