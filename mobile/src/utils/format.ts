/**
 * Chameleon Wallet - Formatting Utilities
 */

import { CHAMELEON } from '@/constants';

/**
 * Format CHML amount from base units to human readable
 * @param amount Amount in base units (wei equivalent)
 * @param decimals Number of decimal places to show
 * @returns Formatted amount string
 */
export function formatCHML(amount: string | number, decimals: number = 4): string {
  const amountBN = BigInt(amount.toString());
  const divisor = BigInt(10 ** CHAMELEON.DECIMALS);
  
  const wholePart = amountBN / divisor;
  const fractionalPart = amountBN % divisor;
  
  if (fractionalPart === 0n) {
    return wholePart.toString();
  }
  
  const fractionalStr = fractionalPart.toString().padStart(CHAMELEON.DECIMALS, '0');
  const trimmedFractional = fractionalStr.slice(0, decimals).replace(/0+$/, '');
  
  if (trimmedFractional === '') {
    return wholePart.toString();
  }
  
  return `${wholePart}.${trimmedFractional}`;
}

/**
 * Parse CHML amount from human readable to base units
 * @param amount Human readable amount
 * @returns Amount in base units as string
 */
export function parseCHML(amount: string): string {
  const cleanAmount = amount.replace(/,/g, '').trim();
  
  if (!/^\d*\.?\d*$/.test(cleanAmount)) {
    throw new Error('Invalid number format');
  }
  
  const [wholePart = '0', fractionalPart = ''] = cleanAmount.split('.');
  const paddedFractional = fractionalPart.padEnd(CHAMELEON.DECIMALS, '0');
  
  if (paddedFractional.length > CHAMELEON.DECIMALS) {
    throw new Error(`Too many decimal places. Maximum ${CHAMELEON.DECIMALS} allowed.`);
  }
  
  const result = BigInt(wholePart) * BigInt(10 ** CHAMELEON.DECIMALS) + BigInt(paddedFractional);
  return result.toString();
}

/**
 * Format USD amount
 * @param amount USD amount
 * @param decimals Number of decimal places
 * @returns Formatted USD string
 */
export function formatUSD(amount: string | number, decimals: number = 2): string {
  const num = typeof amount === 'string' ? parseFloat(amount) : amount;
  
  if (isNaN(num)) {
    return '$0.00';
  }
  
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  }).format(num);
}

/**
 * Format percentage
 * @param value Percentage value (0-1 or 0-100)
 * @param decimals Number of decimal places
 * @param isDecimal Whether input is decimal (0-1) or percentage (0-100)
 * @returns Formatted percentage string
 */
export function formatPercentage(
  value: string | number,
  decimals: number = 2,
  isDecimal: boolean = true
): string {
  const num = typeof value === 'string' ? parseFloat(value) : value;
  
  if (isNaN(num)) {
    return '0%';
  }
  
  const percentage = isDecimal ? num * 100 : num;
  
  return new Intl.NumberFormat('en-US', {
    style: 'percent',
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  }).format(percentage / 100);
}

/**
 * Format large numbers with K, M, B suffixes
 * @param value Number to format
 * @param decimals Number of decimal places
 * @returns Formatted number string
 */
export function formatLargeNumber(value: string | number, decimals: number = 1): string {
  const num = typeof value === 'string' ? parseFloat(value) : value;
  
  if (isNaN(num)) {
    return '0';
  }
  
  const absNum = Math.abs(num);
  const sign = num < 0 ? '-' : '';
  
  if (absNum >= 1e9) {
    return `${sign}${(absNum / 1e9).toFixed(decimals)}B`;
  } else if (absNum >= 1e6) {
    return `${sign}${(absNum / 1e6).toFixed(decimals)}M`;
  } else if (absNum >= 1e3) {
    return `${sign}${(absNum / 1e3).toFixed(decimals)}K`;
  }
  
  return `${sign}${absNum.toFixed(decimals)}`;
}

/**
 * Truncate address for display
 * @param address Full address
 * @param startChars Number of characters to show at start
 * @param endChars Number of characters to show at end
 * @returns Truncated address
 */
export function truncateAddress(
  address: string,
  startChars: number = 6,
  endChars: number = 4
): string {
  if (address.length <= startChars + endChars) {
    return address;
  }
  
  return `${address.slice(0, startChars)}...${address.slice(-endChars)}`;
}

