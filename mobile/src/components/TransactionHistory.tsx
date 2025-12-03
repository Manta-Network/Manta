/**
 * Chameleon Wallet - Transaction History Component
 * Displays a list of transactions with filtering and pull-to-refresh
 */

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TouchableOpacity,
  RefreshControl,
  ActivityIndicator,
} from 'react-native';
import { Card } from './Card';
import { TransactionItem } from './TransactionItem';
import { useWalletContext } from '@/context/WalletContext';
import { colors, fonts, spacing, borderRadius } from '@/theme';
import type { Transaction, TransactionType } from '@/types';

interface TransactionHistoryProps {
  onTransactionPress?: (transaction: Transaction) => void;
  limit?: number;
  showFilters?: boolean;
  walletAddress?: string;
}

type FilterType = 'all' | TransactionType;

const FILTER_OPTIONS: { key: FilterType; label: string; icon: string }[] = [
  { key: 'all', label: 'All', icon: '📋' },
  { key: 'send', label: 'Sent', icon: '↗️' },
  { key: 'receive', label: 'Received', icon: '↙️' },
  { key: 'shield', label: 'Shielded', icon: '🛡️' },
  { key: 'unshield', label: 'Unshielded', icon: '🔓' },
  { key: 'stake', label: 'Staked', icon: '🏦' },
  { key: 'unstake', label: 'Unstaked', icon: '📤' },
];

