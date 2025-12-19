/**
 * Create new wallet screen
 * Updated with new light theme design
 */

import React, { useState } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  ScrollView,
  Alert,
  ActivityIndicator,
  StyleSheet,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import * as Clipboard from 'expo-clipboard';
import { useWallet } from '../context/WalletContext';
import { walletService } from '../services/wallet';
import { ScreenContainer } from '../components/ScreenContainer';
import { THEME } from '../constants/theme';

const CreateWalletScreen = () => {
  const router = useRouter();
  const { createWallet, isLoading } = useWallet();
  const [mnemonic, setMnemonic] = useState<string>('');
  const [isGenerated, setIsGenerated] = useState(false);
  const [isConfirmed, setIsConfirmed] = useState(false);

  const generateMnemonic = () => {
    const newMnemonic = walletService.generateMnemonic();
    setMnemonic(newMnemonic);
    setIsGenerated(true);
  };

  const handleCopyMnemonic = async () => {
    if (mnemonic) {
      await Clipboard.setStringAsync(mnemonic);
      Alert.alert('Copied!', 'Seed phrase copied to clipboard');
    }
  };

  const handleCreateWallet = async () => {
    if (!isConfirmed) {
      Alert.alert(
        'Confirmation Required',
        'Please confirm that you have written down your seed phrase before proceeding.'
      );
      return;
    }

    try {
      await createWallet(mnemonic, 'My Wallet');
      Alert.alert(
        'Wallet Created!',
        'Your wallet has been created successfully.',
        [
          {
            text: 'OK',
            onPress: () => router.replace('/(tabs)/wallet'),
          },
        ]
      );
    } catch (error) {
      Alert.alert('Error', 'Failed to create wallet. Please try again.');
    }
  };

  const renderMnemonicWords = () => {
    const words = mnemonic.split(' ');
    return (
      <View style={styles.mnemonicContainer}>
        <View style={styles.mnemonicGrid}>
          {words.map((word, index) => (
            <View key={index} style={styles.wordContainer}>
              <View style={styles.wordCard}>
                <Text style={styles.wordNumber}>
                  {index + 1}
                </Text>
                <Text style={styles.wordText}>
                  {word}
                </Text>
              </View>
            </View>
          ))}
        </View>
      </View>
    );
  };

  return (
    <ScreenContainer scrollable showGradient={false}>
      <SafeAreaView style={{ flex: 1 }}>
        {/* Header */}
        <View style={styles.header}>
          <TouchableOpacity
            onPress={() => router.back()}
            style={styles.backButton}
          >
            <Ionicons name="arrow-back" size={24} color={THEME.colors.text} />
          </TouchableOpacity>
          <Text style={styles.headerTitle}>Create New Wallet</Text>
        </View>

        <View style={styles.content}>
          {!isGenerated ? (
            /* Initial State */
            <View style={styles.initialContainer}>
              <View style={styles.initialContent}>
                <View style={styles.iconContainer}>
                  <Ionicons name="wallet" size={48} color={THEME.colors.white} />
                </View>
                <Text style={styles.title}>
                  Create Your Wallet
                </Text>
                <Text style={styles.subtitle}>
                  We'll generate a secure 12-word seed phrase{"\n"}
                  that you can use to recover your wallet.
                </Text>
            </View>

            <TouchableOpacity
              className="bg-[#18BB59] rounded-xl py-4 px-6"
              onPress={generateMnemonic}
            >
              <Text className="text-white text-center text-lg font-semibold">
                Generate Seed Phrase
              </Text>
            </TouchableOpacity>
          </View>
        ) : (
          /* Mnemonic Display */
          <View className="flex-1">
            {/* Warning */}
            <View className="bg-[#FFB800]/10 border border-[#FFB800] rounded-xl p-4 mb-6">
              <View className="flex-row items-center mb-2">
                <Ionicons name="warning" size={20} color="#FFB800" />
                <Text className="text-[#FFB800] font-semibold ml-2">Important!</Text>
              </View>
              <Text className="text-[#FFB800] text-sm leading-5">
                Write down these words in order and store them safely. 
                This is the only way to recover your wallet if you lose access.
              </Text>
            </View>

            {/* Seed Phrase */}
            <Text className="text-white text-lg font-semibold mb-4">
              Your Seed Phrase
            </Text>
            {renderMnemonicWords()}

            {/* Copy Button */}
            <TouchableOpacity
              className="flex-row items-center justify-center bg-[#1B1B1B] rounded-xl py-3 px-4 mb-6"
              onPress={handleCopyMnemonic}
            >
              <Ionicons name="copy-outline" size={20} color="#13E1BC" />
              <Text className="text-[#13E1BC] font-semibold ml-2">
                Copy to Clipboard
              </Text>
            </TouchableOpacity>

            {/* Confirmation */}
            <TouchableOpacity
              className={`flex-row items-center mb-8 ${
                isConfirmed ? 'opacity-100' : 'opacity-70'
              }`}
              onPress={() => setIsConfirmed(!isConfirmed)}
            >
              <View
                className={`w-6 h-6 rounded border-2 mr-3 items-center justify-center ${
                  isConfirmed
                    ? 'bg-[#18BB59] border-[#18BB59]'
                    : 'border-[#CDCDE0]'
                }`}
              >
                {isConfirmed && (
                  <Ionicons name="checkmark" size={16} color="#FFFFFF" />
                )}
              </View>
              <Text className="text-[#CDCDE0] flex-1 text-sm">
                I have written down my seed phrase and understand that I cannot recover my wallet without it.
              </Text>
            </TouchableOpacity>

            {/* Create Button */}
            <TouchableOpacity
              className={`rounded-xl py-4 px-6 ${
                isConfirmed && !isLoading
                  ? 'bg-[#18BB59]'
                  : 'bg-[#333] opacity-50'
              }`}
              onPress={handleCreateWallet}
              disabled={!isConfirmed || isLoading}
            >
              {isLoading ? (
                <ActivityIndicator color="#FFFFFF" />
              ) : (
                <Text className="text-white text-center text-lg font-semibold">
                  Create Wallet
                </Text>
              )}
            </TouchableOpacity>
          </View>
        )}
      </SafeAreaView>
    </ScrollView>
  );
};

export default CreateWalletScreen;
