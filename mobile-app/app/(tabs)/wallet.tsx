/**
 * Main Wallet screen for Chameleon Network
 * Updated with new light theme design system
 */

import React, { useEffect } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  ScrollView,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { useRouter } from 'expo-router';
import * as Clipboard from 'expo-clipboard';
import { Ionicons } from '@expo/vector-icons';
import { useWallet } from '../../context/WalletContext';
import { useBalance } from '../../hooks/useBalance';
import { useApi } from '../../hooks/useApi';
import { ScreenContainer } from '../../components/ScreenContainer';
import { MainHeader } from '../../components/MainHeader';
import { AccountCard } from '../../components/AccountCard';
import { ActionGrid } from '../../components/ActionGrid';
import { ActivityCard } from '../../components/ActivityCard';
import { NetworkBadge } from '../../components/NetworkBadge';
import { truncateAddress } from '../../utils/address';
import { THEME } from '../../constants/theme';

const WalletScreen = () => {
  const router = useRouter();
  const { wallet, isLoading: walletLoading, importDevAccount } = useWallet();
  const { connectionState, connect } = useApi();
  const { formattedFreeBalance, isLoading: balanceLoading, error: balanceError } = useBalance(
    wallet?.address
  );

  // Auto-connect to network on mount
  useEffect(() => {
    if (connectionState.status === 'disconnected') {
      connect();
    }
  }, [connectionState.status, connect]);

  const handleCopyAddress = async () => {
    if (wallet?.address) {
      await Clipboard.setStringAsync(wallet.address);
      Alert.alert('Copied!', 'Address copied to clipboard');
    }
  };

  const handleDevAccountImport = async (accountName: 'alice' | 'bob' | 'charlie' | 'dave' | 'eve') => {
    try {
      await importDevAccount(accountName);
    } catch (error) {
      Alert.alert('Error', 'Failed to import dev account');
    }
  };

  const renderNoWalletState = () => (
    <ScreenContainer scrollable showGradient>
      <View style={{ padding: THEME.spacing.lg }}>
        {/* Network Badge */}
        <View style={{ alignItems: 'flex-end', marginBottom: THEME.spacing.lg }}>
          <NetworkBadge />
        </View>

        {/* Header */}
        <View style={{ alignItems: 'center', marginBottom: THEME.spacing.xl * 2 }}>
          <Text style={{ fontSize: 64, marginBottom: THEME.spacing.md }}>🦎</Text>
          <Text style={{ 
            fontSize: THEME.fontSize['2xl'], 
            fontWeight: THEME.fontWeight.bold, 
            color: THEME.colors.text,
            marginBottom: THEME.spacing.sm 
          }}>
            Chameleon Wallet
          </Text>
          <Text style={{ 
            color: THEME.colors.textSecondary, 
            textAlign: 'center', 
            fontSize: THEME.fontSize.base,
            lineHeight: 24
          }}>
            Create a new wallet or{"\n"}import an existing one
          </Text>
        </View>

        {/* Main Actions */}
        <View style={{ marginBottom: THEME.spacing.xl }}>
          <TouchableOpacity
            style={{
              backgroundColor: THEME.colors.primary,
              borderRadius: THEME.borderRadius.medium,
              paddingVertical: THEME.spacing.md,
              paddingHorizontal: THEME.spacing.lg,
              marginBottom: THEME.spacing.md,
            }}
            onPress={() => router.push('/create-wallet')}
          >
            <Text style={{
              color: THEME.colors.white,
              textAlign: 'center',
              fontSize: THEME.fontSize.lg,
              fontWeight: THEME.fontWeight.semibold,
            }}>
              Create New Wallet
            </Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={{
              borderWidth: 2,
              borderColor: THEME.colors.primary,
              borderRadius: THEME.borderRadius.medium,
              paddingVertical: THEME.spacing.md,
              paddingHorizontal: THEME.spacing.lg,
            }}
            onPress={() => router.push('/import-wallet')}
          >
            <Text style={{
              color: THEME.colors.primary,
              textAlign: 'center',
              fontSize: THEME.fontSize.lg,
              fontWeight: THEME.fontWeight.semibold,
            }}>
              Import Wallet
            </Text>
          </TouchableOpacity>
        </View>

        {/* Dev Accounts Section */}
        <View style={{ alignItems: 'center' }}>
          <Text style={{ 
            color: THEME.colors.textSecondary, 
            fontSize: THEME.fontSize.sm, 
            marginBottom: THEME.spacing.md 
          }}>
            ── Or use dev account ──
          </Text>
          
          <View style={{ 
            flexDirection: 'row', 
            flexWrap: 'wrap', 
            justifyContent: 'center', 
            gap: THEME.spacing.sm 
          }}>
            {(['alice', 'bob', 'charlie', 'dave', 'eve'] as const).map((account) => (
              <TouchableOpacity
                key={account}
                style={{
                  backgroundColor: THEME.colors.card,
                  borderRadius: THEME.borderRadius.small,
                  paddingVertical: THEME.spacing.sm,
                  paddingHorizontal: THEME.spacing.md,
                  borderWidth: 1,
                  borderColor: THEME.colors.border,
                }}
                onPress={() => handleDevAccountImport(account)}
              >
                <Text style={{
                  color: THEME.colors.secondary,
                  fontSize: THEME.fontSize.sm,
                  fontWeight: THEME.fontWeight.medium,
                  textTransform: 'capitalize',
                }}>
                  {account}
                </Text>
              </TouchableOpacity>
            ))}
          </View>
        </View>
      </View>
    </ScreenContainer>
  );

  const renderWalletState = () => (
    <ScrollView className="flex-1 bg-[#0C0E12]" contentContainerStyle={{ flexGrow: 1 }}>
      <SafeAreaView className="flex-1 px-6">
        {/* Network Badge */}
        <View className="mb-6">
          <NetworkBadge />
        </View>

        {/* Wallet Address */}
        <View className="mb-8">
          <View className="flex-row items-center justify-between bg-[#1B1B1B] rounded-xl p-4">
            <Text className="text-white text-base font-mono flex-1 mr-3">
              {truncateAddress(wallet!.address)}
            </Text>
            <TouchableOpacity
              onPress={handleCopyAddress}
              className="p-2"
            >
              <Ionicons name="copy-outline" size={20} color="#CDCDE0" />
            </TouchableOpacity>
          </View>
        </View>

        {/* Balance Display */}
        <View className="items-center mb-12">
          {balanceLoading ? (
            <ActivityIndicator size="large" color="#18BB59" />
          ) : balanceError ? (
            <View className="items-center">
              <Text className="text-[#FF4444] text-lg mb-2">Connection Error</Text>
              <Text className="text-[#CDCDE0] text-sm text-center">
                {balanceError}
              </Text>
            </View>
          ) : (
            <View className="items-center">
              <Text className="text-white text-4xl font-bold mb-2">
                {formattedFreeBalance.split(' ')[0]}
              </Text>
              <Text className="text-[#CDCDE0] text-xl">
                {formattedFreeBalance.split(' ')[1] || 'CHML'}
              </Text>
            </View>
          )}
        </View>

        {/* Action Buttons */}
        <View className="flex-row gap-4 mb-8">
          <TouchableOpacity
            className="flex-1 bg-[#18BB59] rounded-xl py-4 px-6"
            onPress={() => router.push('/send')}
          >
            <Text className="text-white text-center text-lg font-semibold">
              Send
            </Text>
          </TouchableOpacity>

          <TouchableOpacity
            className="flex-1 border border-[#18BB59] rounded-xl py-4 px-6"
            onPress={() => router.push('/receive')}
          >
            <Text className="text-[#18BB59] text-center text-lg font-semibold">
              Receive
            </Text>
          </TouchableOpacity>
        </View>

        {/* Recent Activity */}
        <View className="flex-1">
          <Text className="text-white text-lg font-semibold mb-4">Recent Activity</Text>
          <View className="bg-[#1B1B1B] rounded-xl p-6 items-center">
            <Ionicons name="time-outline" size={48} color="#CDCDE0" />
            <Text className="text-[#CDCDE0] text-base mt-4">
              No transactions yet
            </Text>
            <Text className="text-[#666] text-sm mt-2 text-center">
              Your transaction history will appear here
            </Text>
          </View>
        </View>
      </SafeAreaView>
    </ScrollView>
  );

  if (walletLoading) {
    return (
      <View className="flex-1 bg-[#0C0E12] justify-center items-center">
        <ActivityIndicator size="large" color="#18BB59" />
        <Text className="text-white text-lg mt-4">Loading wallet...</Text>
      </View>
    );
  }

  return wallet ? renderWalletState() : renderNoWalletState();
};

export default WalletScreen;
