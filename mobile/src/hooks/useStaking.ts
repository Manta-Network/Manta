/**
 * Chameleon Wallet - Staking Hook
 */

import { useState, useEffect, useCallback } from 'react';
import { walletService } from '@/services/wallet';
import { rpcService } from '@/services/rpc';
import type { Validator, StakingInfo, UseStakingReturn } from '@/types';

/**
 * Custom hook for staking management
 */
export function useStaking(): UseStakingReturn {
  const [validators, setValidators] = useState<Validator[]>([]);
  const [stakingInfo, setStakingInfo] = useState<StakingInfo | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * Load validators list
   */
  const refreshValidators = useCallback(async (): Promise<void> => {
    try {
      setIsLoading(true);
      setError(null);
      
      const validatorsList = await rpcService.getValidators();
      setValidators(validatorsList);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load validators';
      setError(errorMessage);
      console.error('Failed to refresh validators:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  /**
   * Load staking information for current wallet
   */
  const refreshStaking = useCallback(async (): Promise<void> => {
    try {
      const wallet = walletService.getCurrentWallet();
      if (!wallet) return;
      
      setIsLoading(true);
      setError(null);
      
      const stakingData = await rpcService.getStakingInfo(wallet.address);
      setStakingInfo(stakingData);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load staking info';
      setError(errorMessage);
      console.error('Failed to refresh staking info:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  /**
   * Stake tokens to a validator
   */
  const stake = useCallback(async (validatorAddress: string, amount: string): Promise<string> => {
    try {
      setIsLoading(true);
      setError(null);
      
      const txHash = await walletService.stakeTokens(validatorAddress, amount);
      
      // Refresh staking info after successful stake
      setTimeout(() => {
        refreshStaking();
      }, 2000); // Wait a bit for transaction to be processed
      
      return txHash;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to stake tokens';
      setError(errorMessage);
      throw new Error(errorMessage);
    } finally {
      setIsLoading(false);
    }
  }, [refreshStaking]);

  /**
   * Unstake tokens
   */
  const unstake = useCallback(async (amount: string): Promise<string> => {
    try {
      setIsLoading(true);
      setError(null);
      
      const txHash = await walletService.unstakeTokens(amount);
      
      // Refresh staking info after successful unstake
      setTimeout(() => {
        refreshStaking();
      }, 2000); // Wait a bit for transaction to be processed
      
      return txHash;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to unstake tokens';
      setError(errorMessage);
      throw new Error(errorMessage);
    } finally {
      setIsLoading(false);
    }
  }, [refreshStaking]);

  /**
   * Claim staking rewards
   */
  const claimRewards = useCallback(async (): Promise<string> => {
    try {
      setIsLoading(true);
      setError(null);
      
      // This would be implemented when the claim rewards functionality is available
      throw new Error('Claim rewards not yet implemented');
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to claim rewards';
      setError(errorMessage);
      throw new Error(errorMessage);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Load validators and staking info on mount
  useEffect(() => {
    refreshValidators();
    refreshStaking();
  }, [refreshValidators, refreshStaking]);

  return {
    validators,
    stakingInfo,
    isLoading,
    error,
    stake,
    unstake,
    claimRewards,
    refreshStaking,
  };
}

/**
 * Hook for individual validator information
 */
export function useValidator(validatorAddress?: string) {
  const [validator, setValidator] = useState<Validator | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadValidator = useCallback(async (address: string): Promise<void> => {
    try {
      setIsLoading(true);
      setError(null);
      
      const validators = await rpcService.getValidators();
      const validatorData = validators.find(v => v.address === address);
      
      if (validatorData) {
        setValidator(validatorData);
      } else {
        setError('Validator not found');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load validator';
      setError(errorMessage);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    if (validatorAddress) {
      loadValidator(validatorAddress);
    }
  }, [validatorAddress, loadValidator]);

  return {
    validator,
    isLoading,
    error,
    loadValidator,
  };
}

/**
 * Hook for staking statistics and calculations
 */
export function useStakingStats() {
  const [stats, setStats] = useState({
    totalStaked: '0',
    totalRewards: '0',
    averageAPY: '0',
    activeValidators: 0,
  });
  const [isLoading, setIsLoading] = useState(false);

  const calculateStats = useCallback(async (): Promise<void> => {
    try {
      setIsLoading(true);
      
      const validators = await rpcService.getValidators();
      const wallet = walletService.getCurrentWallet();
      
      if (!wallet) return;
      
      const stakingInfo = await rpcService.getStakingInfo(wallet.address);
      
      // Calculate statistics
      const totalStaked = stakingInfo?.stakedAmount || '0';
      const totalRewards = stakingInfo?.rewards || '0';
      const activeValidators = validators.filter(v => v.isActive).length;
      
      // Calculate average APY
      const totalAPY = validators.reduce((sum, v) => sum + parseFloat(v.apy), 0);
      const averageAPY = validators.length > 0 ? (totalAPY / validators.length).toFixed(2) : '0';
      
      setStats({
        totalStaked,
        totalRewards,
        averageAPY,
        activeValidators,
      });
    } catch (err) {
      console.error('Failed to calculate staking stats:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    calculateStats();
  }, [calculateStats]);

  return {
    stats,
    isLoading,
    refreshStats: calculateStats,
  };
}
