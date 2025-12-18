/**
 * Import existing wallet screen
 */

import React, { useState } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  ScrollView,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useWallet } from '../context/WalletContext';
import { walletService, DEV_ACCOUNTS } from '../services/wallet';

const ImportWalletScreen = () => {
  const router = useRouter();
  const { importWallet, importDevAccount, isLoading } = useWallet();
  const [mnemonic, setMnemonic] = useState('');
  const [selectedDevAccount, setSelectedDevAccount] = useState<keyof typeof DEV_ACCOUNTS | null>(null);
  const [addressPreview, setAddressPreview] = useState<string>('');
  const [isValidMnemonic, setIsValidMnemonic] = useState(false);

  const validateAndPreviewMnemonic = async (text: string) => {
    setMnemonic(text);
    
    if (!text.trim()) {
      setIsValidMnemonic(false);
      setAddressPreview('');
      return;
    }

    const isValid = walletService.validateMnemonic(text.trim());
    setIsValidMnemonic(isValid);

    if (isValid) {
      try {
        const address = await walletService.getAddressPreview(text.trim());
        setAddressPreview(address);
      } catch (error) {
        setAddressPreview('');
      }
    } else {
      setAddressPreview('');
    }
  };

  const handleDevAccountSelect = async (accountName: keyof typeof DEV_ACCOUNTS) => {
    setSelectedDevAccount(accountName);
    const devMnemonic = DEV_ACCOUNTS[accountName];
    setMnemonic(devMnemonic);
    setIsValidMnemonic(true);
    
    try {
      const address = await walletService.getAddressPreview(devMnemonic);
      setAddressPreview(address);
    } catch (error) {
      setAddressPreview('');
    }
  };

  const handleImportWallet = async () => {
    if (!isValidMnemonic) {
      Alert.alert('Invalid Seed Phrase', 'Please enter a valid 12 or 24 word seed phrase.');
      return;
    }

    try {
      if (selectedDevAccount) {
        await importDevAccount(selectedDevAccount);
      } else {
        await importWallet(mnemonic.trim(), 'Imported Wallet');
      }
      
      Alert.alert(
        'Wallet Imported!',
        'Your wallet has been imported successfully.',
        [
          {
            text: 'OK',
            onPress: () => router.replace('/(tabs)/wallet'),
          },
        ]
      );
    } catch (error) {
      Alert.alert('Error', 'Failed to import wallet. Please check your seed phrase and try again.');
    }
  };

  const formatAddressForPreview = (address: string) => {
    if (!address) return '';
    return `${address.slice(0, 6)}...${address.slice(-6)}`;
  };

  return (
    <ScrollView className="flex-1 bg-[#0C0E12]" contentContainerStyle={{ flexGrow: 1 }}>
      <SafeAreaView className="flex-1 px-6">
        {/* Header */}
        <View className="flex-row items-center mb-8">
          <TouchableOpacity
            onPress={() => router.back()}
            className="mr-4 p-2"
          >
            <Ionicons name="arrow-back" size={24} color="#FFFFFF" />
          </TouchableOpacity>
          <Text className="text-white text-xl font-bold">Import Wallet</Text>
        </View>

        {/* Dev Accounts Section */}
        <View className="mb-8">
          <Text className="text-white text-lg font-semibold mb-4">
            Quick Import (Dev Accounts)
          </Text>
          <View className="flex-row flex-wrap gap-3 mb-4">
            {(Object.keys(DEV_ACCOUNTS) as Array<keyof typeof DEV_ACCOUNTS>).map((account) => (
              <TouchableOpacity
                key={account}
                className={`rounded-lg py-3 px-4 border ${
                  selectedDevAccount === account
                    ? 'bg-[#18BB59] border-[#18BB59]'
                    : 'bg-[#1B1B1B] border-[#333]'
                }`}
                onPress={() => handleDevAccountSelect(account)}
              >
                <Text
                  className={`font-medium capitalize ${
                    selectedDevAccount === account
                      ? 'text-white'
                      : 'text-[#13E1BC]'
                  }`}
                >
                  {account}
                </Text>
              </TouchableOpacity>
            ))}
          </View>
          <Text className="text-[#CDCDE0] text-sm">
            These are pre-funded development accounts for testing.
          </Text>
        </View>

        {/* Divider */}
        <View className="flex-row items-center mb-8">
          <View className="flex-1 h-px bg-[#333]" />
          <Text className="text-[#CDCDE0] text-sm mx-4">Or enter seed phrase</Text>
          <View className="flex-1 h-px bg-[#333]" />
        </View>

        {/* Seed Phrase Input */}
        <View className="mb-6">
          <Text className="text-white text-lg font-semibold mb-4">
            Seed Phrase
          </Text>
          <View className="bg-[#1B1B1B] rounded-xl border border-[#333] p-4">
            <TextInput
              className="text-white text-base font-mono min-h-[120px]"
              placeholder="Enter your 12 or 24 word seed phrase..."
              placeholderTextColor="#666"
              multiline
              textAlignVertical="top"
              value={mnemonic}
              onChangeText={validateAndPreviewMnemonic}
              autoCapitalize="none"
              autoCorrect={false}
            />
          </View>
          
          {/* Validation Status */}
          {mnemonic.trim() && (
            <View className="flex-row items-center mt-3">
              <Ionicons
                name={isValidMnemonic ? 'checkmark-circle' : 'close-circle'}
                size={20}
                color={isValidMnemonic ? '#18BB59' : '#FF4444'}
              />
              <Text
                className={`ml-2 text-sm ${
                  isValidMnemonic ? 'text-[#18BB59]' : 'text-[#FF4444]'
                }`}
              >
                {isValidMnemonic ? 'Valid seed phrase' : 'Invalid seed phrase'}
              </Text>
            </View>
          )}
        </View>

        {/* Address Preview */}
        {addressPreview && (
          <View className="bg-[#1B1B1B] rounded-xl p-4 mb-8">
            <Text className="text-[#CDCDE0] text-sm mb-2">Wallet Address Preview:</Text>
            <Text className="text-white font-mono text-base">
              {formatAddressForPreview(addressPreview)}
            </Text>
          </View>
        )}

        {/* Import Button */}
        <TouchableOpacity
          className={`rounded-xl py-4 px-6 ${
            isValidMnemonic && !isLoading
              ? 'bg-[#18BB59]'
              : 'bg-[#333] opacity-50'
          }`}
          onPress={handleImportWallet}
          disabled={!isValidMnemonic || isLoading}
        >
          {isLoading ? (
            <ActivityIndicator color="#FFFFFF" />
          ) : (
            <Text className="text-white text-center text-lg font-semibold">
              Import Wallet
            </Text>
          )}
        </TouchableOpacity>

        {/* Security Note */}
        <View className="bg-[#FFB800]/10 border border-[#FFB800] rounded-xl p-4 mt-6">
          <View className="flex-row items-center mb-2">
            <Ionicons name="shield-checkmark" size={20} color="#FFB800" />
            <Text className="text-[#FFB800] font-semibold ml-2">Security Note</Text>
          </View>
          <Text className="text-[#FFB800] text-sm leading-5">
            Your seed phrase is stored securely on your device and never sent to our servers.
          </Text>
        </View>
      </SafeAreaView>
    </ScrollView>
  );
};

export default ImportWalletScreen;
