/**
 * Chameleon Wallet - Mock Transaction Data
 * For development and testing purposes
 */

import type { Transaction } from '../types';

/**
 * Mock transaction history for development
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
    memo: 'Payment for services rendered',
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
    memo: 'Monthly subscription',
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
    memo: 'Enhanced privacy protection',
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
    memo: 'Transaction failed due to network congestion',
    blockNumber: 1232500,
  },
  {
    hash: '0x2222333344445555666677778888999900001111aaaabbbbccccddddeeeeffff',
    type: 'unshield',
    amount: '15.000000',
    timestamp: Date.now() - 259200000, // 3 days ago
    status: 'confirmed',
    isPrivate: false,
    from: 'self',
    to: 'self',
    fee: '0.110000',
    memo: 'Convert to public for exchange',
    blockNumber: 1232000,
    confirmations: 200,
  },
  {
    hash: '0x3333444455556666777788889999000011112222aaaabbbbccccddddeeeeffff',
    type: 'receive',
    amount: '200.000000',
    timestamp: Date.now() - 345600000, // 4 days ago
    status: 'confirmed',
    isPrivate: true,
    from: '5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z',
    to: 'self',
    fee: '0.100000',
    memo: 'Salary payment',
    blockNumber: 1231500,
    confirmations: 250,
  },
  {
    hash: '0x4444555566667777888899990000111122223333aaaabbbbccccddddeeeeffff',
    type: 'send',
    amount: '8.500000',
    timestamp: Date.now() - 432000000, // 5 days ago
    status: 'confirmed',
    isPrivate: false,
    from: 'self',
    to: '5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL',
    fee: '0.100000',
    memo: 'Coffee shop payment',
    blockNumber: 1231000,
    confirmations: 300,
  },
  {
    hash: '0x5555666677778888999900001111222233334444aaaabbbbccccddddeeeeffff',
    type: 'stake',
    amount: '1000.000000',
    timestamp: Date.now() - 604800000, // 1 week ago
    status: 'confirmed',
    isPrivate: false,
    from: 'self',
    to: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY', // Validator
    fee: '0.100000',
    memo: 'Staking delegation',
    blockNumber: 1230000,
    confirmations: 500,
  },
  {
    hash: '0x6666777788889999000011112222333344445555aaaabbbbccccddddeeeeffff',
    type: 'receive',
    amount: '3.250000',
    timestamp: Date.now() - 691200000, // 8 days ago
    status: 'confirmed',
    isPrivate: true,
    from: '5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y',
    to: 'self',
    fee: '0.100000',
    memo: 'Freelance work payment',
    blockNumber: 1229500,
    confirmations: 600,
  },
];

/**
 * Mock pending transactions
 */
export const mockPendingTransactions: Transaction[] = [
  {
    hash: '0xpending1111222233334444555566667777888899990000aaaabbbbccccdddd',
    type: 'send',
    amount: '15.750000',
    timestamp: Date.now() - 300000, // 5 minutes ago
    status: 'pending',
    isPrivate: true,
    from: 'self',
    to: '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy',
    fee: '0.120000',
    memo: 'Pending private transfer',
  },
  {
    hash: '0xpending2222333344445555666677778888999900001111aaaabbbbccccdddd',
    type: 'shield',
    amount: '30.000000',
    timestamp: Date.now() - 180000, // 3 minutes ago
    status: 'pending',
    isPrivate: true,
    from: 'self',
    to: 'self',
    fee: '0.130000',
    memo: 'Shielding for privacy',
  },
];

/**
 * Get mock transactions with optional filtering
 */
export function getMockTransactions(options?: {
  limit?: number;
  type?: string;
  status?: string;
  includePending?: boolean;
}): Transaction[] {
  let transactions = [...mockTransactionHistory];
  
  if (options?.includePending) {
    transactions = [...mockPendingTransactions, ...transactions];
  }
  
  if (options?.type) {
    transactions = transactions.filter(tx => tx.type === options.type);
  }
  
  if (options?.status) {
    transactions = transactions.filter(tx => tx.status === options.status);
  }
  
  if (options?.limit) {
    transactions = transactions.slice(0, options.limit);
  }
  
  return transactions.sort((a, b) => b.timestamp - a.timestamp);
}

/**
 * Generate a random mock transaction
 */
export function generateMockTransaction(type: 'send' | 'receive' = 'receive'): Transaction {
  const addresses = [
    '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy',
    '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
    '5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z',
    '5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL',
  ];
  
  const memos = [
    'Payment for services',
    'Monthly subscription',
    'Freelance work',
    'Coffee purchase',
    'Grocery shopping',
    'Rent payment',
    'Utility bill',
    'Gift to friend',
  ];
  
  const randomAddress = addresses[Math.floor(Math.random() * addresses.length)];
  const randomMemo = memos[Math.floor(Math.random() * memos.length)];
  const randomAmount = (Math.random() * 100 + 1).toFixed(6);
  const isPrivate = Math.random() > 0.5;
  
  return {
    hash: `0x${Math.random().toString(16).slice(2).padStart(64, '0')}`,
    type,
    amount: randomAmount,
    timestamp: Date.now(),
    status: 'confirmed',
    isPrivate,
    from: type === 'send' ? 'self' : randomAddress,
    to: type === 'receive' ? 'self' : randomAddress,
    fee: isPrivate ? '0.120000' : '0.100000',
    memo: randomMemo,
    blockNumber: Math.floor(Math.random() * 1000000) + 1000000,
    confirmations: Math.floor(Math.random() * 100) + 1,
  };
}
