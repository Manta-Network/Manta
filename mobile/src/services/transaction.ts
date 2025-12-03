/**
 * Chameleon Wallet - Transaction Service
 * Handles transaction creation, signing, and broadcasting
 */

import { CHAMELEON, FEES } from '../constants';
import type { Transaction, TransactionType, TransactionStatus } from '../types';

export interface TransactionRequest {
  recipient: string;
  amount: string;
  isPrivate: boolean;
  memo?: string;
}

export interface TransactionResult {
  hash: string;
  status: TransactionStatus;
  blockNumber?: number;
  timestamp: number;
  fee: string;
}

export interface FeeEstimate {
  baseFee: string;
  priorityFee: string;
  totalFee: string;
  gasLimit: string;
}

/**
 * Mock transaction service for Week 2 development
 * Will be replaced with real RPC calls in Week 5
 */
export class TransactionService {
  /**
   * Send a transaction (mocked for now)
   */
  static async sendTransaction(
    request: TransactionRequest
  ): Promise<TransactionResult> {
    // Validate inputs
    if (!request.recipient || !request.amount) {
      throw new Error('Recipient and amount are required');
    }

    if (parseFloat(request.amount) <= 0) {
      throw new Error('Amount must be greater than 0');
    }

    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Calculate fee
    const fee = await this.getTransactionFee(request.isPrivate, request.amount);

    // Mock successful transaction
    const result: TransactionResult = {
      hash: `0x${Math.random().toString(16).slice(2).padStart(64, '0')}`,
      status: 'confirmed',
      blockNumber: Math.floor(Math.random() * 1000000) + 500000,
      timestamp: Date.now(),
      fee,
    };

    // Simulate occasional failures (10% chance)
    if (Math.random() < 0.1) {
      result.status = 'failed';
      throw new Error('Transaction failed: Network congestion');
    }

    return result;
  }

  /**
   * Get transaction fee estimate
   */
  static async getTransactionFee(
    isPrivate: boolean,
    amount?: string
  ): Promise<string> {
    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 500));

    if (isPrivate) {
      // Private transaction: 0.02% or 0.1 CHML minimum
      const percentageFee = amount 
        ? (parseFloat(amount) * FEES.SHIELD_PERCENT).toString()
        : '0';
      const calculatedFee = Math.max(
        parseFloat(percentageFee),
        parseFloat(FEES.SHIELD_MIN)
      );
      return calculatedFee.toFixed(6);
    } else {
      // Public transaction: flat 0.1 CHML
      return FEES.BASE_GAS;
    }
  }

  /**
   * Get detailed fee estimate
   */
  static async getFeeEstimate(
    isPrivate: boolean,
    amount: string
  ): Promise<FeeEstimate> {
    const baseFee = FEES.BASE_GAS;
    const priorityFee = isPrivate ? 
      (parseFloat(amount) * FEES.SHIELD_PERCENT).toString() : '0';
    const totalFee = await this.getTransactionFee(isPrivate, amount);

    return {
      baseFee,
      priorityFee,
      totalFee,
      gasLimit: '21000', // Standard gas limit
    };
  }

  /**
   * Validate wallet address
   */
  static validateAddress(address: string): boolean {
    // Basic SS58 address validation
    if (!address || typeof address !== 'string') {
      return false;
    }

    // Check length (SS58 addresses are typically 47-48 characters)
    if (address.length < 47 || address.length > 48) {
      return false;
    }

    // Check if it starts with '5' (Substrate SS58 format)
    if (!address.startsWith('5')) {
      return false;
    }

    // Check if it contains only valid base58 characters
    const base58Regex = /^[123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz]+$/;
    return base58Regex.test(address);
  }

  /**
   * Validate transaction amount
   */
  static validateAmount(amount: string, balance: string): {
    isValid: boolean;
    error?: string;
  } {
    const numAmount = parseFloat(amount);
    const numBalance = parseFloat(balance);

    if (isNaN(numAmount) || numAmount <= 0) {
      return {
        isValid: false,
        error: 'Invalid amount',
      };
    }

    if (numAmount < parseFloat(CHAMELEON.EXISTENTIAL_DEPOSIT)) {
      return {
        isValid: false,
        error: `Minimum amount is ${CHAMELEON.EXISTENTIAL_DEPOSIT} CHML`,
      };
    }

    if (numAmount > numBalance) {
      return {
        isValid: false,
        error: 'Insufficient balance',
      };
    }

    return { isValid: true };
  }

  /**
   * Shield tokens (convert public to private)
   */
  static async shieldTokens(amount: string): Promise<TransactionResult> {
    // Simulate shield transaction
    await new Promise(resolve => setTimeout(resolve, 3000)); // Longer for privacy proof

    const fee = await this.getTransactionFee(true, amount);

    return {
      hash: `0x${Math.random().toString(16).slice(2).padStart(64, '0')}`,
      status: 'confirmed',
      blockNumber: Math.floor(Math.random() * 1000000) + 500000,
      timestamp: Date.now(),
      fee,
    };
  }

  /**
   * Unshield tokens (convert private to public)
   */
  static async unshieldTokens(amount: string): Promise<TransactionResult> {
    // Simulate unshield transaction
    await new Promise(resolve => setTimeout(resolve, 2500)); // Longer for privacy proof

    const fee = await this.getTransactionFee(false, amount);

    return {
      hash: `0x${Math.random().toString(16).slice(2).padStart(64, '0')}`,
      status: 'confirmed',
      blockNumber: Math.floor(Math.random() * 1000000) + 500000,
      timestamp: Date.now(),
      fee,
    };
  }

  /**
   * Get transaction status
   */
  static async getTransactionStatus(hash: string): Promise<TransactionStatus> {
    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Mock status progression: pending -> confirmed
    const statuses: TransactionStatus[] = ['pending', 'confirmed'];
    return statuses[Math.floor(Math.random() * statuses.length)];
  }

  /**
   * Calculate maximum sendable amount (balance - fee)
   */
  static async getMaxSendableAmount(
    balance: string,
    isPrivate: boolean
  ): Promise<string> {
    const numBalance = parseFloat(balance);
    const fee = parseFloat(await this.getTransactionFee(isPrivate, balance));
    const maxAmount = Math.max(0, numBalance - fee);
    return maxAmount.toFixed(6);
  }
}

