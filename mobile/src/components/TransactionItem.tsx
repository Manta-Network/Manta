/**
 * Chameleon Wallet - Transaction Item Component
 */

import React from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
} from 'react-native';
import { formatCHML, formatTimeAgo, truncateAddress } from '@/utils/format';
import { colors, fonts } from '@/theme';
import { CHAMELEON } from '@/constants';
import type { Transaction } from '@/types';

interface TransactionItemProps {
  transaction: Transaction;
  onPress?: () => void;
  walletAddress?: string;
}

export function TransactionItem({
  transaction,
  onPress,
  walletAddress,
}: TransactionItemProps) {
  const getTransactionIcon = () => {
    switch (transaction.type) {
      case 'send':
        return '↗️';
      case 'receive':
        return '↙️';
      case 'shield':
        return '🛡️';
      case 'unshield':
        return '🔓';
      case 'stake':
        return '🏦';
      case 'unstake':
        return '📤';
      case 'swap':
        return '🔄';
      default:
        return '💰';
    }
  };

  const getTransactionTitle = () => {
    switch (transaction.type) {
      case 'send':
        return 'Sent';
      case 'receive':
        return 'Received';
      case 'shield':
        return 'Shielded';
      case 'unshield':
        return 'Unshielded';
      case 'stake':
        return 'Staked';
      case 'unstake':
        return 'Unstaked';
      case 'swap':
        return 'Swapped';
      default:
        return 'Transaction';
    }
  };

  const getTransactionSubtitle = () => {
    if (transaction.type === 'send' && transaction.to) {
      return `To ${truncateAddress(transaction.to)}`;
    }
    if (transaction.type === 'receive' && transaction.from) {
      return `From ${truncateAddress(transaction.from)}`;
    }
    if (transaction.type === 'stake' && transaction.to) {
      return `To ${truncateAddress(transaction.to)}`;
    }
    if (transaction.isPrivate) {
      return 'Private transaction';
    }
    return 'Public transaction';
  };

  const getStatusColor = () => {
    switch (transaction.status) {
      case 'confirmed':
        return colors.success;
      case 'pending':
        return colors.warning;
      case 'failed':
        return colors.error;
      default:
        return colors.textSecondary;
    }
  };

  const getAmountColor = () => {
    if (transaction.type === 'receive') {
      return colors.success;
    }
    if (transaction.type === 'send') {
      return colors.error;
    }
    return colors.text;
  };

  const getAmountPrefix = () => {
    if (transaction.type === 'receive') {
      return '+';
    }
    if (transaction.type === 'send') {
      return '-';
    }
    return '';
  };

  const formattedAmount = formatCHML(transaction.amount);
  const timeAgo = formatTimeAgo(transaction.timestamp);

  return (
    <TouchableOpacity
      style={styles.container}
      onPress={onPress}
      activeOpacity={0.8}
    >
      <View style={styles.iconContainer}>
        <Text style={styles.icon}>{getTransactionIcon()}</Text>
        {transaction.isPrivate && (
          <View style={styles.privacyBadge}>
            <Text style={styles.privacyText}>🔒</Text>
          </View>
        )}
      </View>

      <View style={styles.content}>
        <View style={styles.header}>
          <Text style={styles.title}>{getTransactionTitle()}</Text>
          <Text style={[styles.amount, { color: getAmountColor() }]}>
            {getAmountPrefix()}{formattedAmount} {CHAMELEON.TOKEN_SYMBOL}
          </Text>
        </View>

        <View style={styles.details}>
          <Text style={styles.subtitle}>{getTransactionSubtitle()}</Text>
          <View style={styles.statusContainer}>
            <View
              style={[
                styles.statusIndicator,
                { backgroundColor: getStatusColor() },
              ]}
            />
            <Text style={[styles.status, { color: getStatusColor() }]}>
              {transaction.status}
            </Text>
          </View>
        </View>

        <View style={styles.footer}>
          <Text style={styles.time}>{timeAgo}</Text>
          {transaction.confirmations && (
            <Text style={styles.confirmations}>
              {transaction.confirmations} confirmations
            </Text>
          )}
        </View>
      </View>
    </TouchableOpacity>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    padding: 16,
    backgroundColor: colors.surface,
    borderRadius: 12,
    marginHorizontal: 16,
    marginVertical: 4,
  },
  
  iconContainer: {
    position: 'relative',
    marginRight: 16,
  },
  
  icon: {
    fontSize: 24,
    width: 40,
    height: 40,
    textAlign: 'center',
    lineHeight: 40,
    backgroundColor: colors.background,
    borderRadius: 20,
  },
  
  privacyBadge: {
    position: 'absolute',
    top: -4,
    right: -4,
    width: 16,
    height: 16,
    borderRadius: 8,
    backgroundColor: colors.secondary,
    alignItems: 'center',
    justifyContent: 'center',
  },
  
  privacyText: {
    fontSize: 8,
  },
  
  content: {
    flex: 1,
  },
  
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 4,
  },
  
  title: {
    fontSize: 16,
    fontFamily: fonts.medium,
    color: colors.text,
  },
  
  amount: {
    fontSize: 16,
    fontFamily: fonts.medium,
  },
  
  details: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  
  subtitle: {
    fontSize: 14,
    fontFamily: fonts.regular,
    color: colors.textSecondary,
    flex: 1,
  },
  
  statusContainer: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  
  statusIndicator: {
    width: 6,
    height: 6,
    borderRadius: 3,
    marginRight: 6,
  },
  
  status: {
    fontSize: 12,
    fontFamily: fonts.medium,
    textTransform: 'capitalize',
  },
  
  footer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  
  time: {
    fontSize: 12,
    fontFamily: fonts.regular,
    color: colors.textSecondary,
  },
  
  confirmations: {
    fontSize: 12,
    fontFamily: fonts.regular,
    color: colors.textSecondary,
  },
});
