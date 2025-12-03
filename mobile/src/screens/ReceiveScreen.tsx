/**
 * Chameleon Wallet - Receive Screen
 * Displays wallet address and QR code for receiving payments
 */

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  SafeAreaView,
  ScrollView,
  TouchableOpacity,
  Alert,
  StatusBar,
  TextInput,
  Dimensions,
} from 'react-native';
import QRCode from 'react-native-qrcode-svg';
import Clipboard from '@react-native-clipboard/clipboard';
import Share from 'react-native-share';
import { Card } from '@/components/Card';
import { Button } from '@/components/Button';
import { useWalletContext } from '@/context/WalletContext';
import { colors, fonts, spacing, borderRadius } from '@/theme';
import { formatCHML, truncateAddress } from '@/utils/format';
import { CHAMELEON } from '@/constants';
import type { NativeStackScreenProps } from '@react-navigation/native-stack';
import type { RootStackParamList } from '@/types';

type Props = NativeStackScreenProps<RootStackParamList, 'Receive'>;

const { width: screenWidth } = Dimensions.get('window');
const QR_SIZE = Math.min(screenWidth - 80, 280);

export function ReceiveScreen({ navigation }: Props) {
  const { state } = useWalletContext();
  const { selectedWallet, balance } = state;

  const [requestAmount, setRequestAmount] = useState('');
  const [requestMemo, setRequestMemo] = useState('');
  const [addressType, setAddressType] = useState<'public' | 'private'>('public');
  const [qrValue, setQrValue] = useState('');

  useEffect(() => {
    if (selectedWallet) {
      updateQRCode();
    }
  }, [selectedWallet, requestAmount, requestMemo, addressType]);

  const updateQRCode = () => {
    if (!selectedWallet) return;

    let qrData = selectedWallet.address;

    // Create payment request URI if amount is specified
    if (requestAmount && parseFloat(requestAmount) > 0) {
      const params = new URLSearchParams();
      params.append('amount', requestAmount);
      
      if (requestMemo) {
        params.append('memo', requestMemo);
      }
      
      if (addressType === 'private') {
        params.append('private', 'true');
      }
      
      qrData = `chameleon:${selectedWallet.address}?${params.toString()}`;
    }

    setQrValue(qrData);
  };

  const handleCopyAddress = async () => {
    if (!selectedWallet) return;

    try {
      await Clipboard.setString(selectedWallet.address);
      Alert.alert('Copied!', 'Wallet address copied to clipboard');
    } catch (error) {
      Alert.alert('Error', 'Failed to copy address to clipboard');
    }
  };

  const handleShareAddress = async () => {
    if (!selectedWallet) return;

    try {
      let shareMessage = `Send CHML to my wallet:\n${selectedWallet.address}`;
      
      if (requestAmount && parseFloat(requestAmount) > 0) {
        shareMessage = `Please send ${requestAmount} CHML to my wallet:\n${selectedWallet.address}`;
        
        if (requestMemo) {
          shareMessage += `\n\nMemo: ${requestMemo}`;
        }
      }

      const shareOptions = {
        title: 'Chameleon Wallet Address',
        message: shareMessage,
        subject: 'Chameleon Wallet Payment Request',
      };

      await Share.open(shareOptions);
    } catch (error) {
      if (error.message !== 'User did not share') {
        Alert.alert('Error', 'Failed to share address');
      }
    }
  };

  const handleClearRequest = () => {
    setRequestAmount('');
    setRequestMemo('');
  };

  const handleAddressTypeToggle = () => {
    setAddressType(prev => prev === 'public' ? 'private' : 'public');
  };

  if (!selectedWallet) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.emptyState}>
          <Text style={styles.emptyStateText}>No wallet found</Text>
        </View>
      </SafeAreaView>
    );
  }

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="light-content" backgroundColor={colors.background} />
      
      {/* Header */}
      <View style={styles.header}>
        <TouchableOpacity
          style={styles.backButton}
          onPress={() => navigation.goBack()}
        >
          <Text style={styles.backButtonText}>←</Text>
        </TouchableOpacity>
        <Text style={styles.headerTitle}>Receive CHML</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView
        style={styles.scrollView}
        contentContainerStyle={styles.scrollContent}
        showsVerticalScrollIndicator={false}
      >
        {/* QR Code Card */}
        <Card style={styles.qrCard}>
          <Text style={styles.qrTitle}>Scan to Send</Text>
          
          <View style={styles.qrContainer}>
            <QRCode
              value={qrValue}
              size={QR_SIZE}
              backgroundColor={colors.white}
              color={colors.black}
              logoSize={40}
              logoBackgroundColor={colors.white}
              logoMargin={4}
              logoBorderRadius={8}
            />
          </View>
          
          {requestAmount && parseFloat(requestAmount) > 0 && (
            <View style={styles.requestInfo}>
              <Text style={styles.requestLabel}>Requested Amount</Text>
              <Text style={styles.requestAmount}>{formatCHML(requestAmount)} CHML</Text>
              {requestMemo && (
                <Text style={styles.requestMemo}>{requestMemo}</Text>
              )}
            </View>
          )}
        </Card>

        {/* Address Display */}
        <Card style={styles.addressCard}>
          <View style={styles.addressHeader}>
            <Text style={styles.addressLabel}>Your Wallet Address</Text>
            <View style={styles.addressTypeToggle}>
              <TouchableOpacity
                style={[
                  styles.addressTypeButton,
                  addressType === 'public' && styles.addressTypeButtonActive,
                ]}
                onPress={() => setAddressType('public')}
              >
                <Text style={[
                  styles.addressTypeText,
                  addressType === 'public' && styles.addressTypeTextActive,
                ]}>🔓 Public</Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={[
                  styles.addressTypeButton,
                  addressType === 'private' && styles.addressTypeButtonActive,
                ]}
                onPress={() => setAddressType('private')}
              >
                <Text style={[
                  styles.addressTypeText,
                  addressType === 'private' && styles.addressTypeTextActive,
                ]}>🛡️ Private</Text>
              </TouchableOpacity>
            </View>
          </View>
          
          <TouchableOpacity style={styles.addressContainer} onPress={handleCopyAddress}>
            <Text style={styles.addressText}>{selectedWallet.address}</Text>
            <View style={styles.copyIcon}>
              <Text style={styles.copyIconText}>📋</Text>
            </View>
          </TouchableOpacity>
          
          <Text style={styles.addressHint}>
            Tap to copy • {addressType === 'private' ? 'Private' : 'Public'} transactions only
          </Text>
        </Card>

        {/* Payment Request */}
        <Card style={styles.requestCard}>
          <Text style={styles.requestTitle}>Request Specific Amount (Optional)</Text>
          
          <View style={styles.amountInputContainer}>
            <TextInput
              style={styles.amountInput}
              value={requestAmount}
              onChangeText={(text) => {
                // Only allow numbers and decimal point
                const cleanText = text.replace(/[^0-9.]/g, '');
                setRequestAmount(cleanText);
              }}
              placeholder="0.00"
              placeholderTextColor={colors.textTertiary}
              keyboardType="decimal-pad"
            />
            <Text style={styles.currencyLabel}>CHML</Text>
          </View>
          
          <TextInput
            style={styles.memoInput}
            value={requestMemo}
            onChangeText={setRequestMemo}
            placeholder="Add a note (optional)..."
            placeholderTextColor={colors.textTertiary}
            multiline
            maxLength={100}
          />
          
          {(requestAmount || requestMemo) && (
            <TouchableOpacity style={styles.clearButton} onPress={handleClearRequest}>
              <Text style={styles.clearButtonText}>Clear Request</Text>
            </TouchableOpacity>
          )}
        </Card>

        {/* Balance Display */}
        <Card style={styles.balanceCard}>
          <Text style={styles.balanceTitle}>Current Balance</Text>
          <View style={styles.balanceRow}>
            <View style={styles.balanceItem}>
              <Text style={styles.balanceLabel}>🔓 Public</Text>
              <Text style={styles.balanceAmount}>
                {formatCHML(balance?.public || '0')} CHML
              </Text>
            </View>
            <View style={styles.balanceItem}>
              <Text style={styles.balanceLabel}>🛡️ Private</Text>
              <Text style={styles.balanceAmount}>
                {formatCHML(balance?.shielded || '0')} CHML
              </Text>
            </View>
          </View>
          {balance?.usdValue && (
            <Text style={styles.balanceUsd}>
              Total: ≈ ${(
                (parseFloat(balance.public || '0') + parseFloat(balance.shielded || '0')) *
                parseFloat(balance.usdValue)
              ).toFixed(2)}
            </Text>
          )}
        </Card>

        {/* Action Buttons */}
        <View style={styles.actionButtons}>
          <Button
            title="Copy Address"
            onPress={handleCopyAddress}
            variant="secondary"
            style={styles.actionButton}
          />
          <Button
            title="Share"
            onPress={handleShareAddress}
            style={styles.actionButton}
          />
        </View>

        {/* Instructions */}
        <Card style={styles.instructionsCard}>
          <Text style={styles.instructionsTitle}>How to Receive CHML</Text>
          <View style={styles.instructionsList}>
            <View style={styles.instructionItem}>
              <Text style={styles.instructionNumber}>1</Text>
              <Text style={styles.instructionText}>
                Share your wallet address or QR code with the sender
              </Text>
            </View>
            <View style={styles.instructionItem}>
              <Text style={styles.instructionNumber}>2</Text>
              <Text style={styles.instructionText}>
                Choose between public (transparent) or private (shielded) transactions
              </Text>
            </View>
            <View style={styles.instructionItem}>
              <Text style={styles.instructionNumber}>3</Text>
              <Text style={styles.instructionText}>
                Optionally specify an amount and memo for payment requests
              </Text>
            </View>
            <View style={styles.instructionItem}>
              <Text style={styles.instructionNumber}>4</Text>
              <Text style={styles.instructionText}>
                Funds will appear in your wallet once the transaction is confirmed
              </Text>
            </View>
          </View>
        </Card>
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: colors.background,
  },
  
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: spacing.lg,
    paddingVertical: spacing.md,
    borderBottomWidth: 1,
    borderBottomColor: colors.border,
  },
  
  backButton: {
    width: 40,
    height: 40,
    alignItems: 'center',
    justifyContent: 'center',
  },
  
  backButtonText: {
    fontSize: 24,
    color: colors.text,
  },
  
  headerTitle: {
    fontSize: fonts.lg,
    fontFamily: fonts.bold,
    color: colors.text,
  },
  
  headerRight: {
    width: 40,
  },
  
  scrollView: {
    flex: 1,
  },
  
  scrollContent: {
    padding: spacing.lg,
    paddingBottom: spacing.xl,
  },
  
  qrCard: {
    alignItems: 'center',
    marginBottom: spacing.lg,
    paddingVertical: spacing.xl,
  },
  
  qrTitle: {
    fontSize: fonts.lg,
    fontFamily: fonts.bold,
    color: colors.text,
    marginBottom: spacing.lg,
  },
  
  qrContainer: {
    padding: spacing.md,
    backgroundColor: colors.white,
    borderRadius: borderRadius.lg,
    marginBottom: spacing.lg,
  },
  
  requestInfo: {
    alignItems: 'center',
  },
  
  requestLabel: {
    fontSize: fonts.sm,
    fontFamily: fonts.medium,
    color: colors.textSecondary,
    marginBottom: spacing.xs,
  },
  
  requestAmount: {
    fontSize: fonts.xl,
    fontFamily: fonts.bold,
    color: colors.primary,
  },
  
  requestMemo: {
    fontSize: fonts.sm,
    fontFamily: fonts.regular,
    color: colors.textSecondary,
    marginTop: spacing.xs,
    textAlign: 'center',
  },
  
  addressCard: {
    marginBottom: spacing.lg,
  },
  
  addressHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: spacing.md,
  },
  
  addressLabel: {
    fontSize: fonts.sm,
    fontFamily: fonts.medium,
    color: colors.text,
  },
  
  addressTypeToggle: {
    flexDirection: 'row',
    backgroundColor: colors.surface,
    borderRadius: borderRadius.md,
    padding: 2,
  },
  
  addressTypeButton: {
    paddingHorizontal: spacing.sm,
    paddingVertical: spacing.xs,
    borderRadius: borderRadius.sm,
  },
  
  addressTypeButtonActive: {
    backgroundColor: colors.primary,
  },
  
  addressTypeText: {
    fontSize: fonts.xs,
    fontFamily: fonts.medium,
    color: colors.textSecondary,
  },
  
  addressTypeTextActive: {
    color: colors.white,
  },
  
  addressContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: colors.surface,
    borderRadius: borderRadius.md,
    padding: spacing.md,
    borderWidth: 1,
    borderColor: colors.border,
  },
  
  addressText: {
    flex: 1,
    fontSize: fonts.sm,
    fontFamily: fonts.regular,
    color: colors.text,
    lineHeight: 20,
  },
  
  copyIcon: {
    marginLeft: spacing.sm,
  },
  
  copyIconText: {
    fontSize: 20,
  },
  
  addressHint: {
    fontSize: fonts.xs,
    fontFamily: fonts.regular,
    color: colors.textTertiary,
    textAlign: 'center',
    marginTop: spacing.sm,
  },
  
  requestCard: {
    marginBottom: spacing.lg,
  },
  
  requestTitle: {
    fontSize: fonts.base,
    fontFamily: fonts.medium,
    color: colors.text,
    marginBottom: spacing.md,
  },
  
  amountInputContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: spacing.md,
  },
  
  amountInput: {
    flex: 1,
    fontSize: fonts.xl,
    fontFamily: fonts.bold,
    color: colors.text,
    backgroundColor: colors.surface,
    borderRadius: borderRadius.md,
    paddingHorizontal: spacing.md,
    paddingVertical: spacing.md,
    borderWidth: 1,
    borderColor: colors.border,
    textAlign: 'right',
  },
  
  currencyLabel: {
    fontSize: fonts.base,
    fontFamily: fonts.medium,
    color: colors.textSecondary,
    marginLeft: spacing.sm,
  },
  
  memoInput: {
    fontSize: fonts.base,
    fontFamily: fonts.regular,
    color: colors.text,
    backgroundColor: colors.surface,
    borderRadius: borderRadius.md,
    paddingHorizontal: spacing.md,
    paddingVertical: spacing.sm,
    borderWidth: 1,
    borderColor: colors.border,
    minHeight: 60,
    textAlignVertical: 'top',
    marginBottom: spacing.md,
  },
  
  clearButton: {
    alignSelf: 'center',
  },
  
  clearButtonText: {
    fontSize: fonts.sm,
    fontFamily: fonts.medium,
    color: colors.error,
  },
  
  balanceCard: {
    marginBottom: spacing.lg,
  },
  
  balanceTitle: {
    fontSize: fonts.base,
    fontFamily: fonts.medium,
    color: colors.text,
    marginBottom: spacing.md,
  },
  
  balanceRow: {
    flexDirection: 'row',
    gap: spacing.md,
  },
  
  balanceItem: {
    flex: 1,
    alignItems: 'center',
    padding: spacing.md,
    backgroundColor: colors.surface,
    borderRadius: borderRadius.md,
  },
  
  balanceLabel: {
    fontSize: fonts.xs,
    fontFamily: fonts.medium,
    color: colors.textSecondary,
    marginBottom: spacing.xs,
  },
  
  balanceAmount: {
    fontSize: fonts.base,
    fontFamily: fonts.bold,
    color: colors.text,
  },
  
  balanceUsd: {
    fontSize: fonts.sm,
    fontFamily: fonts.regular,
    color: colors.textSecondary,
    textAlign: 'center',
    marginTop: spacing.md,
  },
  
  actionButtons: {
    flexDirection: 'row',
    gap: spacing.md,
    marginBottom: spacing.lg,
  },
  
  actionButton: {
    flex: 1,
  },
  
  instructionsCard: {
    marginBottom: spacing.lg,
  },
  
  instructionsTitle: {
    fontSize: fonts.base,
    fontFamily: fonts.medium,
    color: colors.text,
    marginBottom: spacing.md,
  },
  
  instructionsList: {
    gap: spacing.md,
  },
  
  instructionItem: {
    flexDirection: 'row',
    alignItems: 'flex-start',
  },
  
  instructionNumber: {
    width: 24,
    height: 24,
    borderRadius: 12,
    backgroundColor: colors.primary,
    color: colors.white,
    fontSize: fonts.sm,
    fontFamily: fonts.bold,
    textAlign: 'center',
    lineHeight: 24,
    marginRight: spacing.sm,
  },
  
  instructionText: {
    flex: 1,
    fontSize: fonts.sm,
    fontFamily: fonts.regular,
    color: colors.textSecondary,
    lineHeight: 20,
  },
  
  emptyState: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
  },
  
  emptyStateText: {
    fontSize: fonts.lg,
    fontFamily: fonts.medium,
    color: colors.textSecondary,
  },
});
