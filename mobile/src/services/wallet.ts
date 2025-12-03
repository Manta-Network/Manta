/**
 * Chameleon Wallet - Wallet Management Service
 */

import { generateSeedPhrase, validateSeedPhrase, deriveKeypairFromSeed, initializeCrypto } from '@/utils/crypto';
import { SecureStorage, AppStorage } from './storage';
import { rpcService } from './rpc';
import { DEFAULT_NETWORK } from '@/constants';
import type { Wallet, Balance, NetworkConfig, StoredWallet } from '@/types';

/**
 * Wallet management service
 */
export class WalletService {
  private static instance: WalletService;
  private currentWallet: Wallet | null = null;
  private seedPhrase: string | null = null;
  private isInitialized = false;

  private constructor() {}

  static getInstance(): WalletService {
    if (!WalletService.instance) {
      WalletService.instance = new WalletService();
    }
    return WalletService.instance;
  }

  /**
   * Initialize the wallet service
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) {
      return;
    }

    try {
      // Initialize cryptographic libraries
      await initializeCrypto();

      // Load existing wallet if any
      await this.loadExistingWallet();

      this.isInitialized = true;
    } catch (error) {
      console.error('Failed to initialize wallet service:', error);
      throw error;
    }
  }

  /**
   * Create a new wallet
   */
  async createWallet(
    name: string,
    seedPhrase?: string,
    network: NetworkConfig = DEFAULT_NETWORK
  ): Promise<Wallet> {
    try {
      // Generate or validate seed phrase
      const seed = seedPhrase || generateSeedPhrase(12);
      
      if (!validateSeedPhrase(seed)) {
        throw new Error('Invalid seed phrase');
      }

      // Derive keypair from seed
      const { address, publicKey } = deriveKeypairFromSeed(seed, undefined, network);

      // Create wallet object
      const wallet: Wallet = {
        address,
        publicKey,
        name,
        createdAt: Date.now(),
        isHD: true,
        derivationPath: '//chameleon//wallet//0',
      };

      // Store seed phrase securely
      await SecureStorage.storeSeedPhrase(seed);

      // Store wallet data
      const storedWallet: StoredWallet = {
        address: wallet.address,
        name: wallet.name,
        createdAt: wallet.createdAt,
        isHD: wallet.isHD,
        derivationPath: wallet.derivationPath,
      };
      await AppStorage.storeWallet(storedWallet);

      // Set as current wallet
      this.currentWallet = wallet;
      this.seedPhrase = seed;

      return wallet;
    } catch (error) {
      console.error('Failed to create wallet:', error);
      throw new Error(`Failed to create wallet: ${error}`);
    }
  }

  /**
   * Import an existing wallet from seed phrase
   */
  async importWallet(
    seedPhrase: string,
    name: string,
    network: NetworkConfig = DEFAULT_NETWORK
  ): Promise<Wallet> {
    try {
      if (!validateSeedPhrase(seedPhrase)) {
        throw new Error('Invalid seed phrase');
      }

      return await this.createWallet(name, seedPhrase, network);
    } catch (error) {
      console.error('Failed to import wallet:', error);
      throw new Error(`Failed to import wallet: ${error}`);
    }
  }

  /**
   * Load existing wallet from storage
   */
  private async loadExistingWallet(): Promise<void> {
    try {
      const wallets = await AppStorage.getWallets();
      
      if (wallets.length === 0) {
        return;
      }

      // Load the first wallet (in a full implementation, you'd handle multiple wallets)
      const storedWallet = wallets[0];
      const seedPhrase = await SecureStorage.getSeedPhrase();

      if (!seedPhrase) {
        console.warn('Wallet data found but no seed phrase in secure storage');
        return;
      }

      // Reconstruct wallet object
      this.currentWallet = {
        address: storedWallet.address,
        publicKey: '', // Would need to derive this
        name: storedWallet.name,
        createdAt: storedWallet.createdAt,
        isHD: storedWallet.isHD,
        derivationPath: storedWallet.derivationPath,
      };
      
      this.seedPhrase = seedPhrase;
    } catch (error) {
      console.error('Failed to load existing wallet:', error);
    }
  }

