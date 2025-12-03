/**
 * Chameleon Wallet - Home Screen
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  SafeAreaView,
  ScrollView,
  RefreshControl,
  TouchableOpacity,
  StatusBar,
} from 'react-native';
import { Balance } from '@/components/Balance';
import { Card } from '@/components/Card';
import { TransactionItem } from '@/components/TransactionItem';
import { useWalletContext } from '@/context/WalletContext';
import { colors, fonts, spacing } from '@/theme';
import { formatCHML, truncateAddress } from '@/utils/format';
import { CHAMELEON } from '@/constants';
import type { NativeStackScreenProps } from '@react-navigation/native-stack';
import type { RootStackParamList } from '@/types';

type Props = NativeStackScreenProps<RootStackParamList, 'Main'>;

export function HomeScreen({ navigation }: Props) {
  const {
    state,
    refreshBalance,
    refreshTransactions,
  } = useWalletContext();
  
  const [isRefreshing, setIsRefreshing] = useState(false);

  const { selectedWallet, balance, transactions } = state;

  useEffect(() => {
    // Load initial data
    handleRefresh();
  }, []);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      await Promise.all([
        refreshBalance(),
        refreshTransactions(),
      ]);
    } catch (error) {
      console.error('Failed to refresh data:', error);
    } finally {
      setIsRefreshing(false);
    }
  };

  const handleSend = () => {
    navigation.navigate('Send', {});
  };

  const handleReceive = () => {
    navigation.navigate('Receive');
  };

  const handleShield = () => {
    // Navigate to shield screen or show shield modal
    console.log('Shield tokens');
  };

  const handleUnshield = () => {
    // Navigate to unshield screen or show unshield modal
    console.log('Unshield tokens');
  };

  const handleTransactionPress = (transaction: any) => {
    navigation.navigate('TransactionDetail', { transaction });
  };

  const recentTransactions = transactions.slice(0, 5);

  if (!selectedWallet) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.emptyState}>
          <Text style={styles.emptyStateText}>No wallet found</Text>
        </View>
      </SafeAreaView>
    );
  }

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="light-content" backgroundColor={colors.background} />
      
      <ScrollView
        style={styles.scrollView}
        contentContainerStyle={styles.scrollContent}
        refreshControl={
          <RefreshControl
            refreshing={isRefreshing}
            onRefresh={handleRefresh}
            tintColor={colors.primary}
            colors={[colors.primary]}
          />
        }
        showsVerticalScrollIndicator={false}
      >
        {/* Header */}
        <View style={styles.header}>
          <View>
            <Text style={styles.greeting}>Welcome back</Text>
            <Text style={styles.walletName}>{selectedWallet.name}</Text>
          </View>
          <TouchableOpacity style={styles.addressContainer}>
            <Text style={styles.address}>
              {truncateAddress(selectedWallet.address)}
            </Text>
          </TouchableOpacity>
        </View>

        {/* Balance Card */}
        <Balance
          balance={balance || null}
          isLoading={isRefreshing}
          onRefresh={handleRefresh}
        />

        {/* Quick Actions */}
        <View style={styles.quickActions}>
          <TouchableOpacity style={styles.actionButton} onPress={handleSend}>
            <View style={styles.actionIcon}>
              <Text style={styles.actionEmoji}>↗️</Text>
            </View>
            <Text style={styles.actionLabel}>Send</Text>
          </TouchableOpacity>

          <TouchableOpacity style={styles.actionButton} onPress={handleReceive}>
            <View style={styles.actionIcon}>
              <Text style={styles.actionEmoji}>↙️</Text>
            </View>
            <Text style={styles.actionLabel}>Receive</Text>
          </TouchableOpacity>

          <TouchableOpacity style={styles.actionButton} onPress={handleShield}>
            <View style={styles.actionIcon}>
              <Text style={styles.actionEmoji}>🛡️</Text>
            </View>
            <Text style={styles.actionLabel}>Shield</Text>
          </TouchableOpacity>

          <TouchableOpacity style={styles.actionButton} onPress={handleUnshield}>
            <View style={styles.actionIcon}>
              <Text style={styles.actionEmoji}>🔓</Text>
            </View>
            <Text style={styles.actionLabel}>Unshield</Text>
          </TouchableOpacity>
        </View>

        {/* Recent Transactions */}
        <View style={styles.section}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>Recent Activity</Text>
            {transactions.length > 5 && (
              <TouchableOpacity>
                <Text style={styles.seeAllText}>See All</Text>
              </TouchableOpacity>
            )}
          </View>

          {recentTransactions.length > 0 ? (
            <View style={styles.transactionsList}>
              {recentTransactions.map((transaction) => (
                <TransactionItem
                  key={transaction.hash}
                  transaction={transaction}
                  onPress={() => handleTransactionPress(transaction)}
                  walletAddress={selectedWallet.address}
                />
              ))}
            </View>
          ) : (
            <Card style={styles.emptyTransactions}>
              <Text style={styles.emptyTransactionsIcon}>📝</Text>
              <Text style={styles.emptyTransactionsTitle}>
                No transactions yet
              </Text>
              <Text style={styles.emptyTransactionsText}>
                Your transaction history will appear here once you start using your wallet.
              </Text>
            </Card>
          )}
        </View>

        {/* Network Status */}
        <Card style={styles.networkCard}>
          <View style={styles.networkHeader}>
            <Text style={styles.networkTitle}>Network Status</Text>
            <View style={styles.networkStatus}>
              <View style={styles.statusIndicator} />
              <Text style={styles.statusText}>Connected</Text>
            </View>
          </View>
          <Text style={styles.networkName}>Chameleon Testnet</Text>
        </Card>
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: colors.background,
  },
  
  scrollView: {
    flex: 1,
  },
  
  scrollContent: {
    paddingBottom: spacing.xl,
  },
  
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: spacing.lg,
    paddingTop: spacing.lg,
    paddingBottom: spacing.md,
  },
  
  greeting: {
    fontSize: fonts.sm,
    fontFamily: fonts.regular,
    color: colors.textSecondary,
  },
  
  walletName: {
    fontSize: fonts.xl,
    fontFamily: fonts.bold,
    color: colors.text,
  },
  
  addressContainer: {
    backgroundColor: colors.surface,
    paddingHorizontal: spacing.sm,
    paddingVertical: spacing.xs,
    borderRadius: 8,
  },
  
  address: {
    fontSize: fonts.xs,
    fontFamily: fonts.medium,
    color: colors.textSecondary,
  },
  
  quickActions: {
    flexDirection: 'row',
    justifyContent: 'space-around',
    paddingHorizontal: spacing.lg,
    paddingVertical: spacing.lg,
  },
  
  actionButton: {
    alignItems: 'center',
    flex: 1,
  },
  
  actionIcon: {
    width: 56,
    height: 56,
    borderRadius: 28,
    backgroundColor: colors.surface,
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: spacing.sm,
  },
  
  actionEmoji: {
    fontSize: 24,
  },
  
  actionLabel: {
    fontSize: fonts.sm,
    fontFamily: fonts.medium,
    color: colors.text,
  },
  
  section: {
    paddingHorizontal: spacing.lg,
    marginBottom: spacing.lg,
  },
  
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: spacing.md,
  },
  
  sectionTitle: {
    fontSize: fonts.lg,
    fontFamily: fonts.bold,
    color: colors.text,
  },
  
  seeAllText: {
    fontSize: fonts.sm,
    fontFamily: fonts.medium,
    color: colors.primary,
  },
  
  transactionsList: {
    gap: spacing.sm,
  },
  
  emptyTransactions: {
    alignItems: 'center',
    paddingVertical: spacing.xl,
  },
  
  emptyTransactionsIcon: {
    fontSize: 48,
    marginBottom: spacing.md,
  },
  
  emptyTransactionsTitle: {
    fontSize: fonts.lg,
    fontFamily: fonts.medium,
    color: colors.text,
    marginBottom: spacing.sm,
  },
  
  emptyTransactionsText: {
    fontSize: fonts.sm,
    fontFamily: fonts.regular,
    color: colors.textSecondary,
    textAlign: 'center',
    lineHeight: 20,
  },
  
  networkCard: {
    marginHorizontal: spacing.lg,
  },
  
  networkHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: spacing.xs,
  },
  
  networkTitle: {
    fontSize: fonts.base,
    fontFamily: fonts.medium,
    color: colors.text,
  },
  
  networkStatus: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  
  statusIndicator: {
    width: 8,
    height: 8,
    borderRadius: 4,
    backgroundColor: colors.success,
    marginRight: spacing.xs,
  },
  
  statusText: {
    fontSize: fonts.xs,
    fontFamily: fonts.medium,
    color: colors.success,
  },
  
  networkName: {
    fontSize: fonts.sm,
    fontFamily: fonts.regular,
    color: colors.textSecondary,
  },
  
  emptyState: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
  },
  
  emptyStateText: {
    fontSize: fonts.lg,
    fontFamily: fonts.medium,
    color: colors.textSecondary,
  },
});
