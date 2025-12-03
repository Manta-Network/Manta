/**
 * Chameleon Wallet - Wallet Hook
 */

import { useState, useEffect, useCallback } from 'react';
import { walletService } from '@/services/wallet';
import type { Wallet, Balance, UseWalletReturn } from '@/types';

/**
 * Custom hook for wallet management
 */
export function useWallet(): UseWalletReturn {
  const [wallet, setWallet] = useState<Wallet | null>(null);
  const [balance, setBalance] = useState<Balance | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Initialize wallet service and load existing wallet
  useEffect(() => {
    const initializeWallet = async () => {
      try {
        setIsLoading(true);
        await walletService.initialize();
        
        const currentWallet = walletService.getCurrentWallet();
        if (currentWallet) {
          setWallet(currentWallet);
          await refreshBalance();
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to initialize wallet');
      } finally {
        setIsLoading(false);
      }
    };

    initializeWallet();
  }, []);

  // Subscribe to balance updates when wallet is available
  useEffect(() => {
    if (!wallet) return;

    let unsubscribe: (() => void) | null = null;

    try {
      unsubscribe = walletService.subscribeToBalance((newBalance) => {
        setBalance(newBalance);
      });
    } catch (err) {
      console.error('Failed to subscribe to balance updates:', err);
    }

    return () => {
      if (unsubscribe) {
        unsubscribe();
      }
    };
  }, [wallet]);

  /**
   * Create a new wallet
   */
  const createWallet = useCallback(async (name: string, seed?: string): Promise<Wallet> => {
    try {
      setIsLoading(true);
      setError(null);
      
      const newWallet = await walletService.createWallet(name, seed);
      setWallet(newWallet);
      
      // Get initial balance
      await refreshBalance();
      
      return newWallet;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to create wallet';
      setError(errorMessage);
      throw new Error(errorMessage);
    } finally {
      setIsLoading(false);
    }
  }, []);

  /**
   * Import an existing wallet
   */
  const importWallet = useCallback(async (seed: string, name: string): Promise<Wallet> => {
    try {
      setIsLoading(true);
      setError(null);
      
      const importedWallet = await walletService.importWallet(seed, name);
      setWallet(importedWallet);
      
      // Get initial balance
      await refreshBalance();
      
      return importedWallet;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to import wallet';
      setError(errorMessage);
      throw new Error(errorMessage);
    } finally {
      setIsLoading(false);
    }
  }, []);

  /**
   * Delete the current wallet
   */
  const deleteWallet = useCallback(async (): Promise<void> => {
    try {
      setIsLoading(true);
      setError(null);
      
      await walletService.deleteWallet();
      setWallet(null);
      setBalance(null);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to delete wallet';
      setError(errorMessage);
      throw new Error(errorMessage);
    } finally {
      setIsLoading(false);
    }
  }, []);

  /**
   * Refresh wallet balance
   */
  const refreshBalance = useCallback(async (): Promise<void> => {
    if (!wallet) return;

    try {
      const newBalance = await walletService.getBalance();
      setBalance(newBalance);
    } catch (err) {
      console.error('Failed to refresh balance:', err);
      // Don't set error state for balance refresh failures
      // as this is often called in the background
    }
  }, [wallet]);

  return {
    wallet,
    balance,
    isLoading,
    error,
    createWallet,
    importWallet,
    deleteWallet,
    refreshBalance,
  };
}
