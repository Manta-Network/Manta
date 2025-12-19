/**
 * ActivityCard component for displaying transaction history
 */

import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { THEME } from '../constants/theme';
import { truncateAddress } from '../utils/address';

export type TransactionType = 'deposit' | 'withdrawal' | 'send' | 'receive' | 'shield' | 'unshield';
export type TransactionStatus = 'success' | 'pending' | 'failed';

interface ActivityCardProps {
  type: TransactionType;
  transactionId: string;
  amount: string;
  currency?: string;
  status: TransactionStatus;
  date?: string;
  account?: string;
  onPress?: () => void;
}

const getTransactionIcon = (type: TransactionType): string => {
  switch (type) {
    case 'deposit':
    case 'receive':
      return 'arrow-down';
    case 'withdrawal':
    case 'send':
      return 'arrow-up';
    case 'shield':
      return 'shield';
    case 'unshield':
      return 'shield-outline';
    default:
      return 'swap-horizontal';
  }
};

const getTransactionLabel = (type: TransactionType): string => {
  switch (type) {
    case 'deposit':
      return 'Deposit';
    case 'withdrawal':
      return 'Withdrawal';
    case 'send':
      return 'Sent';
    case 'receive':
      return 'Received';
    case 'shield':
      return 'Shielded';
    case 'unshield':
      return 'Unshielded';
    default:
      return 'Transaction';
  }
};

const getStatusConfig = (status: TransactionStatus) => {
  switch (status) {
    case 'success':
      return {
        label: 'Success',
        color: THEME.colors.success,
        backgroundColor: THEME.colors.successBg,
      };
    case 'pending':
      return {
        label: 'Process',
        color: THEME.colors.warning,
        backgroundColor: THEME.colors.warningBg,
      };
    case 'failed':
      return {
        label: 'Reject',
        color: THEME.colors.error,
        backgroundColor: THEME.colors.errorBg,
      };
    default:
      return {
        label: 'Unknown',
        color: THEME.colors.textMuted,
        backgroundColor: THEME.colors.lightGrey,
      };
  }
};

const getAmountPrefix = (type: TransactionType): string => {
  switch (type) {
    case 'deposit':
    case 'receive':
      return '+';
    case 'withdrawal':
    case 'send':
      return '-';
    default:
      return '';
  }
};

export const ActivityCard: React.FC<ActivityCardProps> = ({
  type,
  transactionId,
  amount,
  currency = 'CHML',
  status,
  date,
  account,
  onPress,
}) => {
  const statusConfig = getStatusConfig(status);
  const amountPrefix = getAmountPrefix(type);
  const isPositive = amountPrefix === '+';

  return (
    <TouchableOpacity
      style={styles.container}
      onPress={onPress}
      activeOpacity={0.7}
      disabled={!onPress}
    >
      {/* Left Side - Icon and Info */}
      <View style={styles.leftSection}>
        <View style={[
          styles.iconContainer,
          { backgroundColor: isPositive ? THEME.colors.successBg : THEME.colors.errorBg }
        ]}>
          <Ionicons
            name={getTransactionIcon(type) as any}
            size={20}
            color={isPositive ? THEME.colors.success : THEME.colors.error}
          />
        </View>
        
        <View style={styles.infoSection}>
          <Text style={styles.transactionType}>
            {getTransactionLabel(type)}
          </Text>
          <Text style={styles.transactionId}>
            ID: {truncateAddress(transactionId, 4, 4)}
          </Text>
          {date && (
            <Text style={styles.date}>{date}</Text>
          )}
          {account && (
            <Text style={styles.account}>
              {truncateAddress(account, 6, 4)}
            </Text>
          )}
        </View>
      </View>

      {/* Right Side - Amount and Status */}
      <View style={styles.rightSection}>
        <Text style={[
          styles.amount,
          { color: isPositive ? THEME.colors.success : THEME.colors.error }
        ]}>
          {amountPrefix}{amount} {currency}
        </Text>
        
        <View style={[
          styles.statusBadge,
          { backgroundColor: statusConfig.backgroundColor }
        ]}>
          <Text style={[
            styles.statusText,
            { color: statusConfig.color }
          ]}>
            {statusConfig.label}
          </Text>
        </View>
      </View>
    </TouchableOpacity>
  );
};

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: THEME.colors.white,
    borderRadius: THEME.borderRadius.medium,
    padding: THEME.spacing.md,
    marginHorizontal: THEME.spacing.md,
    marginVertical: THEME.spacing.xs,
    ...THEME.shadows.small,
  },
  leftSection: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
  },
  iconContainer: {
    width: 40,
    height: 40,
    borderRadius: 20,
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: THEME.spacing.sm,
  },
  infoSection: {
    flex: 1,
  },
  transactionType: {
    fontSize: THEME.fontSize.base,
    fontWeight: THEME.fontWeight.semibold,
    color: THEME.colors.text,
    marginBottom: 2,
  },
  transactionId: {
    fontSize: THEME.fontSize.sm,
    color: THEME.colors.textSecondary,
    marginBottom: 2,
  },
  date: {
    fontSize: THEME.fontSize.xs,
    color: THEME.colors.textMuted,
  },
  account: {
    fontSize: THEME.fontSize.xs,
    color: THEME.colors.textMuted,
  },
  rightSection: {
    alignItems: 'flex-end',
  },
  amount: {
    fontSize: THEME.fontSize.base,
    fontWeight: THEME.fontWeight.semibold,
    marginBottom: THEME.spacing.xs,
  },
  statusBadge: {
    paddingHorizontal: THEME.spacing.sm,
    paddingVertical: THEME.spacing.xs,
    borderRadius: THEME.borderRadius.small,
  },
  statusText: {
    fontSize: THEME.fontSize.xs,
    fontWeight: THEME.fontWeight.medium,
  },
});

export default ActivityCard;
