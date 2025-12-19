/**
 * Receive CHML screen for Chameleon Network
 * Updated with new light theme design
 */

import React from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  ScrollView,
  Alert,
  Share,
  StyleSheet,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useRouter } from 'expo-router';
import * as Clipboard from 'expo-clipboard';
import { Ionicons } from '@expo/vector-icons';
import { useWallet } from '../context/WalletContext';
import { QRCode } from '../components/QRCode';
import { ScreenContainer } from '../components/ScreenContainer';
import { truncateAddress } from '../utils/address';
import { THEME } from '../constants/theme';

const ReceiveScreen = () => {
  const router = useRouter();
  const { wallet } = useWallet();

  if (!wallet?.address) {
    return (
      <ScreenContainer showGradient={false}>
        <View style={styles.noWalletContainer}>
          <Ionicons name="wallet-outline" size={64} color={THEME.colors.textMuted} />
          <Text style={styles.noWalletTitle}>No Wallet Found</Text>
          <Text style={styles.noWalletSubtitle}>
            Please create or import a wallet first
          </Text>
          <TouchableOpacity
            style={styles.goBackButton}
            onPress={() => router.back()}
          >
            <Text style={styles.goBackButtonText}>Go Back</Text>
          </TouchableOpacity>
        </View>
      </ScreenContainer>
    );
  }

  const handleCopyAddress = async () => {
    try {
      await Clipboard.setStringAsync(wallet.address);
      Alert.alert('Copied!', 'Address copied to clipboard');
    } catch (error) {
      Alert.alert('Error', 'Failed to copy address');
    }
  };

  const handleShareAddress = async () => {
    try {
      await Share.share({
        message: `My Chameleon Network address: ${wallet.address}`,
        title: 'Chameleon Wallet Address',
      });
    } catch (error) {
      console.error('Error sharing address:', error);
    }
  };

  return (
    <ScrollView className="flex-1 bg-[#0C0E12]" contentContainerStyle={{ flexGrow: 1 }}>
      <SafeAreaView className="flex-1 px-6">
        {/* Header */}
        <View className="flex-row items-center mb-8">
          <TouchableOpacity
            onPress={() => router.back()}
            className="mr-4"
          >
            <Ionicons name="arrow-back" size={24} color="#FFFFFF" />
          </TouchableOpacity>
          <Text className="text-white text-xl font-bold">Receive CHML</Text>
        </View>

        {/* QR Code Section */}
        <View className="items-center mb-8">
          <View className="mb-6">
            <QRCode
              value={wallet.address}
              size={240}
              backgroundColor="#FFFFFF"
              color="#18BB59"
            />
          </View>
          
          <Text className="text-[#CDCDE0] text-sm mb-2 text-center">
            Scan this QR code to get my address
          </Text>
        </View>

        {/* Address Section */}
        <View className="mb-8">
          <Text className="text-white text-lg font-semibold mb-4 text-center">
            Your CHML Address
          </Text>
          
          <View className="bg-[#1B1B1B] rounded-xl p-4 mb-4">
            <Text className="text-white font-mono text-sm text-center leading-6">
              {wallet.address}
            </Text>
          </View>
          
          {/* Action Buttons */}
          <View className="flex-row gap-4">
            <TouchableOpacity
              className="flex-1 bg-[#18BB59] rounded-xl py-4 px-6"
              onPress={handleCopyAddress}
            >
              <View className="flex-row items-center justify-center">
                <Ionicons name="copy-outline" size={20} color="#FFFFFF" />
                <Text className="text-white font-semibold ml-2">Copy</Text>
              </View>
            </TouchableOpacity>
            
            <TouchableOpacity
              className="flex-1 border border-[#18BB59] rounded-xl py-4 px-6"
              onPress={handleShareAddress}
            >
              <View className="flex-row items-center justify-center">
                <Ionicons name="share-outline" size={20} color="#18BB59" />
                <Text className="text-[#18BB59] font-semibold ml-2">Share</Text>
              </View>
            </TouchableOpacity>
          </View>
        </View>

        {/* Warning Section */}
        <View className="bg-[#2A1B1B] border border-[#FF4444] rounded-xl p-4 mb-6">
          <View className="flex-row items-start">
            <Ionicons name="warning" size={20} color="#FF4444" />
            <View className="flex-1 ml-3">
              <Text className="text-[#FF4444] font-semibold mb-2">
                ⚠️ DEVNET ADDRESS
              </Text>
              <Text className="text-[#CDCDE0] text-sm leading-5">
                This is a development network address. Only send test CHML tokens to this address.
              </Text>
            </View>
          </View>
        </View>

        {/* Tips Section */}
        <View className="bg-[#1B1B1B] rounded-xl p-4">
          <Text className="text-white font-semibold mb-3">💡 Tips</Text>
          
          <View className="space-y-3">
            <View className="flex-row items-start">
              <Text className="text-[#13E1BC] mr-2">•</Text>
              <Text className="text-[#CDCDE0] text-sm flex-1">
                Only send CHML tokens to this address
              </Text>
            </View>
            
            <View className="flex-row items-start">
              <Text className="text-[#13E1BC] mr-2">•</Text>
              <Text className="text-[#CDCDE0] text-sm flex-1">
                Double-check the address before sharing
              </Text>
            </View>
            
            <View className="flex-row items-start">
              <Text className="text-[#13E1BC] mr-2">•</Text>
              <Text className="text-[#CDCDE0] text-sm flex-1">
                Transactions on devnet are for testing only
              </Text>
            </View>
            
            <View className="flex-row items-start">
              <Text className="text-[#13E1BC] mr-2">•</Text>
              <Text className="text-[#CDCDE0] text-sm flex-1">
                Your address is public and safe to share
              </Text>
            </View>
          </View>
        </View>

        {/* Spacer */}
        <View className="flex-1" />

        {/* Additional Info */}
        <View className="items-center py-4">
          <Text className="text-[#666] text-xs text-center">
            Powered by Chameleon Network
          </Text>
        </View>
      </SafeAreaView>
    </ScrollView>
  );
};

export default ReceiveScreen;
