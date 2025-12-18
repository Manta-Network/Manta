/**
 * Formatting utilities for the Chameleon wallet
 */

import { BN } from '@polkadot/util';
import { NETWORK_CONFIG } from '../config/network';

/**
 * Format a number with thousand separators
 */
export function formatNumber(num: number | string): string {
  const numStr = typeof num === 'number' ? num.toString() : num;
  return numStr.replace(/\B(?=(\d{3})+(?!\d))/g, ',');
}

/**
 * Format balance from planck units to human readable
 */
export function formatTokenAmount(
  amount: string | BN, 
  decimals: number = NETWORK_CONFIG.tokenDecimals,
  symbol: string = NETWORK_CONFIG.tokenSymbol,
  maxDecimals: number = 6
): string {
  const balance = new BN(amount.toString());
  
  if (balance.isZero()) {
    return `0 ${symbol}`;
  }
  
  // Convert from planck units to token units
  const divisor = new BN(10).pow(new BN(decimals));
  const tokenAmount = balance.div(divisor);
  const remainder = balance.mod(divisor);
  
  // Format integer part with thousand separators
  let formatted = formatNumber(tokenAmount.toString());
  
  // Add decimal part if there's a remainder
  if (!remainder.isZero()) {
    const remainderStr = remainder.toString().padStart(decimals, '0');
    // Remove trailing zeros and limit decimal places
    const decimalPart = remainderStr.replace(/0+$/, '').substring(0, maxDecimals);
    if (decimalPart.length > 0) {
      formatted += '.' + decimalPart;
    }
  }
  
  return `${formatted} ${symbol}`;
}

/**
 * Format USD value
 */
export function formatUSD(amount: number): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: 6,
  }).format(amount);
}

/**
 * Format percentage
 */
export function formatPercentage(value: number, decimals: number = 2): string {
  return `${value.toFixed(decimals)}%`;
}

/**
 * Format time duration (e.g., "2h 30m")
 */
export function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;
  
  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  } else if (minutes > 0) {
    return `${minutes}m ${secs}s`;
  } else {
    return `${secs}s`;
  }
}

/**
 * Format block time (e.g., "2 minutes ago")
 */
export function formatTimeAgo(timestamp: number): string {
  const now = Date.now();
  const diff = Math.floor((now - timestamp) / 1000);
  
  if (diff < 60) {
    return `${diff} seconds ago`;
  } else if (diff < 3600) {
    const minutes = Math.floor(diff / 60);
    return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
  } else if (diff < 86400) {
    const hours = Math.floor(diff / 3600);
    return `${hours} hour${hours > 1 ? 's' : ''} ago`;
  } else {
    const days = Math.floor(diff / 86400);
    return `${days} day${days > 1 ? 's' : ''} ago`;
  }
}

/**
 * Parse token amount from user input to planck units
 */
export function parseTokenAmount(
  input: string, 
  decimals: number = NETWORK_CONFIG.tokenDecimals
): string {
  // Remove any non-numeric characters except decimal point
  const cleaned = input.replace(/[^0-9.]/g, '');
  
  if (!cleaned || cleaned === '.') {
    return '0';
  }
  
  // Split into integer and decimal parts
  const parts = cleaned.split('.');
  const integerPart = parts[0] || '0';
  const decimalPart = (parts[1] || '').substring(0, decimals); // Limit to token decimals
  
  // Pad decimal part to full decimals
  const paddedDecimal = decimalPart.padEnd(decimals, '0');
  
  // Combine and convert to BN
  const fullAmount = integerPart + paddedDecimal;
  return new BN(fullAmount).toString();
}
