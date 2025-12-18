/**
 * Main Wallet screen for Chameleon Network
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
import { SafeAreaView } from 'react-native-safe-area-context';
import { useRouter } from 'expo-router';
import * as Clipboard from 'expo-clipboard';
import { Ionicons } from '@expo/vector-icons';
import { useWallet } from '../../context/WalletContext';
import { useBalance } from '../../hooks/useBalance';
import { useApi } from '../../hooks/useApi';
import { NetworkBadge } from '../../components/NetworkBadge';
import { truncateAddress } from '../../utils/address';

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
    <ScrollView className="flex-1 bg-[#0C0E12]" contentContainerStyle={{ flexGrow: 1 }}>
      <SafeAreaView className="flex-1 px-6">
        {/* Network Badge */}
        <View className="mb-8">
          <NetworkBadge />
        </View>

        {/* Header */}
        <View className="items-center mb-12">
          <Text className="text-6xl mb-4">🦎</Text>
          <Text className="text-white text-2xl font-bold mb-2">Chameleon Wallet</Text>
          <Text className="text-[#CDCDE0] text-center text-base leading-6">
            Create a new wallet or{"\n"}import an existing one
          </Text>
        </View>

        {/* Main Actions */}
        <View className="mb-8">
          <TouchableOpacity
            className="bg-[#18BB59] rounded-xl py-4 px-6 mb-4"
            onPress={() => router.push('/create-wallet')}
          >
            <Text className="text-white text-center text-lg font-semibold">
              Create New Wallet
            </Text>
          </TouchableOpacity>

          <TouchableOpacity
            className="border border-[#18BB59] rounded-xl py-4 px-6"
            onPress={() => router.push('/import-wallet')}
          >
            <Text className="text-[#18BB59] text-center text-lg font-semibold">
              Import Wallet
            </Text>
          </TouchableOpacity>
        </View>

        {/* Dev Accounts Section */}
        <View className="items-center">
          <Text className="text-[#CDCDE0] text-sm mb-4">── Or use dev account ──</Text>
          
          <View className="flex-row flex-wrap justify-center gap-3">
            {(['alice', 'bob', 'charlie', 'dave', 'eve'] as const).map((account) => (
              <TouchableOpacity
                key={account}
                className="bg-[#1B1B1B] rounded-lg py-2 px-4 border border-[#333]"
                onPress={() => handleDevAccountImport(account)}
              >
                <Text className="text-[#13E1BC] text-sm font-medium capitalize">
                  {account}
                </Text>
              </TouchableOpacity>
            ))}
          </View>
        </View>
      </SafeAreaView>
    </ScrollView>
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
            onPress={() => Alert.alert('Coming Soon', 'Send functionality will be available in Phase 3')}
          >
            <Text className="text-white text-center text-lg font-semibold">
              Send
            </Text>
          </TouchableOpacity>

          <TouchableOpacity
            className="flex-1 border border-[#18BB59] rounded-xl py-4 px-6"
            onPress={() => Alert.alert('Coming Soon', 'Receive functionality will be available in Phase 3')}
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