export function TransactionHistory({
  onTransactionPress,
  limit,
  showFilters = true,
  walletAddress,
}: TransactionHistoryProps) {
  const { state, refreshTransactions } = useWalletContext();
  const { transactions, selectedWallet } = state;

  const [isRefreshing, setIsRefreshing] = useState(false);
  const [selectedFilter, setSelectedFilter] = useState<FilterType>('all');
  const [isLoading, setIsLoading] = useState(false);

  const currentWalletAddress = walletAddress || selectedWallet?.address || '';

  useEffect(() => {
    if (transactions.length === 0) {
      handleRefresh();
    }
  }, []);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      await refreshTransactions();
    } catch (error) {
      console.error('Failed to refresh transactions:', error);
    } finally {
      setIsRefreshing(false);
    }
  };

  const handleFilterChange = (filter: FilterType) => {
    setSelectedFilter(filter);
  };

  // Filter transactions based on selected filter
  const filteredTransactions = transactions.filter(transaction => {
    if (selectedFilter === 'all') return true;
    return transaction.type === selectedFilter;
  });

  // Apply limit if specified
  const displayTransactions = limit 
    ? filteredTransactions.slice(0, limit)
    : filteredTransactions;

  const renderTransactionItem = ({ item }: { item: Transaction }) => (
    <TransactionItem
      transaction={item}
      onPress={onTransactionPress ? () => onTransactionPress(item) : undefined}
      walletAddress={currentWalletAddress}
    />
  );

  const renderFilterButton = ({ key, label, icon }: typeof FILTER_OPTIONS[0]) => (
    <TouchableOpacity
      key={key}
      style={[
        styles.filterButton,
        selectedFilter === key && styles.filterButtonActive,
      ]}
      onPress={() => handleFilterChange(key)}
    >
      <Text style={styles.filterIcon}>{icon}</Text>
      <Text style={[
        styles.filterText,
        selectedFilter === key && styles.filterTextActive,
      ]}>
        {label}
      </Text>
    </TouchableOpacity>
  );

  const renderEmptyState = () => {
    const emptyMessages = {
      all: 'No transactions yet',
      send: 'No sent transactions',
      receive: 'No received transactions',
      shield: 'No shielded transactions',
      unshield: 'No unshielded transactions',
      stake: 'No staking transactions',
      unstake: 'No unstaking transactions',
      swap: 'No swap transactions',
    };

    const emptyDescriptions = {
      all: 'Your transaction history will appear here once you start using your wallet.',
      send: 'Transactions you send will appear here.',
      receive: 'Transactions you receive will appear here.',
      shield: 'When you shield tokens for privacy, they will appear here.',
      unshield: 'When you unshield tokens, they will appear here.',
      stake: 'When you stake tokens with validators, they will appear here.',
      unstake: 'When you unstake tokens, they will appear here.',
      swap: 'When you swap tokens on pDEX, they will appear here.',
    };

    return (
      <Card style={styles.emptyState}>
        <Text style={styles.emptyStateIcon}>📝</Text>
        <Text style={styles.emptyStateTitle}>
          {emptyMessages[selectedFilter]}
        </Text>
        <Text style={styles.emptyStateText}>
          {emptyDescriptions[selectedFilter]}
        </Text>
      </Card>
    );
  };

  const renderHeader = () => {
    if (!showFilters) return null;

    return (
      <View style={styles.header}>
        <Text style={styles.headerTitle}>Transaction History</Text>
        <View style={styles.filterContainer}>
          <FlatList
            data={FILTER_OPTIONS}
            renderItem={({ item }) => renderFilterButton(item)}
            keyExtractor={(item) => item.key}
            horizontal
            showsHorizontalScrollIndicator={false}
            contentContainerStyle={styles.filterList}
          />
        </View>
      </View>
    );
  };

  const renderFooter = () => {
    if (!isLoading) return null;

    return (
      <View style={styles.loadingFooter}>
        <ActivityIndicator size="small" color={colors.primary} />
        <Text style={styles.loadingText}>Loading more transactions...</Text>
      </View>
    );
  };

  return (
    <View style={styles.container}>
      {renderHeader()}
      
      <FlatList
        data={displayTransactions}
        renderItem={renderTransactionItem}
        keyExtractor={(item) => item.hash}
        refreshControl={
          <RefreshControl
            refreshing={isRefreshing}
            onRefresh={handleRefresh}
            tintColor={colors.primary}
            colors={[colors.primary]}
          />
        }
        ListEmptyComponent={renderEmptyState}
        ListFooterComponent={renderFooter}
        contentContainerStyle={[
          styles.listContent,
          displayTransactions.length === 0 && styles.listContentEmpty,
        ]}
        showsVerticalScrollIndicator={false}
        ItemSeparatorComponent={() => <View style={styles.separator} />}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  
  header: {
    paddingHorizontal: spacing.lg,
    paddingBottom: spacing.md,
  },
  
  headerTitle: {
    fontSize: fonts.lg,
    fontFamily: fonts.bold,
    color: colors.text,
    marginBottom: spacing.md,
  },
  
  filterContainer: {
    marginBottom: spacing.sm,
  },
  
  filterList: {
    paddingRight: spacing.lg,
  },
  
  filterButton: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: spacing.md,
    paddingVertical: spacing.sm,
    marginRight: spacing.sm,
    borderRadius: borderRadius.full,
    backgroundColor: colors.surface,
    borderWidth: 1,
    borderColor: colors.border,
  },
  
  filterButtonActive: {
    backgroundColor: colors.primary,
    borderColor: colors.primary,
  },
  
  filterIcon: {
    fontSize: 16,
    marginRight: spacing.xs,
  },
  
  filterText: {
    fontSize: fonts.sm,
    fontFamily: fonts.medium,
    color: colors.textSecondary,
  },
  
  filterTextActive: {
    color: colors.white,
  },
  
  listContent: {
    paddingHorizontal: spacing.lg,
  },
  
  listContentEmpty: {
    flex: 1,
    justifyContent: 'center',
  },
  
  separator: {
    height: spacing.sm,
  },
  
  emptyState: {
    alignItems: 'center',
    paddingVertical: spacing.xl,
  },
  
  emptyStateIcon: {
    fontSize: 48,
    marginBottom: spacing.md,
  },
  
  emptyStateTitle: {
    fontSize: fonts.lg,
    fontFamily: fonts.medium,
    color: colors.text,
    marginBottom: spacing.sm,
  },
  
  emptyStateText: {
    fontSize: fonts.sm,
    fontFamily: fonts.regular,
    color: colors.textSecondary,
    textAlign: 'center',
    lineHeight: 20,
    maxWidth: 280,
  },
  
  loadingFooter: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    paddingVertical: spacing.lg,
  },
  
  loadingText: {
    fontSize: fonts.sm,
    fontFamily: fonts.regular,
    color: colors.textSecondary,
    marginLeft: spacing.sm,
  },
});