  /**
   * Get current wallet
   */
  getCurrentWallet(): Wallet | null {
    return this.currentWallet;
  }

  /**
   * Get wallet balance
   */
  async getBalance(address?: string): Promise<Balance> {
    const walletAddress = address || this.currentWallet?.address;
    
    if (!walletAddress) {
      throw new Error('No wallet address available');
    }

    try {
      return await rpcService.getBalance(walletAddress);
    } catch (error) {
      console.error('Failed to get balance:', error);
      throw error;
    }
  }

  /**
   * Send a transaction
   */
  async sendTransaction(
    toAddress: string,
    amount: string,
    isPrivate: boolean = false,
    memo?: string
  ): Promise<string> {
    if (!this.currentWallet || !this.seedPhrase) {
      throw new Error('No wallet available');
    }

    try {
      const txHash = await rpcService.sendTransaction(
        this.seedPhrase,
        toAddress,
        amount,
        isPrivate,
        memo
      );

      // Store transaction in local storage
      await AppStorage.addTransaction({
        hash: txHash,
        type: 'send',
        amount,
        timestamp: Date.now(),
        status: 'pending',
        isPrivate,
        from: this.currentWallet.address,
        to: toAddress,
        memo,
      });

      return txHash;
    } catch (error) {
      console.error('Failed to send transaction:', error);
      throw error;
    }
  }

  /**
   * Shield tokens (public to private)
   */
  async shieldTokens(amount: string): Promise<string> {
    if (!this.currentWallet || !this.seedPhrase) {
      throw new Error('No wallet available');
    }

    try {
      const txHash = await rpcService.shieldTokens(this.seedPhrase, amount);

      // Store transaction in local storage
      await AppStorage.addTransaction({
        hash: txHash,
        type: 'shield',
        amount,
        timestamp: Date.now(),
        status: 'pending',
        isPrivate: true,
        from: this.currentWallet.address,
        to: this.currentWallet.address,
      });

      return txHash;
    } catch (error) {
      console.error('Failed to shield tokens:', error);
      throw error;
    }
  }

  /**
   * Unshield tokens (private to public)
   */
  async unshieldTokens(amount: string): Promise<string> {
    if (!this.currentWallet || !this.seedPhrase) {
      throw new Error('No wallet available');
    }

    try {
      const txHash = await rpcService.unshieldTokens(this.seedPhrase, amount);

      // Store transaction in local storage
      await AppStorage.addTransaction({
        hash: txHash,
        type: 'unshield',
        amount,
        timestamp: Date.now(),
        status: 'pending',
        isPrivate: false,
        from: this.currentWallet.address,
        to: this.currentWallet.address,
      });

      return txHash;
    } catch (error) {
      console.error('Failed to unshield tokens:', error);
      throw error;
    }
  }

  /**
   * Stake tokens to a validator
   */
  async stakeTokens(validatorAddress: string, amount: string): Promise<string> {
    if (!this.currentWallet || !this.seedPhrase) {
      throw new Error('No wallet available');
    }

    try {
      const txHash = await rpcService.stakeTokens(
        this.seedPhrase,
        validatorAddress,
        amount
      );

      // Store transaction in local storage
      await AppStorage.addTransaction({
        hash: txHash,
        type: 'stake',
        amount,
        timestamp: Date.now(),
        status: 'pending',
        isPrivate: false,
        from: this.currentWallet.address,
        to: validatorAddress,
      });

      return txHash;
    } catch (error) {
      console.error('Failed to stake tokens:', error);
      throw error;
    }
  }

  /**
   * Unstake tokens
   */
  async unstakeTokens(amount: string): Promise<string> {
    if (!this.currentWallet || !this.seedPhrase) {
      throw new Error('No wallet available');
    }

    try {
      const txHash = await rpcService.unstakeTokens(this.seedPhrase, amount);

      // Store transaction in local storage
      await AppStorage.addTransaction({
        hash: txHash,
        type: 'unstake',
        amount,
        timestamp: Date.now(),
        status: 'pending',
        isPrivate: false,
        from: this.currentWallet.address,
      });

      return txHash;
    } catch (error) {
      console.error('Failed to unstake tokens:', error);
      throw error;
    }
  }

