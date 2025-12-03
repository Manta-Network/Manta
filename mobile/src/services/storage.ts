/**
 * Chameleon Wallet - Secure Storage Service
 */

import EncryptedStorage from 'react-native-encrypted-storage';
import * as Keychain from 'react-native-keychain';
import { STORAGE_KEYS, KEYCHAIN_KEYS } from '@/constants';
import type {
  StoredWallet,
  StoredTransaction,
  SecuritySettings,
  NetworkConfig,
} from '@/types';

/**
 * Secure storage for sensitive data (seed phrases, private keys)
 */
export class SecureStorage {
  /**
   * Store seed phrase securely in keychain
   */
  static async storeSeedPhrase(seedPhrase: string): Promise<void> {
    try {
      await Keychain.setGenericPassword(
        KEYCHAIN_KEYS.SEED_PHRASE,
        seedPhrase,
        {
          service: 'chameleon_wallet',
          accessible: Keychain.ACCESSIBLE.WHEN_UNLOCKED_THIS_DEVICE_ONLY,
          authenticationType: Keychain.AUTHENTICATION_TYPE.BIOMETRICS,
          accessGroup: 'group.com.chameleonnetwork.wallet',
        }
      );
    } catch (error) {
      throw new Error(`Failed to store seed phrase: ${error}`);
    }
  }

  /**
   * Retrieve seed phrase from keychain
   */
  static async getSeedPhrase(): Promise<string | null> {
    try {
      const credentials = await Keychain.getGenericPassword({
        service: 'chameleon_wallet',
        authenticationType: Keychain.AUTHENTICATION_TYPE.BIOMETRICS,
      });
      
      if (credentials && credentials.username === KEYCHAIN_KEYS.SEED_PHRASE) {
        return credentials.password;
      }
      
      return null;
    } catch (error) {
      console.error('Failed to retrieve seed phrase:', error);
      return null;
    }
  }

  /**
   * Store PIN hash securely
   */
  static async storePinHash(pinHash: string, salt: string): Promise<void> {
    try {
      const data = JSON.stringify({ hash: pinHash, salt });
      await Keychain.setGenericPassword(
        KEYCHAIN_KEYS.PIN_HASH,
        data,
        {
          service: 'chameleon_wallet_pin',
          accessible: Keychain.ACCESSIBLE.WHEN_UNLOCKED_THIS_DEVICE_ONLY,
        }
      );
    } catch (error) {
      throw new Error(`Failed to store PIN hash: ${error}`);
    }
  }

  /**
   * Retrieve PIN hash
   */
  static async getPinHash(): Promise<{ hash: string; salt: string } | null> {
    try {
      const credentials = await Keychain.getGenericPassword({
        service: 'chameleon_wallet_pin',
      });
      
      if (credentials && credentials.username === KEYCHAIN_KEYS.PIN_HASH) {
        return JSON.parse(credentials.password);
      }
      
      return null;
    } catch (error) {
      console.error('Failed to retrieve PIN hash:', error);
      return null;
    }
  }

  /**
   * Delete all secure data
   */
  static async clearAll(): Promise<void> {
    try {
      await Promise.all([
        Keychain.resetGenericPassword({ service: 'chameleon_wallet' }),
        Keychain.resetGenericPassword({ service: 'chameleon_wallet_pin' }),
      ]);
    } catch (error) {
      console.error('Failed to clear secure storage:', error);
    }
  }

  /**
   * Check if biometric authentication is available
   */
  static async isBiometricAvailable(): Promise<{
    available: boolean;
    type: string | null;
  }> {
    try {
      const biometryType = await Keychain.getSupportedBiometryType();
      return {
        available: biometryType !== null,
        type: biometryType,
      };
    } catch (error) {
      return { available: false, type: null };
    }
  }
}

/**
 * Regular encrypted storage for app data
 */
