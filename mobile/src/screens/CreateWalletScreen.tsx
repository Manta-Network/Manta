/**
 * Chameleon Wallet - Create Wallet Screen
 */

import React, { useState, useEffect } from 'react';
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
import { generateSeedPhrase } from '@/utils/crypto';
import { formatSeedPhrase } from '@/utils/format';
import { colors, fonts, spacing } from '@/theme';
import { SUCCESS_MESSAGES, ERROR_MESSAGES } from '@/constants';
import type { NativeStackScreenProps } from '@react-navigation/native-stack';
import type { RootStackParamList } from '@/types';

type Props = NativeStackScreenProps<RootStackParamList, 'CreateWallet'>;

export function CreateWalletScreen({ navigation }: Props) {
  const [walletName, setWalletName] = useState('');
  const [seedPhrase, setSeedPhrase] = useState('');
  const [isGenerating, setIsGenerating] = useState(false);
  const [step, setStep] = useState<'name' | 'seed' | 'confirm'>('name');

  useEffect(() => {
    // Generate seed phrase when component mounts
    generateNewSeedPhrase();
  }, []);

  const generateNewSeedPhrase = async () => {
    try {
      setIsGenerating(true);
      // Add a small delay to show loading state
      await new Promise(resolve => setTimeout(resolve, 500));
      const newSeed = generateSeedPhrase(12);
      setSeedPhrase(newSeed);
    } catch (error) {
      Alert.alert('Error', 'Failed to generate seed phrase. Please try again.');
    } finally {
      setIsGenerating(false);
    }
  };

  const handleContinueFromName = () => {
    if (!walletName.trim()) {
      Alert.alert('Error', 'Please enter a wallet name');
      return;
    }
    setStep('seed');
  };

  const handleContinueFromSeed = () => {
    navigation.navigate('BackupSeed', { seed: seedPhrase });
  };

  const handleGoBack = () => {
    if (step === 'name') {
      navigation.goBack();
    } else {
      setStep('name');
    }
  };

  const renderNameStep = () => (
    <View style={styles.stepContainer}>
      <View style={styles.header}>
        <Text style={styles.title}>Create New Wallet</Text>
        <Text style={styles.subtitle}>
          Choose a name for your wallet. This will help you identify it later.
        </Text>
      </View>

      <Card style={styles.inputCard}>
        <Text style={styles.inputLabel}>Wallet Name</Text>
        <TextInput
          style={styles.textInput}
          value={walletName}
          onChangeText={setWalletName}
          placeholder="My Chameleon Wallet"
          placeholderTextColor={colors.textTertiary}
          maxLength={50}
          autoFocus
        />
        <Text style={styles.inputHint}>
          Choose a memorable name (you can change this later)
        </Text>
      </Card>

      <View style={styles.actions}>
        <Button
          title="Continue"
          onPress={handleContinueFromName}
          variant="primary"
          size="large"
          fullWidth
        />
      </View>
    </View>
  );

  const renderSeedStep = () => {
    const formattedSeed = formatSeedPhrase(seedPhrase);

    return (
      <View style={styles.stepContainer}>
        <View style={styles.header}>
          <Text style={styles.title}>Your Recovery Phrase</Text>
          <Text style={styles.subtitle}>
            Write down these 12 words in order. You'll need them to recover your wallet.
          </Text>
        </View>

        <Card style={styles.seedCard}>
          <View style={styles.seedGrid}>
            {formattedSeed.map((item) => (
              <View key={item.number} style={styles.seedItem}>
                <Text style={styles.seedNumber}>{item.number}</Text>
                <Text style={styles.seedWord}>{item.word}</Text>
              </View>
            ))}
          </View>
        </Card>

        <Card style={styles.warningCard}>
          <Text style={styles.warningIcon}>⚠️</Text>
          <Text style={styles.warningTitle}>Important Security Notice</Text>
          <Text style={styles.warningText}>
            • Never share your recovery phrase with anyone
            • Store it in a safe, offline location
            • Anyone with this phrase can access your funds
            • Chameleon cannot recover lost phrases
          </Text>
        </Card>

        <View style={styles.actions}>
          <Button
            title="I've Written It Down"
            onPress={handleContinueFromSeed}
            variant="primary"
            size="large"
            fullWidth
          />
          
          <Button
            title="Generate New Phrase"
            onPress={generateNewSeedPhrase}
            variant="outline"
            size="medium"
            fullWidth
            loading={isGenerating}
            style={styles.regenerateButton}
          />
        </View>
      </View>
    );
  };

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="light-content" backgroundColor={colors.background} />
      
      <ScrollView
        style={styles.scrollView}
        contentContainerStyle={styles.scrollContent}
        showsVerticalScrollIndicator={false}
      >
        {step === 'name' && renderNameStep()}
        {step === 'seed' && renderSeedStep()}
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
    marginBottom: spacing.xl,
  },
  
  inputLabel: {
    fontSize: fonts.sm,
    fontFamily: fonts.medium,
    color: colors.text,
    marginBottom: spacing.sm,
  },
  
  textInput: {
    fontSize: fonts.base,
    fontFamily: fonts.regular,
    color: colors.text,
    backgroundColor: colors.background,
    borderRadius: 12,
    paddingHorizontal: spacing.md,
    paddingVertical: spacing.sm,
    marginBottom: spacing.sm,
    borderWidth: 1,
    borderColor: colors.border,
  },
  
  inputHint: {
    fontSize: fonts.xs,
    fontFamily: fonts.regular,
    color: colors.textTertiary,
  },
  
  seedCard: {
    marginBottom: spacing.lg,
  },
  
  seedGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: spacing.sm,
  },
  
  seedItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: colors.background,
    borderRadius: 8,
    paddingHorizontal: spacing.sm,
    paddingVertical: spacing.xs,
    width: '48%',
  },
  
  seedNumber: {
    fontSize: fonts.xs,
    fontFamily: fonts.medium,
    color: colors.textTertiary,
    marginRight: spacing.xs,
    minWidth: 16,
  },
  
  seedWord: {
    fontSize: fonts.sm,
    fontFamily: fonts.medium,
    color: colors.text,
    flex: 1,
  },
  
  warningCard: {
    backgroundColor: colors.warning + '20',
    borderColor: colors.warning,
    borderWidth: 1,
    marginBottom: spacing.xl,
  },
  
  warningIcon: {
    fontSize: 24,
    textAlign: 'center',
    marginBottom: spacing.sm,
  },
  
  warningTitle: {
    fontSize: fonts.base,
    fontFamily: fonts.bold,
    color: colors.warning,
    textAlign: 'center',
    marginBottom: spacing.sm,
  },
  
  warningText: {
    fontSize: fonts.sm,
    fontFamily: fonts.regular,
    color: colors.text,
    lineHeight: 20,
  },
  
  actions: {
    gap: spacing.md,
  },
  
  regenerateButton: {
    marginTop: spacing.sm,
  },
  
  bottomActions: {
    paddingHorizontal: spacing.lg,
    paddingVertical: spacing.md,
    borderTopWidth: 1,
    borderTopColor: colors.border,
  },
});
