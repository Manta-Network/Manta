/**
 * Chameleon Wallet - RPC Service
 * Handles blockchain communication via Polkadot.js API
 */

import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { DEFAULT_NETWORK, TIME } from '@/constants';
import type { NetworkConfig, Balance, Transaction, Validator, StakingInfo } from '@/types';

/**
 * RPC Service for blockchain communication
 */
export class RpcService {
  private static instance: RpcService;
  private api: ApiPromise | null = null;
  private provider: WsProvider | null = null;
  private network: NetworkConfig = DEFAULT_NETWORK;
  private isConnecting = false;
  private connectionPromise: Promise<ApiPromise> | null = null;

  private constructor() {}

  static getInstance(): RpcService {
    if (!RpcService.instance) {
      RpcService.instance = new RpcService();
    }
    return RpcService.instance;
  }

  /**
   * Connect to the blockchain network
   */
  async connect(network?: NetworkConfig): Promise<ApiPromise> {
    if (network) {
      this.network = network;
    }

    // Return existing connection if available
    if (this.api && this.api.isConnected) {
      return this.api;
    }

    // Return existing connection promise if connecting
    if (this.isConnecting && this.connectionPromise) {
      return this.connectionPromise;
    }

    this.isConnecting = true;
    this.connectionPromise = this.establishConnection();

    try {
      const api = await this.connectionPromise;
      this.isConnecting = false;
      return api;
    } catch (error) {
      this.isConnecting = false;
      this.connectionPromise = null;
      throw error;
    }
  }

  private async establishConnection(): Promise<ApiPromise> {
    try {
      // Initialize crypto if not already done
      await cryptoWaitReady();

      // Disconnect existing provider if any
      if (this.provider) {
        await this.provider.disconnect();
      }

      // Create new provider
      this.provider = new WsProvider(this.network.rpcUrl, 1000);

      // Create API instance
      this.api = await ApiPromise.create({
        provider: this.provider,
        types: {
          // Add custom types if needed for Chameleon
        },
      });

      // Wait for API to be ready
      await this.api.isReady;

      console.log(`Connected to ${this.network.name} at ${this.network.rpcUrl}`);
      return this.api;
    } catch (error) {
      console.error('Failed to connect to blockchain:', error);
      throw new Error(`Failed to connect to ${this.network.name}: ${error}`);
    }
  }

  /**
   * Disconnect from the network
   */
  async disconnect(): Promise<void> {
    try {
      if (this.api) {
        await this.api.disconnect();
        this.api = null;
      }
      if (this.provider) {
        await this.provider.disconnect();
        this.provider = null;
      }
      this.isConnecting = false;
      this.connectionPromise = null;
    } catch (error) {
      console.error('Error disconnecting:', error);
    }
  }

  /**
   * Get account balance (public and shielded)
   */
  async getBalance(address: string): Promise<Balance> {
    const api = await this.connect();

    try {
      // Get public balance
      const { data: balance } = await api.query.system.account(address);
      const publicBalance = balance.free.toString();

      // Get shielded balance (custom RPC call)
      let shieldedBalance = '0';
      try {
        // This would be a custom RPC method for Chameleon
        const shielded = await api.rpc.mantaPay?.getShieldedBalance?.(address);
        shieldedBalance = shielded?.toString() || '0';
      } catch (error) {
        console.warn('Shielded balance not available:', error);
      }

      return {
        public: publicBalance,
        shielded: shieldedBalance,
        usdValue: '0', // Will be calculated separately
        lastUpdated: Date.now(),
      };
    } catch (error) {
      console.error('Failed to get balance:', error);
      throw new Error(`Failed to get balance: ${error}`);
    }
  }

  /**
   * Send a transaction
   */
  async sendTransaction(
    fromSeed: string,
    toAddress: string,
    amount: string,
    isPrivate: boolean = false,
    memo?: string
  ): Promise<string> {
    const api = await this.connect();

    try {
      // Create keyring and add sender
      const keyring = new Keyring({ type: 'sr25519', ss58Format: this.network.ss58Prefix });
      const sender = keyring.addFromMnemonic(fromSeed);

      let tx;
      if (isPrivate) {
        // Private transaction using Manta Pay
        // This would require zkSNARK proof generation
        throw new Error('Private transactions not yet implemented');
      } else {
        // Public transaction
        tx = api.tx.balances.transfer(toAddress, amount);
      }

      // Sign and send transaction
      const hash = await tx.signAndSend(sender);
      return hash.toHex();
    } catch (error) {
      console.error('Failed to send transaction:', error);
      throw new Error(`Failed to send transaction: ${error}`);
    }
  }