  /**
   * Get transaction history
   */
  async getTransactionHistory(): Promise<any[]> {
    if (!this.currentWallet) {
      return [];
    }

    try {
      // Get transactions from blockchain
      const blockchainTxs = await rpcService.getTransactionHistory(this.currentWallet.address);
      
      // Get cached transactions from storage
      const cachedTxs = await AppStorage.getTransactions();
      
      // Merge and deduplicate transactions
      const allTxs = [...blockchainTxs, ...cachedTxs];
      const uniqueTxs = allTxs.filter((tx, index, self) => 
        index === self.findIndex(t => t.hash === tx.hash)
      );
      
      // Sort by timestamp (newest first)
      return uniqueTxs.sort((a, b) => b.timestamp - a.timestamp);
    } catch (error) {
      console.error('Failed to get transaction history:', error);
      // Return cached transactions as fallback
      return await AppStorage.getTransactions();
    }
  }

  /**
   * Get seed phrase (requires authentication)
   */
  async getSeedPhrase(): Promise<string | null> {
    try {
      return await SecureStorage.getSeedPhrase();
    } catch (error) {
      console.error('Failed to get seed phrase:', error);
      return null;
    }
  }

  /**
   * Delete wallet and all associated data
   */
  async deleteWallet(): Promise<void> {
    try {
      if (this.currentWallet) {
        await AppStorage.deleteWallet(this.currentWallet.address);
      }
      
      await SecureStorage.clearAll();
      
      this.currentWallet = null;
      this.seedPhrase = null;
    } catch (error) {
      console.error('Failed to delete wallet:', error);
      throw error;
    }
  }

  /**
   * Check if wallet exists
   */
  async hasWallet(): Promise<boolean> {
    try {
      const wallets = await AppStorage.getWallets();
      return wallets.length > 0;
    } catch (error) {
      console.error('Failed to check wallet existence:', error);
      return false;
    }
  }

  /**
   * Validate address format
   */
  validateAddress(address: string): boolean {
    try {
      // Use the crypto utility to validate address
      const { validateAddress } = require('@/utils/crypto');
      return validateAddress(address);
    } catch (error) {
      return false;
    }
  }

  /**
   * Get wallet statistics
   */
  async getWalletStats(): Promise<{
    totalTransactions: number;
    totalSent: string;
    totalReceived: string;
    firstTransactionDate?: number;
  }> {
    try {
      const transactions = await this.getTransactionHistory();
      
      let totalSent = BigInt(0);
      let totalReceived = BigInt(0);
      let firstTransactionDate: number | undefined;
      
      transactions.forEach(tx => {
        if (tx.type === 'send') {
          totalSent += BigInt(tx.amount);
        } else if (tx.type === 'receive') {
          totalReceived += BigInt(tx.amount);
        }
        
        if (!firstTransactionDate || tx.timestamp < firstTransactionDate) {
          firstTransactionDate = tx.timestamp;
        }
      });
      
      return {
        totalTransactions: transactions.length,
        totalSent: totalSent.toString(),
        totalReceived: totalReceived.toString(),
        firstTransactionDate,
      };
    } catch (error) {
      console.error('Failed to get wallet stats:', error);
      return {
        totalTransactions: 0,
        totalSent: '0',
        totalReceived: '0',
      };
    }
  }

  /**
   * Subscribe to balance updates
   */
  subscribeToBalance(callback: (balance: Balance) => void): () => void {
    if (!this.currentWallet) {
      throw new Error('No wallet available');
    }

    return rpcService.subscribeToBalance(this.currentWallet.address, callback);
  }

  /**
   * Check if service is initialized
   */
  isServiceInitialized(): boolean {
    return this.isInitialized;
  }
}

// Export singleton instance
export const walletService = WalletService.getInstance();