/**
 * Transaction history mock data
 */
export const mockTransactionHistory: Transaction[] = [
  {
    hash: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
    type: 'receive',
    amount: '100.500000',
    timestamp: Date.now() - 3600000, // 1 hour ago
    status: 'confirmed',
    isPrivate: true,
    from: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    to: 'self',
    fee: '0.100000',
    memo: 'Payment for services',
    blockNumber: 1234567,
    confirmations: 12,
  },
  {
    hash: '0x5678901234567890abcdef1234567890abcdef1234567890abcdef1234567890',
    type: 'send',
    amount: '25.000000',
    timestamp: Date.now() - 7200000, // 2 hours ago
    status: 'confirmed',
    isPrivate: false,
    from: 'self',
    to: '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy',
    fee: '0.100000',
    blockNumber: 1234550,
    confirmations: 25,
  },
  {
    hash: '0x9abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
    type: 'shield',
    amount: '50.000000',
    timestamp: Date.now() - 14400000, // 4 hours ago
    status: 'confirmed',
    isPrivate: true,
    from: 'self',
    to: 'self',
    fee: '0.120000',
    memo: 'Shield for privacy',
    blockNumber: 1234520,
    confirmations: 45,
  },
  {
    hash: '0xdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcd',
    type: 'receive',
    amount: '5.750000',
    timestamp: Date.now() - 86400000, // 1 day ago
    status: 'confirmed',
    isPrivate: false,
    from: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
    to: 'self',
    fee: '0.100000',
    blockNumber: 1233000,
    confirmations: 156,
  },
  {
    hash: '0x1111222233334444555566667777888899990000aaaabbbbccccddddeeeeffff',
    type: 'send',
    amount: '12.345000',
    timestamp: Date.now() - 172800000, // 2 days ago
    status: 'failed',
    isPrivate: true,
    from: 'self',
    to: '5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY',
    fee: '0.120000',
    memo: 'Failed transaction',
    blockNumber: 1232500,
  },
];