/**
 * Format transaction hash for display
 * @param hash Transaction hash
 * @returns Formatted hash
 */
export function formatTxHash(hash: string): string {
  return truncateAddress(hash, 8, 6);
}

/**
 * Format time ago (relative time)
 * @param timestamp Unix timestamp in milliseconds
 * @returns Relative time string
 */
export function formatTimeAgo(timestamp: number): string {
  const now = Date.now();
  const diff = now - timestamp;
  
  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);
  
  if (days > 0) {
    return `${days}d ago`;
  } else if (hours > 0) {
    return `${hours}h ago`;
  } else if (minutes > 0) {
    return `${minutes}m ago`;
  } else {
    return 'Just now';
  }
}

/**
 * Format date for display
 * @param timestamp Unix timestamp in milliseconds
 * @param format Format type
 * @returns Formatted date string
 */
export function formatDate(
  timestamp: number,
  format: 'short' | 'long' | 'time' = 'short'
): string {
  const date = new Date(timestamp);
  
  switch (format) {
    case 'short':
      return date.toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        year: 'numeric',
      });
    case 'long':
      return date.toLocaleDateString('en-US', {
        weekday: 'long',
        year: 'numeric',
        month: 'long',
        day: 'numeric',
      });
    case 'time':
      return date.toLocaleTimeString('en-US', {
        hour: '2-digit',
        minute: '2-digit',
      });
    default:
      return date.toLocaleDateString();
  }
}

/**
 * Format block number with commas
 * @param blockNumber Block number
 * @returns Formatted block number
 */
export function formatBlockNumber(blockNumber: number): string {
  return new Intl.NumberFormat('en-US').format(blockNumber);
}

/**
 * Format APY (Annual Percentage Yield)
 * @param apy APY value (0-1 or 0-100)
 * @param isDecimal Whether input is decimal (0-1) or percentage (0-100)
 * @returns Formatted APY string
 */
export function formatAPY(apy: string | number, isDecimal: boolean = false): string {
  return formatPercentage(apy, 2, isDecimal);
}

/**
 * Format validator commission
 * @param commission Commission rate (0-1 or 0-100)
 * @param isDecimal Whether input is decimal (0-1) or percentage (0-100)
 * @returns Formatted commission string
 */
export function formatCommission(commission: string | number, isDecimal: boolean = false): string {
  return formatPercentage(commission, 1, isDecimal);
}

/**
 * Format slippage tolerance
 * @param slippage Slippage value (0-1 or 0-100)
 * @param isDecimal Whether input is decimal (0-1) or percentage (0-100)
 * @returns Formatted slippage string
 */
export function formatSlippage(slippage: string | number, isDecimal: boolean = true): string {
  return formatPercentage(slippage, 1, isDecimal);
}

/**
 * Validate and format user input amount
 * @param input User input string
 * @param maxDecimals Maximum decimal places allowed
 * @returns Formatted input or null if invalid
 */
export function validateAndFormatInput(
  input: string,
  maxDecimals: number = CHAMELEON.DECIMALS
): string | null {
  // Remove any non-numeric characters except decimal point
  const cleaned = input.replace(/[^0-9.]/g, '');
  
  // Check for multiple decimal points
  const decimalCount = (cleaned.match(/\./g) || []).length;
  if (decimalCount > 1) {
    return null;
  }
  
  // Check decimal places
  const parts = cleaned.split('.');
  if (parts[1] && parts[1].length > maxDecimals) {
    return null;
  }
  
  return cleaned;
}

/**
 * Format seed phrase for display (with word numbers)
 * @param seedPhrase Seed phrase string
 * @returns Array of formatted seed words with numbers
 */
export function formatSeedPhrase(seedPhrase: string): Array<{ number: number; word: string }> {
  const words = seedPhrase.trim().split(/\s+/);
  return words.map((word, index) => ({
    number: index + 1,
    word: word.toLowerCase(),
  }));
}

/**
 * Format network name for display
 * @param networkName Network name
 * @returns Formatted network name
 */
export function formatNetworkName(networkName: string): string {
  return networkName
    .split(/[-_\s]+/)
    .map(word => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
    .join(' ');
}
