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
  StyleSheet,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useWallet } from '../context/WalletContext';
import { walletService, DEV_ACCOUNTS } from '../services/wallet';
import { ScreenContainer } from '../components/ScreenContainer';
import { THEME } from '../constants/theme';

const ImportWalletScreen = () => {
  const router = useRouter();
  const { importWallet, importDevAccount, isLoading } = useWallet();
  const [mnemonic, setMnemonic] = useState('');
  const [selectedDevAccount, setSelectedDevAccount] = useState(null);
  const [addressPreview, setAddressPreview] = useState('');
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

  const handleDevAccountSelect = async (accountName) => {
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
    <ScreenContainer scrollable showGradient={false}>
      <SafeAreaView style={styles.container}>
        {/* Header */}
        <View style={styles.header}>
          <TouchableOpacity
            onPress={() => router.back()}
            style={styles.backButton}
          >
            <Ionicons name="arrow-back" size={24} color={THEME.colors.text} />
          </TouchableOpacity>
          <Text style={styles.headerTitle}>Import Wallet</Text>
        </View>

        {/* Dev Accounts Section */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>
            Quick Import (Dev Accounts)
          </Text>
          <View style={styles.devAccountsGrid}>
            {(Object.keys(DEV_ACCOUNTS) as Array<keyof typeof DEV_ACCOUNTS>).map((account) => (
              <TouchableOpacity
                key={account}
                style={[
                  styles.devAccountButton,
                  selectedDevAccount === account ? styles.devAccountButtonSelected : styles.devAccountButtonDefault
                ]}
                onPress={() => handleDevAccountSelect(account)}
              >
                <Text
                  style={[
                    styles.devAccountButtonText,
                    selectedDevAccount === account ? styles.devAccountButtonTextSelected : styles.devAccountButtonTextDefault
                  ]}
                >
                  {account}
                </Text>
              </TouchableOpacity>
            ))}
          </View>
          <Text style={styles.devAccountsNote}>
            These are pre-funded development accounts for testing.
          </Text>
        </View>

        {/* Divider */}
        <View style={styles.divider}>
          <View style={styles.dividerLine} />
          <Text style={styles.dividerText}>Or enter seed phrase</Text>
          <View style={styles.dividerLine} />
        </View>

        {/* Seed Phrase Input */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>
            Seed Phrase
          </Text>
          <View style={styles.inputContainer}>
            <TextInput
              style={styles.textInput}
              placeholder="Enter your 12 or 24 word seed phrase..."
              placeholderTextColor={THEME.colors.textMuted}
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
            <View style={styles.validationRow}>
              <Ionicons
                name={isValidMnemonic ? 'checkmark-circle' : 'close-circle'}
                size={20}
                color={isValidMnemonic ? THEME.colors.success : THEME.colors.error}
              />
              <Text
                style={[
                  styles.validationText,
                  { color: isValidMnemonic ? THEME.colors.success : THEME.colors.error }
                ]}
              >
                {isValidMnemonic ? 'Valid seed phrase' : 'Invalid seed phrase'}
              </Text>
            </View>
          )}
        </View>

        {/* Address Preview */}
        {addressPreview && (
          <View style={styles.previewContainer}>
            <Text style={styles.previewLabel}>Wallet Address Preview:</Text>
            <Text style={styles.previewAddress}>
              {formatAddressForPreview(addressPreview)}
            </Text>
          </View>
        )}

        {/* Import Button */}
        <TouchableOpacity
          style={[
            styles.importButton,
            isValidMnemonic && !isLoading ? styles.importButtonEnabled : styles.importButtonDisabled
          ]}
          onPress={handleImportWallet}
          disabled={!isValidMnemonic || isLoading}
        >
          {isLoading ? (
            <ActivityIndicator color={THEME.colors.white} />
          ) : (
            <Text style={styles.importButtonText}>
              Import Wallet
            </Text>
          )}
        </TouchableOpacity>

        {/* Security Note */}
        <View style={styles.securityNote}>
          <View style={styles.securityNoteHeader}>
            <Ionicons name="shield-checkmark" size={20} color={THEME.colors.warning} />
            <Text style={styles.securityNoteTitle}>Security Note</Text>
          </View>
          <Text style={styles.securityNoteText}>
            Your seed phrase is stored securely on your device and never sent to our servers.
          </Text>
        </View>
      </SafeAreaView>
    </ScreenContainer>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    paddingHorizontal: THEME.spacing.lg,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: THEME.spacing.xl,
    paddingVertical: THEME.spacing.sm,
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
  section: {
    marginBottom: THEME.spacing.xl,
  },
  sectionTitle: {
    fontSize: THEME.fontSize.lg,
    fontWeight: THEME.fontWeight.semibold,
    color: THEME.colors.text,
    marginBottom: THEME.spacing.md,
  },
  devAccountsGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: THEME.spacing.sm,
    marginBottom: THEME.spacing.md,
  },
  devAccountButton: {
    borderRadius: THEME.borderRadius.small,
    paddingVertical: THEME.spacing.sm,
    paddingHorizontal: THEME.spacing.md,
    borderWidth: 1,
  },
  devAccountButtonDefault: {
    backgroundColor: THEME.colors.card,
    borderColor: THEME.colors.border,
  },
  devAccountButtonSelected: {
    backgroundColor: THEME.colors.primary,
    borderColor: THEME.colors.primary,
  },
  devAccountButtonText: {
    fontWeight: THEME.fontWeight.medium,
    textTransform: 'capitalize',
  },
  devAccountButtonTextDefault: {
    color: THEME.colors.secondary,
  },
  devAccountButtonTextSelected: {
    color: THEME.colors.white,
  },
  devAccountsNote: {
    color: THEME.colors.textSecondary,
    fontSize: THEME.fontSize.sm,
  },
  divider: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: THEME.spacing.xl,
  },
  dividerLine: {
    flex: 1,
    height: 1,
    backgroundColor: THEME.colors.border,
  },
  dividerText: {
    color: THEME.colors.textSecondary,
    fontSize: THEME.fontSize.sm,
    marginHorizontal: THEME.spacing.md,
  },
  inputContainer: {
    backgroundColor: THEME.colors.white,
    borderRadius: THEME.borderRadius.medium,
    borderWidth: 1,
    borderColor: THEME.colors.border,
    padding: THEME.spacing.md,
    ...THEME.shadows.small,
  },
  textInput: {
    color: THEME.colors.text,
    fontSize: THEME.fontSize.base,
    fontFamily: 'monospace',
    minHeight: 120,
    textAlignVertical: 'top',
  },
  validationRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: THEME.spacing.sm,
  },
  validationText: {
    marginLeft: THEME.spacing.sm,
    fontSize: THEME.fontSize.sm,
  },
  previewContainer: {
    backgroundColor: THEME.colors.white,
    borderRadius: THEME.borderRadius.medium,
    padding: THEME.spacing.md,
    marginBottom: THEME.spacing.xl,
    borderWidth: 1,
    borderColor: THEME.colors.border,
    ...THEME.shadows.small,
  },
  previewLabel: {
    color: THEME.colors.textSecondary,
    fontSize: THEME.fontSize.sm,
    marginBottom: THEME.spacing.xs,
  },
  previewAddress: {
    color: THEME.colors.text,
    fontFamily: 'monospace',
    fontSize: THEME.fontSize.base,
  },
  importButton: {
    borderRadius: THEME.borderRadius.medium,
    paddingVertical: THEME.spacing.md,
    paddingHorizontal: THEME.spacing.lg,
    marginBottom: THEME.spacing.lg,
  },
  importButtonEnabled: {
    backgroundColor: THEME.colors.primary,
  },
  importButtonDisabled: {
    backgroundColor: THEME.colors.lightGrey,
    opacity: 0.5,
  },
  importButtonText: {
    color: THEME.colors.white,
    textAlign: 'center',
    fontSize: THEME.fontSize.lg,
    fontWeight: THEME.fontWeight.semibold,
  },
  securityNote: {
    backgroundColor: THEME.colors.warningBg,
    borderWidth: 1,
    borderColor: THEME.colors.warning,
    borderRadius: THEME.borderRadius.medium,
    padding: THEME.spacing.md,
  },
  securityNoteHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: THEME.spacing.xs,
  },
  securityNoteTitle: {
    color: THEME.colors.warning,
    fontWeight: THEME.fontWeight.semibold,
    marginLeft: THEME.spacing.sm,
  },
  securityNoteText: {
    color: THEME.colors.warning,
    fontSize: THEME.fontSize.sm,
    lineHeight: 20,
  },
});

export default ImportWalletScreen;
