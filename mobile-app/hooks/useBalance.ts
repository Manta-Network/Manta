/**
 * React hook for balance subscription and formatting
 */

import { useState, useEffect, useCallback } from 'react';
import { chainService, type BalanceInfo } from '../services/chain';
import { apiService } from '../services/api';
import { NETWORK_CONFIG } from '../config/network';

export interface UseBalanceReturn {
  balance: BalanceInfo | null;
  formattedBalance: string;
  formattedFreeBalance: string;
  formattedReservedBalance: string;
  isLoading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
}

/**
 * Hook for subscribing to balance changes and formatting
 */
export function useBalance(address?: string): UseBalanceReturn {
  const [balance, setBalance] = useState<BalanceInfo | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Format balance helper
  const formatBalance = useCallback((amount: string): string => {
    if (!amount || amount === '0') {
      return `0 ${NETWORK_CONFIG.tokenSymbol}`;
    }
    return chainService.formatBalance(amount);
  }, []);

  // Refresh balance manually
  const refresh = useCallback(async () => {
    if (!address || !apiService.isConnected()) {
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const balanceInfo = await chainService.getBalance(address);
      setBalance(balanceInfo);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch balance';
      setError(errorMessage);
      console.error('Error fetching balance:', err);
    } finally {
      setIsLoading(false);
    }
  }, [address]);

  // Subscribe to balance changes
  useEffect(() => {
    if (!address || !apiService.isConnected()) {
      setBalance(null);
      setError(null);
      return;
    }

    setIsLoading(true);
    setError(null);

    let unsubscribe: (() => void) | null = null;

    try {
      unsubscribe = chainService.subscribeToBalance(address, (balanceInfo) => {
        setBalance(balanceInfo);
        setIsLoading(false);
        setError(null);
      });
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to subscribe to balance';
      setError(errorMessage);
      setIsLoading(false);
      console.error('Error subscribing to balance:', err);
    }

    return () => {
      if (unsubscribe) {
        unsubscribe();
      }
    };
  }, [address]);

  // Handle API connection changes
  useEffect(() => {
    const unsubscribe = apiService.onConnectionStateChange((state) => {
      if (state.status === 'connected' && address && !balance) {
        // Refresh balance when API reconnects
        refresh();
      } else if (state.status === 'disconnected') {
        setError('Network disconnected');
      }
    });

    return unsubscribe;
  }, [address, balance, refresh]);

  // Formatted balance strings
  const formattedBalance = balance ? formatBalance(balance.total) : `0 ${NETWORK_CONFIG.tokenSymbol}`;
  const formattedFreeBalance = balance ? formatBalance(balance.free) : `0 ${NETWORK_CONFIG.tokenSymbol}`;
  const formattedReservedBalance = balance ? formatBalance(balance.reserved) : `0 ${NETWORK_CONFIG.tokenSymbol}`;

  return {
    balance,
    formattedBalance,
    formattedFreeBalance,
    formattedReservedBalance,
    isLoading,
    error,
    refresh,
  };
}

/**
 * Hook for getting formatted balance without subscription (one-time fetch)
 */
export function useBalanceOnce(address?: string) {
  const [balance, setBalance] = useState<BalanceInfo | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchBalance = useCallback(async () => {
    if (!address || !apiService.isConnected()) {
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const balanceInfo = await chainService.getBalance(address);
      setBalance(balanceInfo);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch balance';
      setError(errorMessage);
      console.error('Error fetching balance:', err);
    } finally {
      setIsLoading(false);
    }
  }, [address]);

  useEffect(() => {
    fetchBalance();
  }, [fetchBalance]);

  const formatBalance = useCallback((amount: string): string => {
    if (!amount || amount === '0') {
      return `0 ${NETWORK_CONFIG.tokenSymbol}`;
    }
    return chainService.formatBalance(amount);
  }, []);

  return {
    balance,
    formattedBalance: balance ? formatBalance(balance.total) : `0 ${NETWORK_CONFIG.tokenSymbol}`,
    formattedFreeBalance: balance ? formatBalance(balance.free) : `0 ${NETWORK_CONFIG.tokenSymbol}`,
    isLoading,
    error,
    refetch: fetchBalance,
  };
}
