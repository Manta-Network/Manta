/**
 * Chameleon Wallet - Balance Display Component
 */

import React, { useState } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  ActivityIndicator,
} from 'react-native';
import { formatCHML, formatUSD } from '@/utils/format';
import { colors, fonts } from '@/theme';
import { CHAMELEON } from '@/constants';
import type { Balance as BalanceType } from '@/types';

interface BalanceProps {
  balance: BalanceType | null;
  isLoading?: boolean;
  showUSD?: boolean;
  onRefresh?: () => void;
}

export function Balance({
  balance,
  isLoading = false,
  showUSD = true,
  onRefresh,
}: BalanceProps) {
  const [showPrivate, setShowPrivate] = useState(false);

  const publicBalance = balance ? formatCHML(balance.public) : '0';
  const shieldedBalance = balance ? formatCHML(balance.shielded) : '0';
  const usdValue = balance ? formatUSD(balance.usdValue) : '$0.00';

  const totalBalance = balance
    ? formatCHML(
        (BigInt(balance.public) + BigInt(balance.shielded)).toString()
      )
    : '0';

  const handleTogglePrivacy = () => {
    setShowPrivate(!showPrivate);
  };

  if (isLoading) {
    return (
      <View style={styles.container}>
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={colors.primary} />
          <Text style={styles.loadingText}>Loading balance...</Text>
        </View>
      </View>
    );
  }

  return (
    <TouchableOpacity
      style={styles.container}
      onPress={onRefresh}
      activeOpacity={0.8}
    >
      <View style={styles.header}>
        <Text style={styles.label}>Total Balance</Text>
        <TouchableOpacity
          style={styles.privacyToggle}
          onPress={handleTogglePrivacy}
        >
          <Text style={styles.privacyText}>
            {showPrivate ? '👁️' : '🙈'}
          </Text>
        </TouchableOpacity>
      </View>

      <View style={styles.balanceContainer}>
        <Text style={styles.balance}>
          {showPrivate ? '••••••' : `${totalBalance} ${CHAMELEON.TOKEN_SYMBOL}`}
        </Text>
        {showUSD && (
          <Text style={styles.usdValue}>
            {showPrivate ? '••••••' : usdValue}
          </Text>
        )}
      </View>

      {!showPrivate && (
        <View style={styles.breakdown}>
          <View style={styles.breakdownItem}>
            <View style={styles.breakdownHeader}>
              <View style={[styles.indicator, styles.publicIndicator]} />
              <Text style={styles.breakdownLabel}>Public</Text>
            </View>
            <Text style={styles.breakdownValue}>
              {publicBalance} {CHAMELEON.TOKEN_SYMBOL}
            </Text>
          </View>

          <View style={styles.breakdownItem}>
            <View style={styles.breakdownHeader}>
              <View style={[styles.indicator, styles.privateIndicator]} />
              <Text style={styles.breakdownLabel}>Private</Text>
            </View>
            <Text style={styles.breakdownValue}>
              {shieldedBalance} {CHAMELEON.TOKEN_SYMBOL}
            </Text>
          </View>
        </View>
      )}

      {balance && (
        <Text style={styles.lastUpdated}>
          Last updated: {new Date(balance.lastUpdated).toLocaleTimeString()}
        </Text>
      )}
    </TouchableOpacity>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: colors.surface,
    borderRadius: 20,
    padding: 24,
    marginHorizontal: 16,
    marginVertical: 8,
  },
  
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 16,
  },
  
  label: {
    fontSize: 16,
    fontFamily: fonts.medium,
    color: colors.textSecondary,
  },
  
  privacyToggle: {
    padding: 8,
  },
  
  privacyText: {
    fontSize: 20,
  },
  
  balanceContainer: {
    alignItems: 'center',
    marginBottom: 24,
  },
  
  balance: {
    fontSize: 32,
    fontFamily: fonts.bold,
    color: colors.text,
    marginBottom: 8,
  },
  
  usdValue: {
    fontSize: 18,
    fontFamily: fonts.medium,
    color: colors.textSecondary,
  },
  
  breakdown: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 16,
  },
  
  breakdownItem: {
    flex: 1,
    alignItems: 'center',
  },
  
  breakdownHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 8,
  },
  
  indicator: {
    width: 8,
    height: 8,
    borderRadius: 4,
    marginRight: 8,
  },
  
  publicIndicator: {
    backgroundColor: colors.primary,
  },
  
  privateIndicator: {
    backgroundColor: colors.secondary,
  },
  
  breakdownLabel: {
    fontSize: 14,
    fontFamily: fonts.medium,
    color: colors.textSecondary,
  },
  
  breakdownValue: {
    fontSize: 16,
    fontFamily: fonts.medium,
    color: colors.text,
    textAlign: 'center',
  },
  
  lastUpdated: {
    fontSize: 12,
    fontFamily: fonts.regular,
    color: colors.textSecondary,
    textAlign: 'center',
  },
  
  loadingContainer: {
    alignItems: 'center',
    paddingVertical: 40,
  },
  
  loadingText: {
    fontSize: 16,
    fontFamily: fonts.medium,
    color: colors.textSecondary,
    marginTop: 16,
  },
});
