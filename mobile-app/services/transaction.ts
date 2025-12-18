/**
 * Transaction service for Chameleon Network
 * Handles sending, fee estimation, and transaction monitoring
 */

import { ApiPromise } from '@polkadot/api';
import { BN } from '@polkadot/util';
import type { KeyringPair } from '@polkadot/keyring/types';
import type { SubmittableExtrinsic } from '@polkadot/api/types';
import type { ISubmittableResult } from '@polkadot/types/types';
import { apiService } from './api';
import { chainService } from './chain';

export type TransactionStatus = 'pending' | 'inBlock' | 'finalized' | 'failed';

export interface TransactionResult {
  hash: string;
  status: TransactionStatus;
  blockHash?: string;
  blockNumber?: number;
  error?: string;
}

export interface FeeEstimate {
  partialFee: string;
  weight: string;
  class: string;
  formatted: string;
}

export interface TransactionInfo {
  hash: string;
  from: string;
  to: string;
  amount: string;
  fee: string;
  status: TransactionStatus;
  timestamp: number;
  blockNumber?: number;
  blockHash?: string;
}

class TransactionService {
  private static instance: TransactionService;
  private watchedTransactions = new Map<string, (result: TransactionResult) => void>();
  
  private constructor() {}

  public static getInstance(): TransactionService {
    if (!TransactionService.instance) {
      TransactionService.instance = new TransactionService();
    }
    return TransactionService.instance;
  }