  /**
   * Shield tokens (public to private)
   */
  async shieldTokens(fromSeed: string, amount: string): Promise<string> {
    const api = await this.connect();

    try {
      const keyring = new Keyring({ type: 'sr25519', ss58Format: this.network.ss58Prefix });
      const sender = keyring.addFromMnemonic(fromSeed);

      // This would require zkSNARK proof generation
      // For now, throw an error indicating it's not implemented
      throw new Error('Shield functionality not yet implemented');
    } catch (error) {
      console.error('Failed to shield tokens:', error);
      throw new Error(`Failed to shield tokens: ${error}`);
    }
  }

  /**
   * Unshield tokens (private to public)
   */
  async unshieldTokens(fromSeed: string, amount: string): Promise<string> {
    const api = await this.connect();

    try {
      const keyring = new Keyring({ type: 'sr25519', ss58Format: this.network.ss58Prefix });
      const sender = keyring.addFromMnemonic(fromSeed);

      // This would require zkSNARK proof generation
      // For now, throw an error indicating it's not implemented
      throw new Error('Unshield functionality not yet implemented');
    } catch (error) {
      console.error('Failed to unshield tokens:', error);
      throw new Error(`Failed to unshield tokens: ${error}`);
    }
  }

  /**
   * Get transaction history
   */
  async getTransactionHistory(address: string, limit: number = 50): Promise<Transaction[]> {
    const api = await this.connect();

    try {
      // This is a simplified implementation
      // In a real implementation, you'd need to query events and extrinsics
      const transactions: Transaction[] = [];

      // Get recent blocks and filter for transactions involving this address
      const latestHeader = await api.rpc.chain.getHeader();
      const latestBlockNumber = latestHeader.number.toNumber();

      // Look back through recent blocks (limited for performance)
      const blocksToCheck = Math.min(100, latestBlockNumber);
      
      for (let i = 0; i < blocksToCheck && transactions.length < limit; i++) {
        const blockNumber = latestBlockNumber - i;
        const blockHash = await api.rpc.chain.getBlockHash(blockNumber);
        const block = await api.rpc.chain.getBlock(blockHash);
        
        // Process extrinsics in the block
        block.block.extrinsics.forEach((extrinsic, index) => {
          if (extrinsic.method.section === 'balances' && extrinsic.method.method === 'transfer') {
            const args = extrinsic.method.args;
            const dest = args[0].toString();
            const value = args[1].toString();
            
            // Check if this transaction involves our address
            if (dest === address || extrinsic.signer?.toString() === address) {
              transactions.push({
                hash: `${blockHash.toHex()}-${index}`,
                type: dest === address ? 'receive' : 'send',
                amount: value,
                timestamp: Date.now() - (i * TIME.BLOCK_TIME * 1000),
                status: 'confirmed',
                isPrivate: false,
                from: extrinsic.signer?.toString(),
                to: dest,
                blockNumber,
                confirmations: i + 1,
              });
            }
          }
        });
      }

      return transactions.slice(0, limit);
    } catch (error) {
      console.error('Failed to get transaction history:', error);
      return [];
    }
  }

  /**
   * Get validators list
   */
  async getValidators(): Promise<Validator[]> {
    const api = await this.connect();

    try {
      const validators = await api.query.staking.validators.entries();
      const validatorList: Validator[] = [];

      for (const [key, prefs] of validators) {
        const address = key.args[0].toString();
        const commission = prefs.commission.toNumber() / 10000000; // Convert from Perbill
        
        // Get staking info
        const exposure = await api.query.staking.erasStakers.entries();
        let totalStake = '0';
        let ownStake = '0';
        let delegatorCount = 0;

        // This is simplified - in reality you'd need to get current era info
        // and calculate proper staking amounts
        
        validatorList.push({
          address,
          commission: (commission * 100).toFixed(2), // Convert to percentage
          totalStake,
          ownStake,
          apy: '12.5', // This would be calculated based on rewards
          isActive: true,
          uptime: '99.8',
          delegatorCount,
        });
      }

      return validatorList;
    } catch (error) {
      console.error('Failed to get validators:', error);
      return [];
    }
  }