export class AppStorage {
  /**
   * Store wallet data
   */
  static async storeWallet(wallet: StoredWallet): Promise<void> {
    try {
      const existingWallets = await this.getWallets();
      const updatedWallets = existingWallets.filter(w => w.address !== wallet.address);
      updatedWallets.push(wallet);
      
      await EncryptedStorage.setItem(
        STORAGE_KEYS.WALLET_DATA,
        JSON.stringify(updatedWallets)
      );
    } catch (error) {
      throw new Error(`Failed to store wallet: ${error}`);
    }
  }

  /**
   * Get all stored wallets
   */
  static async getWallets(): Promise<StoredWallet[]> {
    try {
      const data = await EncryptedStorage.getItem(STORAGE_KEYS.WALLET_DATA);
      return data ? JSON.parse(data) : [];
    } catch (error) {
      console.error('Failed to get wallets:', error);
      return [];
    }
  }

  /**
   * Delete a wallet
   */
  static async deleteWallet(address: string): Promise<void> {
    try {
      const wallets = await this.getWallets();
      const filteredWallets = wallets.filter(w => w.address !== address);
      
      await EncryptedStorage.setItem(
        STORAGE_KEYS.WALLET_DATA,
        JSON.stringify(filteredWallets)
      );
    } catch (error) {
      throw new Error(`Failed to delete wallet: ${error}`);
    }
  }

  /**
   * Store transactions
   */
  static async storeTransactions(transactions: StoredTransaction[]): Promise<void> {
    try {
      await EncryptedStorage.setItem(
        STORAGE_KEYS.TRANSACTIONS,
        JSON.stringify(transactions)
      );
    } catch (error) {
      throw new Error(`Failed to store transactions: ${error}`);
    }
  }

  /**
   * Get stored transactions
   */
  static async getTransactions(): Promise<StoredTransaction[]> {
    try {
      const data = await EncryptedStorage.getItem(STORAGE_KEYS.TRANSACTIONS);
      return data ? JSON.parse(data) : [];
    } catch (error) {
      console.error('Failed to get transactions:', error);
      return [];
    }
  }

  /**
   * Add a new transaction
   */
  static async addTransaction(transaction: StoredTransaction): Promise<void> {
    try {
      const transactions = await this.getTransactions();
      
      // Remove existing transaction with same hash (update)
      const filteredTransactions = transactions.filter(tx => tx.hash !== transaction.hash);
      
      // Add new transaction at the beginning
      filteredTransactions.unshift(transaction);
      
      // Keep only the latest 100 transactions
      const limitedTransactions = filteredTransactions.slice(0, 100);
      
      await this.storeTransactions(limitedTransactions);
    } catch (error) {
      throw new Error(`Failed to add transaction: ${error}`);
    }
  }

  /**
   * Store security settings
   */
  static async storeSecuritySettings(settings: SecuritySettings): Promise<void> {
    try {
      await EncryptedStorage.setItem(
        STORAGE_KEYS.SECURITY,
        JSON.stringify(settings)
      );
    } catch (error) {
      throw new Error(`Failed to store security settings: ${error}`);
    }
  }

  /**
   * Get security settings
   */
  static async getSecuritySettings(): Promise<SecuritySettings | null> {
    try {
      const data = await EncryptedStorage.getItem(STORAGE_KEYS.SECURITY);
      return data ? JSON.parse(data) : null;
    } catch (error) {
      console.error('Failed to get security settings:', error);
      return null;
    }
  }

  /**
   * Store network configuration
   */
  static async storeNetworkConfig(network: NetworkConfig): Promise<void> {
    try {
      await EncryptedStorage.setItem(
        STORAGE_KEYS.NETWORK,
        JSON.stringify(network)
      );
    } catch (error) {
      throw new Error(`Failed to store network config: ${error}`);
    }
  }

  /**
   * Get network configuration
   */
  static async getNetworkConfig(): Promise<NetworkConfig | null> {
    try {
      const data = await EncryptedStorage.getItem(STORAGE_KEYS.NETWORK);
      return data ? JSON.parse(data) : null;
    } catch (error) {
      console.error('Failed to get network config:', error);
      return null;
    }
  }