  /**
   * Send a transfer transaction
   */
  public async sendTransaction(
    from: KeyringPair,
    to: string,
    amount: BN
  ): Promise<TransactionResult> {
    const api = apiService.getApi();
    if (!api) {
      throw new Error('API not connected');
    }

    try {
      // Create transfer extrinsic using transferKeepAlive to prevent account reaping
      const tx = api.tx.balances.transferKeepAlive(to, amount);
      
      // Sign and send the transaction
      const hash = await tx.signAndSend(from);
      
      return {
        hash: hash.toHex(),
        status: 'pending',
      };
    } catch (error) {
      console.error('Error sending transaction:', error);
      throw new Error(`Failed to send transaction: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Send transaction with status monitoring
   */
  public async sendTransactionWithMonitoring(
    from: KeyringPair,
    to: string,
    amount: BN,
    onStatusUpdate: (result: TransactionResult) => void
  ): Promise<string> {
    const api = apiService.getApi();
    if (!api) {
      throw new Error('API not connected');
    }

    return new Promise((resolve, reject) => {
      const tx = api.tx.balances.transferKeepAlive(to, amount);
      
      tx.signAndSend(from, (result: ISubmittableResult) => {
        const hash = result.txHash.toHex();
        
        if (result.status.isInBlock) {
          const blockHash = result.status.asInBlock.toHex();
          onStatusUpdate({
            hash,
            status: 'inBlock',
            blockHash,
          });
        } else if (result.status.isFinalized) {
          const blockHash = result.status.asFinalized.toHex();
          
          // Check for transaction success/failure
          const success = result.events.some(({ event }) => 
            api.events.system.ExtrinsicSuccess.is(event)
          );
          
          const failed = result.events.some(({ event }) => 
            api.events.system.ExtrinsicFailed.is(event)
          );
          
          if (failed) {
            const errorEvent = result.events.find(({ event }) => 
              api.events.system.ExtrinsicFailed.is(event)
            );
            
            let errorMessage = 'Transaction failed';
            if (errorEvent) {
              const [dispatchError] = errorEvent.event.data;
              if (dispatchError.isModule) {
                const decoded = api.registry.findMetaError(dispatchError.asModule);
                errorMessage = `${decoded.section}.${decoded.name}: ${decoded.docs.join(' ')}`;
              }
            }
            
            onStatusUpdate({
              hash,
              status: 'failed',
              blockHash,
              error: errorMessage,
            });
            reject(new Error(errorMessage));
          } else if (success) {
            onStatusUpdate({
              hash,
              status: 'finalized',
              blockHash,
            });
            resolve(hash);
          }
        } else if (result.isError) {
          const error = 'Transaction error';
          onStatusUpdate({
            hash,
            status: 'failed',
            error,
          });
          reject(new Error(error));
        }
      }).catch((error) => {
        console.error('Error in signAndSend:', error);
        reject(error);
      });
    });
  }

  /**
   * Estimate transaction fee
   */
  public async estimateFee(
    from: string,
    to: string,
    amount: BN
  ): Promise<FeeEstimate> {
    const api = apiService.getApi();
    if (!api) {
      throw new Error('API not connected');
    }

    try {
      const tx = api.tx.balances.transferKeepAlive(to, amount);
      const info = await tx.paymentInfo(from);
      
      const partialFee = info.partialFee.toString();
      const formatted = chainService.formatBalance(partialFee);
      
      return {
        partialFee,
        weight: info.weight.toString(),
        class: info.class.toString(),
        formatted,
      };
    } catch (error) {
      console.error('Error estimating fee:', error);
      throw new Error(`Failed to estimate fee: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Watch transaction status
   */
  public watchTransaction(
    hash: string,
    callback: (result: TransactionResult) => void
  ): () => void {
    this.watchedTransactions.set(hash, callback);
    
    // Start monitoring (simplified - in production you'd use WebSocket subscriptions)
    this.monitorTransaction(hash, callback);
    
    // Return unsubscribe function
    return () => {
      this.watchedTransactions.delete(hash);
    };
  }

  /**
   * Monitor transaction status (simplified implementation)
   */
  private async monitorTransaction(
    hash: string,
    callback: (result: TransactionResult) => void
  ): Promise<void> {
    const api = apiService.getApi();
    if (!api) {
      return;
    }

    try {
      // Check if transaction is in a block
      const signedBlock = await api.rpc.chain.getBlock();
      const blockHash = signedBlock.block.header.hash.toHex();
      const blockNumber = signedBlock.block.header.number.toNumber();
      
      // This is a simplified check - in production you'd need more sophisticated monitoring
      callback({
        hash,
        status: 'inBlock',
        blockHash,
        blockNumber,
      });
      
      // Simulate finalization after a delay
      setTimeout(() => {
        callback({
          hash,
          status: 'finalized',
          blockHash,
          blockNumber,
        });
      }, 6000); // 6 seconds for finalization
      
    } catch (error) {
      console.error('Error monitoring transaction:', error);
      callback({
        hash,
        status: 'failed',
        error: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  }

  /**
   * Get transaction history for an address (placeholder)
   * In production, this would query an indexer or scan blocks
   */
  public async getTransactionHistory(address: string): Promise<TransactionInfo[]> {
    // Placeholder implementation
    // In production, you would:
    // 1. Query a blockchain indexer (like SubQuery or Subsquid)
    // 2. Or scan recent blocks for transactions involving this address
    // 3. Store transaction history locally for offline access
    
    console.log('Getting transaction history for:', address);
    
    // Return empty array for now
    return [];
  }

  /**
   * Validate address format
   */
  public validateAddress(address: string): boolean {
    const api = apiService.getApi();
    if (!api) {
      // Basic validation without API
      return address.length >= 47 && address.length <= 48 && address.startsWith('5');
    }

    try {
      // Use Polkadot.js validation
      api.createType('AccountId', address);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Parse amount string to BN (handles decimal input)
   */
  public parseAmount(amount: string, decimals: number = 18): BN {
    if (!amount || amount === '0') {
      return new BN(0);
    }

    // Remove any commas and trim
    const cleanAmount = amount.replace(/,/g, '').trim();
    
    // Split by decimal point
    const parts = cleanAmount.split('.');
    if (parts.length > 2) {
      throw new Error('Invalid amount format');
    }

    const wholePart = parts[0] || '0';
    const decimalPart = parts[1] || '';
    
    // Pad or truncate decimal part to match token decimals
    const paddedDecimalPart = decimalPart.padEnd(decimals, '0').substring(0, decimals);
    
    // Combine and convert to BN
    const fullAmountStr = wholePart + paddedDecimalPart;
    return new BN(fullAmountStr);
  }

  /**
   * Format BN amount to display string
   */
  public formatAmount(amount: BN, decimals: number = 18): string {
    return chainService.formatBalance(amount.toString(), decimals);
  }
}

// Export singleton instance
export const transactionService = TransactionService.getInstance();