  /**
   * Stake tokens to a validator
   */
  async stakeTokens(
    fromSeed: string,
    validatorAddress: string,
    amount: string
  ): Promise<string> {
    const api = await this.connect();

    try {
      const keyring = new Keyring({ type: 'sr25519', ss58Format: this.network.ss58Prefix });
      const sender = keyring.addFromMnemonic(fromSeed);

      // Bond tokens to validator
      const tx = api.tx.staking.bond(validatorAddress, amount, 'Staked');
      const hash = await tx.signAndSend(sender);
      
      return hash.toHex();
    } catch (error) {
      console.error('Failed to stake tokens:', error);
      throw new Error(`Failed to stake tokens: ${error}`);
    }
  }

  /**
   * Unstake tokens
   */
  async unstakeTokens(fromSeed: string, amount: string): Promise<string> {
    const api = await this.connect();

    try {
      const keyring = new Keyring({ type: 'sr25519', ss58Format: this.network.ss58Prefix });
      const sender = keyring.addFromMnemonic(fromSeed);

      // Unbond tokens
      const tx = api.tx.staking.unbond(amount);
      const hash = await tx.signAndSend(sender);
      
      return hash.toHex();
    } catch (error) {
      console.error('Failed to unstake tokens:', error);
      throw new Error(`Failed to unstake tokens: ${error}`);
    }
  }

  /**
   * Get staking information for an address
   */
  async getStakingInfo(address: string): Promise<StakingInfo | null> {
    const api = await this.connect();

    try {
      const ledger = await api.query.staking.ledger(address);
      
      if (ledger.isNone) {
        return null;
      }

      const stakingLedger = ledger.unwrap();
      
      return {
        validatorAddress: stakingLedger.stash.toString(),
        stakedAmount: stakingLedger.total.toString(),
        rewards: '0', // Would need to calculate from rewards
        unbondingAmount: stakingLedger.unlocking.length > 0 ? 
          stakingLedger.unlocking[0].value.toString() : undefined,
        unbondingBlocks: stakingLedger.unlocking.length > 0 ? 
          stakingLedger.unlocking[0].era.toNumber() : undefined,
        isActive: true,
      };
    } catch (error) {
      console.error('Failed to get staking info:', error);
      return null;
    }
  }

  /**
   * Get current block number
   */
  async getCurrentBlockNumber(): Promise<number> {
    const api = await this.connect();
    const header = await api.rpc.chain.getHeader();
    return header.number.toNumber();
  }

  /**
   * Get network info
   */
  async getNetworkInfo(): Promise<{
    blockNumber: number;
    blockTime: number;
    totalIssuance: string;
    activeValidators: number;
  }> {
    const api = await this.connect();

    try {
      const [header, totalIssuance, validators] = await Promise.all([
        api.rpc.chain.getHeader(),
        api.query.balances.totalIssuance(),
        api.query.staking.validators.entries(),
      ]);

      return {
        blockNumber: header.number.toNumber(),
        blockTime: TIME.BLOCK_TIME,
        totalIssuance: totalIssuance.toString(),
        activeValidators: validators.length,
      };
    } catch (error) {
      console.error('Failed to get network info:', error);
      throw error;
    }
  }

  /**
   * Subscribe to balance changes
   */
  subscribeToBalance(
    address: string,
    callback: (balance: Balance) => void
  ): () => void {
    let unsubscribe: (() => void) | null = null;

    this.connect().then(api => {
      api.query.system.account(address, (account: any) => {
        const balance: Balance = {
          public: account.data.free.toString(),
          shielded: '0', // Would need custom query
          usdValue: '0',
          lastUpdated: Date.now(),
        };
        callback(balance);
      }).then(unsub => {
        unsubscribe = unsub;
      });
    });

    return () => {
      if (unsubscribe) {
        unsubscribe();
      }
    };
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.api?.isConnected || false;
  }

  /**
   * Get current network
   */
  getCurrentNetwork(): NetworkConfig {
    return this.network;
  }
}

// Export singleton instance
export const rpcService = RpcService.getInstance();
