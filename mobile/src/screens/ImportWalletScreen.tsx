/**
 * Chameleon Wallet - Import Wallet Screen
 */

import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  SafeAreaView,
  ScrollView,
  TextInput,
  Alert,
  StatusBar,
} from 'react-native';
import { Button } from '@/components/Button';
import { Card } from '@/components/Card';
import { useWalletContext } from '@/context/WalletContext';
import { validateSeedPhrase } from '@/utils/crypto';
import { colors, fonts, spacing } from '@/theme';
import { SUCCESS_MESSAGES, ERROR_MESSAGES, VALIDATION } from '@/constants';
import type { NativeStackScreenProps } from '@react-navigation/native-stack';
import type { RootStackParamList } from '@/types';

type Props = NativeStackScreenProps<RootStackParamList, 'ImportWallet'>;

export function ImportWalletScreen({ navigation }: Props) {
  const { importWallet } = useWalletContext();
  const [walletName, setWalletName] = useState('');
  const [seedPhrase, setSeedPhrase] = useState('');
  const [isImporting, setIsImporting] = useState(false);
  const [step, setStep] = useState<'seed' | 'name'>('seed');

  const handleSeedPhraseChange = (text: string) => {
    // Clean up the input - remove extra spaces and convert to lowercase
    const cleaned = text.toLowerCase().replace(/\s+/g, ' ').trim();
    setSeedPhrase(cleaned);
  };

  const validateSeed = () => {
    if (!seedPhrase.trim()) {
      Alert.alert('Error', 'Please enter your recovery phrase');
      return false;
    }

    const words = seedPhrase.trim().split(/\s+/);
    if (!VALIDATION.SEED_PHRASE_WORDS.includes(words.length as 12 | 24)) {
      Alert.alert(
        'Invalid Recovery Phrase',
        `Recovery phrase must be ${VALIDATION.SEED_PHRASE_WORDS.join(' or ')} words`
      );
      return false;
    }

    if (!validateSeedPhrase(seedPhrase)) {
      Alert.alert(
        'Invalid Recovery Phrase',
        'The recovery phrase you entered is not valid. Please check and try again.'
      );
      return false;
    }

    return true;
  };

  const handleContinueFromSeed = () => {
    if (!validateSeed()) {
      return;
    }
    setStep('name');
  };

  const handleImportWallet = async () => {
    if (!walletName.trim()) {
      Alert.alert('Error', 'Please enter a wallet name');
      return;
    }

    if (!validateSeed()) {
      return;
    }

    try {
      setIsImporting(true);
      await importWallet(seedPhrase, walletName);
      
      Alert.alert(
        'Success',
        SUCCESS_MESSAGES.WALLET_IMPORTED,
        [
          {
            text: 'OK',
            onPress: () => navigation.navigate('Main'),
          },
        ]
      );
    } catch (error) {
      Alert.alert(
        'Import Failed',
        error instanceof Error ? error.message : ERROR_MESSAGES.SEED_PHRASE_INVALID
      );
    } finally {
      setIsImporting(false);
    }
  };

  const handleGoBack = () => {
    if (step === 'seed') {
      navigation.goBack();
    } else {
      setStep('seed');
    }
  };

  const renderSeedStep = () => {
    const wordCount = seedPhrase.trim() ? seedPhrase.trim().split(/\s+/).length : 0;
    const isValidLength = VALIDATION.SEED_PHRASE_WORDS.includes(wordCount as 12 | 24);

    return (
      <View style={styles.stepContainer}>
        <View style={styles.header}>
          <Text style={styles.title}>Import Wallet</Text>
          <Text style={styles.subtitle}>
            Enter your 12 or 24-word recovery phrase to restore your wallet.
          </Text>
        </View>

        <Card style={styles.inputCard}>
          <Text style={styles.inputLabel}>Recovery Phrase</Text>
          <TextInput
            style={styles.seedInput}
            value={seedPhrase}
            onChangeText={handleSeedPhraseChange}
            placeholder="Enter your recovery phrase here..."
            placeholderTextColor={colors.textTertiary}
            multiline
            numberOfLines={4}
            textAlignVertical="top"
            autoCapitalize="none"
            autoCorrect={false}
            spellCheck={false}
            autoFocus
          />
          <View style={styles.wordCountContainer}>
            <Text style={[
              styles.wordCount,
              { color: isValidLength ? colors.success : colors.textTertiary }
            ]}>
              {wordCount} words
            </Text>
            {wordCount > 0 && (
              <Text style={[
                styles.validationStatus,
                { color: isValidLength ? colors.success : colors.warning }
              ]}>
                {isValidLength ? '✓ Valid length' : '⚠ Invalid length'}
              </Text>
            )}
          </View>
        </Card>

        <Card style={styles.tipCard}>
          <Text style={styles.tipIcon}>💡</Text>
          <Text style={styles.tipTitle}>Tips</Text>
          <Text style={styles.tipText}>
            • Separate each word with a space
            • Make sure there are no extra spaces
            • Words should be lowercase
            • Double-check for typos
          </Text>
        </Card>

        <View style={styles.actions}>
          <Button
            title="Continue"
            onPress={handleContinueFromSeed}
            variant="primary"
            size="large"
            fullWidth
            disabled={!isValidLength || wordCount === 0}
          />
        </View>
      </View>
    );
  };

  const renderNameStep = () => (
    <View style={styles.stepContainer}>
      <View style={styles.header}>
        <Text style={styles.title}>Name Your Wallet</Text>
        <Text style={styles.subtitle}>
          Choose a name to identify this wallet.
        </Text>
      </View>

      <Card style={styles.inputCard}>
        <Text style={styles.inputLabel}>Wallet Name</Text>
        <TextInput
          style={styles.textInput}
          value={walletName}
          onChangeText={setWalletName}
          placeholder="My Imported Wallet"
          placeholderTextColor={colors.textTertiary}
          maxLength={50}
          autoFocus
        />
      </Card>

      <View style={styles.actions}>
        <Button
          title="Import Wallet"
          onPress={handleImportWallet}
          variant="primary"
          size="large"
          fullWidth
          loading={isImporting}
        />
      </View>
    </View>
  );

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="light-content" backgroundColor={colors.background} />
      
      <ScrollView
        style={styles.scrollView}
        contentContainerStyle={styles.scrollContent}
        showsVerticalScrollIndicator={false}
        keyboardShouldPersistTaps="handled"
      >
        {step === 'seed' && renderSeedStep()}
        {step === 'name' && renderNameStep()}
      </ScrollView>

      <View style={styles.bottomActions}>
        <Button
          title="Back"
          onPress={handleGoBack}
          variant="ghost"
          size="medium"
        />
      </View>
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
    flexGrow: 1,
    paddingHorizontal: spacing.lg,
  },
  
  stepContainer: {
    flex: 1,
    paddingTop: spacing.xl,
  },
  
  header: {
    marginBottom: spacing.xl,
  },
  
  title: {
    fontSize: fonts['2xl'],
    fontFamily: fonts.bold,
    color: colors.text,
    marginBottom: spacing.sm,
  },
  
  subtitle: {
    fontSize: fonts.base,
    fontFamily: fonts.regular,
    color: colors.textSecondary,
    lineHeight: 24,
  },
  
  inputCard: {
    marginBottom: spacing.lg,
  },
  
  inputLabel: {
    fontSize: fonts.sm,
    fontFamily: fonts.medium,
    color: colors.text,
    marginBottom: spacing.sm,
  },
  
  seedInput: {
    fontSize: fonts.base,
    fontFamily: fonts.regular,
    color: colors.text,
    backgroundColor: colors.background,
    borderRadius: 12,
    paddingHorizontal: spacing.md,
    paddingVertical: spacing.md,
    marginBottom: spacing.sm,
    borderWidth: 1,
    borderColor: colors.border,
    minHeight: 120,
    lineHeight: 24,
  },
  
  textInput: {
    fontSize: fonts.base,
    fontFamily: fonts.regular,
    color: colors.text,
    backgroundColor: colors.background,
    borderRadius: 12,
    paddingHorizontal: spacing.md,
    paddingVertical: spacing.sm,
    borderWidth: 1,
    borderColor: colors.border,
  },
  
  wordCountContainer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  
  wordCount: {
    fontSize: fonts.xs,
    fontFamily: fonts.medium,
  },
  
  validationStatus: {
    fontSize: fonts.xs,
    fontFamily: fonts.medium,
  },
  
  tipCard: {
    backgroundColor: colors.info + '20',
    borderColor: colors.info,
    borderWidth: 1,
    marginBottom: spacing.xl,
  },
  
  tipIcon: {
    fontSize: 24,
    textAlign: 'center',
    marginBottom: spacing.sm,
  },
  
  tipTitle: {
    fontSize: fonts.base,
    fontFamily: fonts.bold,
    color: colors.info,
    textAlign: 'center',
    marginBottom: spacing.sm,
  },
  
  tipText: {
    fontSize: fonts.sm,
    fontFamily: fonts.regular,
    color: colors.text,
    lineHeight: 20,
  },
  
  actions: {
    gap: spacing.md,
  },
  
  bottomActions: {
    paddingHorizontal: spacing.lg,
    paddingVertical: spacing.md,
    borderTopWidth: 1,
    borderTopColor: colors.border,
  },
});
