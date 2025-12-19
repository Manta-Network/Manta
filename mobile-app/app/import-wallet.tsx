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

export default ImportWalletScreen;
