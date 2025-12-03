/**
 * Chameleon Wallet - Transactions Hook
 */

import { useState, useEffect, useCallback } from 'react';
import { walletService } from '@/services/wallet';
import type { Transaction, UseTransactionsReturn } from '@/types';

/**
 * Custom hook for transaction management
 */
export function useTransactions(): UseTransactionsReturn {
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * Load transaction history
   */
  const refreshTransactions = useCallback(async (): Promise<void> => {
    try {
      setIsLoading(true);
      setError(null);
      
      const txHistory = await walletService.getTransactionHistory();
      setTransactions(txHistory);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load transactions';
      setError(errorMessage);
      console.error('Failed to refresh transactions:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  /**
   * Send a transaction
   */
  const sendTransaction = useCallback(async (
    to: string,
    amount: string,
    isPrivate: boolean,
    memo?: string
  ): Promise<string> => {
    try {
      setIsLoading(true);
      setError(null);
      
      const txHash = await walletService.sendTransaction(to, amount, isPrivate, memo);
      
      // Refresh transactions to include the new one
      await refreshTransactions();
      
      return txHash;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to send transaction';
      setError(errorMessage);
      throw new Error(errorMessage);
    } finally {
      setIsLoading(false);
    }
  }, [refreshTransactions]);

  /**
   * Shield tokens (public to private)
   */
  const shieldTokens = useCallback(async (amount: string): Promise<string> => {
    try {
      setIsLoading(true);
      setError(null);
      
      const txHash = await walletService.shieldTokens(amount);
      
      // Refresh transactions to include the new one
      await refreshTransactions();
      
      return txHash;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to shield tokens';
      setError(errorMessage);
      throw new Error(errorMessage);
    } finally {
      setIsLoading(false);
    }
  }, [refreshTransactions]);

  /**
   * Unshield tokens (private to public)
   */
  const unshieldTokens = useCallback(async (amount: string): Promise<string> => {
    try {
      setIsLoading(true);
      setError(null);
      
      const txHash = await walletService.unshieldTokens(amount);
      
      // Refresh transactions to include the new one
      await refreshTransactions();
      
      return txHash;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to unshield tokens';
      setError(errorMessage);
      throw new Error(errorMessage);
    } finally {
      setIsLoading(false);
    }
  }, [refreshTransactions]);

  // Load transactions on mount
  useEffect(() => {
    refreshTransactions();
  }, [refreshTransactions]);

  return {
    transactions,
    isLoading,
    error,
    sendTransaction,
    shieldTokens,
    unshieldTokens,
    refreshTransactions,
  };
}

/**
 * Hook for individual transaction operations
 */
export function useTransaction(txHash?: string) {
  const [transaction, setTransaction] = useState<Transaction | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadTransaction = useCallback(async (hash: string): Promise<void> => {
    try {
      setIsLoading(true);
      setError(null);
      
      const transactions = await walletService.getTransactionHistory();
      const tx = transactions.find(t => t.hash === hash);
      
      if (tx) {
        setTransaction(tx);
      } else {
        setError('Transaction not found');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load transaction';
      setError(errorMessage);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    if (txHash) {
      loadTransaction(txHash);
    }
  }, [txHash, loadTransaction]);

  return {
    transaction,
    isLoading,
    error,
    loadTransaction,
  };
}
