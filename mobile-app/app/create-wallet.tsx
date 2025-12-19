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
                style={styles.generateButton}
                onPress={generateMnemonic}
              >
                <Text style={styles.generateButtonText}>
                  Generate Seed Phrase
                </Text>
              </TouchableOpacity>
            </View>
          ) : (
            /* Mnemonic Display */
            <View style={styles.mnemonicSection}>
              {/* Warning */}
              <View style={styles.warningBox}>
                <View style={styles.warningHeader}>
                  <Ionicons name="warning" size={20} color={THEME.colors.warning} />
                  <Text style={styles.warningTitle}>Important!</Text>
                </View>
                <Text style={styles.warningText}>
                  Write down these words in order and store them safely. 
                  This is the only way to recover your wallet if you lose access.
                </Text>
              </View>

              {/* Seed Phrase */}
              <Text style={styles.seedTitle}>
                Your Seed Phrase
              </Text>
              {renderMnemonicWords()}

              {/* Copy Button */}
              <TouchableOpacity
                style={styles.copyButton}
                onPress={handleCopyMnemonic}
              >
                <Ionicons name="copy-outline" size={20} color={THEME.colors.secondary} />
                <Text style={styles.copyButtonText}>
                  Copy to Clipboard
                </Text>
              </TouchableOpacity>

              {/* Confirmation */}
              <TouchableOpacity
                style={[styles.confirmationRow, { opacity: isConfirmed ? 1 : 0.7 }]}
                onPress={() => setIsConfirmed(!isConfirmed)}
              >
                <View
                  style={[
                    styles.checkbox,
                    isConfirmed ? styles.checkboxChecked : styles.checkboxUnchecked
                  ]}
                >
                  {isConfirmed && (
                    <Ionicons name="checkmark" size={16} color={THEME.colors.white} />
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
