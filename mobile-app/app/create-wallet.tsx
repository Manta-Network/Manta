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

const styles = StyleSheet.create({
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: THEME.spacing.md,
    paddingVertical: THEME.spacing.sm,
    borderBottomWidth: 1,
    borderBottomColor: THEME.colors.border,
  },
  backButton: {
    marginRight: THEME.spacing.md,
    padding: THEME.spacing.xs,
  },
  headerTitle: {
    fontSize: THEME.fontSize.xl,
    fontWeight: THEME.fontWeight.bold,
    color: THEME.colors.text,
  },
  content: {
    flex: 1,
    padding: THEME.spacing.md,
  },
  initialContainer: {
    flex: 1,
    justifyContent: 'center',
  },
  initialContent: {
    alignItems: 'center',
    marginBottom: THEME.spacing.xl * 2,
  },
  iconContainer: {
    backgroundColor: THEME.colors.primary,
    borderRadius: THEME.borderRadius.full,
    padding: THEME.spacing.lg,
    marginBottom: THEME.spacing.lg,
  },
  title: {
    fontSize: THEME.fontSize['2xl'],
    fontWeight: THEME.fontWeight.bold,
    color: THEME.colors.text,
    marginBottom: THEME.spacing.md,
    textAlign: 'center',
  },
  subtitle: {
    color: THEME.colors.textSecondary,
    textAlign: 'center',
    fontSize: THEME.fontSize.base,
    lineHeight: 24,
  },
  generateButton: {
    backgroundColor: THEME.colors.primary,
    borderRadius: THEME.borderRadius.medium,
    paddingVertical: THEME.spacing.md,
    paddingHorizontal: THEME.spacing.lg,
  },
  generateButtonText: {
    color: THEME.colors.white,
    textAlign: 'center',
    fontSize: THEME.fontSize.lg,
    fontWeight: THEME.fontWeight.semibold,
  },
  mnemonicSection: {
    flex: 1,
  },
  warningBox: {
    backgroundColor: THEME.colors.warningBg,
    borderWidth: 1,
    borderColor: THEME.colors.warning,
    borderRadius: THEME.borderRadius.medium,
    padding: THEME.spacing.md,
    marginBottom: THEME.spacing.lg,
  },
  warningHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: THEME.spacing.xs,
  },
  warningTitle: {
    color: THEME.colors.warning,
    fontWeight: THEME.fontWeight.semibold,
    marginLeft: THEME.spacing.xs,
  },
  warningText: {
    color: THEME.colors.warning,
    fontSize: THEME.fontSize.sm,
    lineHeight: 18,
  },
  seedTitle: {
    fontSize: THEME.fontSize.lg,
    fontWeight: THEME.fontWeight.semibold,
    color: THEME.colors.text,
    marginBottom: THEME.spacing.md,
  },
  mnemonicContainer: {
    backgroundColor: THEME.colors.white,
    borderRadius: THEME.borderRadius.medium,
    padding: THEME.spacing.md,
    marginBottom: THEME.spacing.lg,
    ...THEME.shadows.small,
  },
  mnemonicGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
  },
  wordContainer: {
    width: '33.333%',
    padding: THEME.spacing.xs,
  },
  wordCard: {
    backgroundColor: THEME.colors.card,
    borderRadius: THEME.borderRadius.small,
    padding: THEME.spacing.sm,
    borderWidth: 1,
    borderColor: THEME.colors.border,
    alignItems: 'center',
  },
  wordNumber: {
    color: THEME.colors.textSecondary,
    fontSize: THEME.fontSize.xs,
    marginBottom: 2,
  },
  wordText: {
    color: THEME.colors.text,
    fontSize: THEME.fontSize.sm,
    fontFamily: 'monospace',
    textAlign: 'center',
  },
  copyButton: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: THEME.colors.white,
    borderRadius: THEME.borderRadius.medium,
    paddingVertical: THEME.spacing.sm,
    paddingHorizontal: THEME.spacing.md,
    marginBottom: THEME.spacing.lg,
    borderWidth: 1,
    borderColor: THEME.colors.border,
    ...THEME.shadows.small,
  },
  copyButtonText: {
    color: THEME.colors.secondary,
    fontWeight: THEME.fontWeight.semibold,
    marginLeft: THEME.spacing.xs,
  },
  confirmationRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: THEME.spacing.xl,
  },
  checkbox: {
    width: 24,
    height: 24,
    borderRadius: 4,
    borderWidth: 2,
    marginRight: THEME.spacing.sm,
    alignItems: 'center',
    justifyContent: 'center',
  },
  checkboxChecked: {
    backgroundColor: THEME.colors.primary,
    borderColor: THEME.colors.primary,
  },
  checkboxUnchecked: {
    borderColor: THEME.colors.textSecondary,
  },
  confirmationText: {
    color: THEME.colors.textSecondary,
    flex: 1,
    fontSize: THEME.fontSize.sm,
    lineHeight: 18,
  },
  createButton: {
    borderRadius: THEME.borderRadius.medium,
    paddingVertical: THEME.spacing.md,
    paddingHorizontal: THEME.spacing.lg,
  },
  createButtonEnabled: {
    backgroundColor: THEME.colors.primary,
  },
  createButtonDisabled: {
    backgroundColor: THEME.colors.lightGrey,
    opacity: 0.5,
  },
  createButtonText: {
    color: THEME.colors.white,
    textAlign: 'center',
    fontSize: THEME.fontSize.lg,
    fontWeight: THEME.fontWeight.semibold,
  },
});

export default CreateWalletScreen;
