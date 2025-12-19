/**
 * AccountCard component with frosted glass effect for balance display
 */

import React, { useState } from 'react';
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';
import { BlurView } from 'expo-blur';
import { Ionicons } from '@expo/vector-icons';
import { THEME } from '../constants/theme';

interface AccountCardProps {
  balance: string;
  currency?: string;
  usdValue?: string;
  growthPercentage?: string;
  isPositiveGrowth?: boolean;
  onCurrencyPress?: () => void;
}

const CURRENCIES = ['USD', 'EUR', 'BTC', 'ETH'];

export const AccountCard: React.FC<AccountCardProps> = ({
  balance,
  currency = 'USD',
  usdValue,
  growthPercentage = '+3.75%',
  isPositiveGrowth = true,
  onCurrencyPress,
}) => {
  const [selectedCurrency, setSelectedCurrency] = useState(currency);

  const handleCurrencyPress = () => {
    if (onCurrencyPress) {
      onCurrencyPress();
    } else {
      // Cycle through currencies as default behavior
      const currentIndex = CURRENCIES.indexOf(selectedCurrency);
      const nextIndex = (currentIndex + 1) % CURRENCIES.length;
      setSelectedCurrency(CURRENCIES[nextIndex]);
    }
  };

  return (
    <View style={styles.container}>
      <BlurView intensity={20} tint="light" style={styles.blurContainer}>
        <View style={styles.cardContent}>
          {/* Header */}
          <View style={styles.header}>
            <Text style={styles.accountTitle}>My Account</Text>
            <TouchableOpacity
              style={styles.currencySelector}
              onPress={handleCurrencyPress}
              activeOpacity={0.7}
            >
              <Text style={styles.currencyText}>{selectedCurrency}</Text>
              <Ionicons
                name="chevron-down"
                size={16}
                color={THEME.colors.textSecondary}
              />
            </TouchableOpacity>
          </View>

          {/* Balance */}
          <View style={styles.balanceSection}>
            <Text style={styles.balanceAmount}>{balance}</Text>
            <Text style={styles.balanceCurrency}>CHML</Text>
          </View>

          {/* Growth Indicator */}
          <View style={styles.growthSection}>
            <View style={[
              styles.growthBadge,
              { backgroundColor: isPositiveGrowth ? THEME.colors.successBg : THEME.colors.errorBg }
            ]}>
              <Text style={[
                styles.growthText,
                { color: isPositiveGrowth ? THEME.colors.success : THEME.colors.error }
              ]}>
                {growthPercentage}
              </Text>
            </View>
            <Text style={styles.growthPeriod}>This week</Text>
          </View>

          {/* USD Value (if provided) */}
          {usdValue && (
            <Text style={styles.usdValue}>≈ ${usdValue} USD</Text>
          )}
        </View>
      </BlurView>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    marginHorizontal: THEME.spacing.md,
    marginVertical: THEME.spacing.sm,
  },
  blurContainer: {
    borderRadius: THEME.borderRadius.large,
    overflow: 'hidden',
    ...THEME.shadows.medium,
  },
  cardContent: {
    padding: THEME.spacing.lg,
    backgroundColor: 'rgba(255, 255, 255, 0.1)', // Additional frosted effect
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: THEME.spacing.md,
  },
  accountTitle: {
    fontSize: THEME.fontSize.base,
    fontWeight: THEME.fontWeight.medium,
    color: THEME.colors.text,
  },
  currencySelector: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: 'rgba(255, 255, 255, 0.3)',
    paddingHorizontal: THEME.spacing.sm,
    paddingVertical: THEME.spacing.xs,
    borderRadius: THEME.borderRadius.small,
    gap: THEME.spacing.xs,
  },
  currencyText: {
    fontSize: THEME.fontSize.sm,
    fontWeight: THEME.fontWeight.medium,
    color: THEME.colors.text,
  },
  balanceSection: {
    flexDirection: 'row',
    alignItems: 'baseline',
    marginBottom: THEME.spacing.sm,
    gap: THEME.spacing.xs,
  },
  balanceAmount: {
    fontSize: THEME.fontSize['3xl'],
    fontWeight: THEME.fontWeight.bold,
    color: THEME.colors.text,
  },
  balanceCurrency: {
    fontSize: THEME.fontSize.lg,
    fontWeight: THEME.fontWeight.medium,
    color: THEME.colors.textSecondary,
  },
  growthSection: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: THEME.spacing.sm,
  },
  growthBadge: {
    paddingHorizontal: THEME.spacing.sm,
    paddingVertical: THEME.spacing.xs,
    borderRadius: THEME.borderRadius.small,
  },
  growthText: {
    fontSize: THEME.fontSize.sm,
    fontWeight: THEME.fontWeight.semibold,
  },
  growthPeriod: {
    fontSize: THEME.fontSize.sm,
    color: THEME.colors.textSecondary,
  },
  usdValue: {
    fontSize: THEME.fontSize.sm,
    color: THEME.colors.textMuted,
    marginTop: THEME.spacing.xs,
  },
});

export default AccountCard;
