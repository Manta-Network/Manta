/**
 * Wallet/Keyring management for Chameleon Network
 */

import { Keyring } from '@polkadot/keyring';
import { mnemonicGenerate, mnemonicValidate, cryptoWaitReady } from '@polkadot/util-crypto';
import { u8aToHex } from '@polkadot/util';
import type { KeyringPair } from '@polkadot/keyring/types';

export interface WalletState {
  address: string;
  name: string;
  isLocked: boolean;
}

// Standard Substrate dev account mnemonics
export const DEV_ACCOUNTS = {
  alice: 'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice',
  bob: 'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Bob',
  charlie: 'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Charlie',
  dave: 'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Dave',
  eve: 'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Eve',
};

class WalletService {
  private static instance: WalletService;
  private keyring: Keyring | null = null;
  private currentPair: KeyringPair | null = null;
  private walletState: WalletState | null = null;
  private listeners: ((state: WalletState | null) => void)[] = [];

  private constructor() {}

  public static getInstance(): WalletService {
    if (!WalletService.instance) {
      WalletService.instance = new WalletService();
    }
    return WalletService.instance;
  }

  /**
   * Initialize the keyring
   */
  private async initializeKeyring(): Promise<void> {
    if (this.keyring) return;

    await cryptoWaitReady();
    this.keyring = new Keyring({ type: 'sr25519', ss58Format: 42 });
  }

  /**
   * Generate a new 12-word mnemonic seed phrase
   */
  public generateMnemonic(): string {
    return mnemonicGenerate(12);
  }

  /**
   * Validate a mnemonic seed phrase
   */
  public validateMnemonic(mnemonic: string): boolean {
    return mnemonicValidate(mnemonic);
  }

  /**
   * Create a new wallet from mnemonic
   */
  public async createWallet(mnemonic: string, name: string = 'My Wallet'): Promise<WalletState> {
    await this.initializeKeyring();
    
    if (!this.validateMnemonic(mnemonic)) {
      throw new Error('Invalid mnemonic phrase');
    }

    if (!this.keyring) {
      throw new Error('Keyring not initialized');
    }

    try {
      this.currentPair = this.keyring.addFromMnemonic(mnemonic);
      
      this.walletState = {
        address: this.currentPair.address,
        name,
        isLocked: false,
      };

      this.notifyListeners();
      return this.walletState;
    } catch (error) {
      console.error('Error creating wallet:', error);
      throw new Error('Failed to create wallet from mnemonic');
    }
  }

  /**
   * Import an existing wallet from mnemonic
   */
  public async importWallet(mnemonic: string, name: string = 'Imported Wallet'): Promise<WalletState> {
    return this.createWallet(mnemonic, name);
  }

  /**
   * Import a dev account by name
   */
  public async importDevAccount(accountName: keyof typeof DEV_ACCOUNTS): Promise<WalletState> {
    const mnemonic = DEV_ACCOUNTS[accountName];
    const name = `Dev Account (${accountName.charAt(0).toUpperCase() + accountName.slice(1)})`;
    return this.createWallet(mnemonic, name);
  }

  /**
   * Get the current wallet address
   */
  public getAddress(): string | null {
    return this.walletState?.address || null;
  }

  /**
   * Get the current wallet state
   */
  public getWalletState(): WalletState | null {
    return this.walletState ? { ...this.walletState } : null;
  }

  /**
   * Check if wallet exists and is unlocked
   */
  public isWalletReady(): boolean {
    return this.walletState !== null && !this.walletState.isLocked && this.currentPair !== null;
  }

  /**
   * Lock the wallet
   */
  public lockWallet(): void {
    if (this.walletState) {
      this.walletState.isLocked = true;
      this.notifyListeners();
    }
  }

  /**
   * Unlock the wallet with mnemonic
   */
  public async unlockWallet(mnemonic: string): Promise<void> {
    if (!this.walletState) {
      throw new Error('No wallet to unlock');
    }

    await this.initializeKeyring();
    
    if (!this.validateMnemonic(mnemonic)) {
      throw new Error('Invalid mnemonic phrase');
    }

    if (!this.keyring) {
      throw new Error('Keyring not initialized');
    }

    try {
      const pair = this.keyring.addFromMnemonic(mnemonic);
      
      // Verify the address matches
      if (pair.address !== this.walletState.address) {
        throw new Error('Mnemonic does not match wallet address');
      }

      this.currentPair = pair;
      this.walletState.isLocked = false;
      this.notifyListeners();
    } catch (error) {
      console.error('Error unlocking wallet:', error);
      throw new Error('Failed to unlock wallet');
    }
  }

  /**
   * Sign a transaction (placeholder for future implementation)
   */
  public async signTransaction(tx: any): Promise<string> {
    if (!this.isWalletReady()) {
      throw new Error('Wallet not ready for signing');
    }

    if (!this.currentPair) {
      throw new Error('No keypair available for signing');
    }

    // This will be implemented in Phase 3 when we add send/receive functionality
    throw new Error('Transaction signing not yet implemented');
  }

  /**
   * Get the keypair for signing (internal use)
   */
  public getKeyPair(): KeyringPair | null {
    return this.currentPair;
  }

  /**
   * Clear wallet data (logout)
   */
  public clearWallet(): void {
    this.currentPair = null;
    this.walletState = null;
    this.notifyListeners();
  }

  /**
   * Subscribe to wallet state changes
   */
  public onWalletStateChange(listener: (state: WalletState | null) => void): () => void {
    this.listeners.push(listener);
    
    // Return unsubscribe function
    return () => {
      const index = this.listeners.indexOf(listener);
      if (index > -1) {
        this.listeners.splice(index, 1);
      }
    };
  }

  /**
   * Notify all listeners of state changes
   */
  private notifyListeners(): void {
    this.listeners.forEach(listener => listener(this.walletState));
  }

  /**
   * Get derived address preview for a mnemonic (without creating wallet)
   */
  public async getAddressPreview(mnemonic: string): Promise<string> {
    await this.initializeKeyring();
    
    if (!this.validateMnemonic(mnemonic)) {
      throw new Error('Invalid mnemonic phrase');
    }

    if (!this.keyring) {
      throw new Error('Keyring not initialized');
    }

    try {
      const tempPair = this.keyring.addFromMnemonic(mnemonic);
      const address = tempPair.address;
      
      // Remove the temporary pair
      this.keyring.removePair(tempPair.address);
      
      return address;
    } catch (error) {
      console.error('Error getting address preview:', error);
      throw new Error('Failed to derive address from mnemonic');
    }
  }
}

// Export singleton instance
export const walletService = WalletService.getInstance();