  /**
   * Store price cache
   */
  static async storePriceCache(priceData: { price: number; timestamp: number }): Promise<void> {
    try {
      await EncryptedStorage.setItem(
        STORAGE_KEYS.PRICE_CACHE,
        JSON.stringify(priceData)
      );
    } catch (error) {
      console.error('Failed to store price cache:', error);
    }
  }

  /**
   * Get price cache
   */
  static async getPriceCache(): Promise<{ price: number; timestamp: number } | null> {
    try {
      const data = await EncryptedStorage.getItem(STORAGE_KEYS.PRICE_CACHE);
      return data ? JSON.parse(data) : null;
    } catch (error) {
      console.error('Failed to get price cache:', error);
      return null;
    }
  }

  /**
   * Clear all app storage
   */
  static async clearAll(): Promise<void> {
    try {
      await Promise.all([
        EncryptedStorage.removeItem(STORAGE_KEYS.WALLET_DATA),
        EncryptedStorage.removeItem(STORAGE_KEYS.TRANSACTIONS),
        EncryptedStorage.removeItem(STORAGE_KEYS.SETTINGS),
        EncryptedStorage.removeItem(STORAGE_KEYS.SECURITY),
        EncryptedStorage.removeItem(STORAGE_KEYS.NETWORK),
        EncryptedStorage.removeItem(STORAGE_KEYS.PRICE_CACHE),
      ]);
    } catch (error) {
      console.error('Failed to clear app storage:', error);
    }
  }

  /**
   * Get storage usage statistics
   */
  static async getStorageStats(): Promise<{
    wallets: number;
    transactions: number;
    hasSecuritySettings: boolean;
    hasNetworkConfig: boolean;
  }> {
    try {
      const [wallets, transactions, security, network] = await Promise.all([
        this.getWallets(),
        this.getTransactions(),
        this.getSecuritySettings(),
        this.getNetworkConfig(),
      ]);

      return {
        wallets: wallets.length,
        transactions: transactions.length,
        hasSecuritySettings: security !== null,
        hasNetworkConfig: network !== null,
      };
    } catch (error) {
      console.error('Failed to get storage stats:', error);
      return {
        wallets: 0,
        transactions: 0,
        hasSecuritySettings: false,
        hasNetworkConfig: false,
      };
    }
  }
}

/**
 * Migration utilities for storage schema changes
 */
export class StorageMigration {
  private static readonly MIGRATION_VERSION_KEY = 'chameleon_migration_version';
  private static readonly CURRENT_VERSION = 1;

  /**
   * Run storage migrations if needed
   */
  static async runMigrations(): Promise<void> {
    try {
      const currentVersion = await this.getCurrentVersion();
      
      if (currentVersion < this.CURRENT_VERSION) {
        console.log(`Running storage migrations from v${currentVersion} to v${this.CURRENT_VERSION}`);
        
        // Run migrations sequentially
        for (let version = currentVersion + 1; version <= this.CURRENT_VERSION; version++) {
          await this.runMigration(version);
        }
        
        await this.setCurrentVersion(this.CURRENT_VERSION);
        console.log('Storage migrations completed successfully');
      }
    } catch (error) {
      console.error('Storage migration failed:', error);
      throw error;
    }
  }

  private static async getCurrentVersion(): Promise<number> {
    try {
      const version = await EncryptedStorage.getItem(this.MIGRATION_VERSION_KEY);
      return version ? parseInt(version, 10) : 0;
    } catch {
      return 0;
    }
  }

  private static async setCurrentVersion(version: number): Promise<void> {
    await EncryptedStorage.setItem(this.MIGRATION_VERSION_KEY, version.toString());
  }

  private static async runMigration(version: number): Promise<void> {
    switch (version) {
      case 1:
        // Initial migration - no changes needed
        break;
      
      // Future migrations will be added here
      default:
        console.warn(`Unknown migration version: ${version}`);
    }
  }
}
