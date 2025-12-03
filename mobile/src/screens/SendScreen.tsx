/**
 * Chameleon Wallet - Send Screen
 * Allows users to send CHML tokens with privacy options
 */

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  SafeAreaView,
  ScrollView,
  TextInput,
  TouchableOpacity,
  Alert,
  ActivityIndicator,
  StatusBar,
  Modal,
} from 'react-native';
import Clipboard from '@react-native-clipboard/clipboard';
import { Card } from '@/components/Card';
import { Button } from '@/components/Button';
import { useWalletContext } from '@/context/WalletContext';
import { TransactionService } from '@/services/transaction';
import { colors, fonts, spacing, borderRadius } from '@/theme';
import { formatCHML, truncateAddress } from '@/utils/format';
import { CHAMELEON, VALIDATION } from '@/constants';
import type { NativeStackScreenProps } from '@react-navigation/native-stack';
import type { RootStackParamList } from '@/types';
import type { TransactionRequest, FeeEstimate } from '@/services/transaction';

type Props = NativeStackScreenProps<RootStackParamList, 'Send'>;

interface FormData {
  recipient: string;
  amount: string;
  memo: string;
  isPrivate: boolean;
}

interface FormErrors {
  recipient?: string;
  amount?: string;
  general?: string;
}

export function SendScreen({ navigation, route }: Props) {
  const { state } = useWalletContext();
  const { selectedWallet, balance } = state;

  // Form state
  const [formData, setFormData] = useState<FormData>({
    recipient: route.params?.recipient || '',
    amount: route.params?.amount || '',
    memo: '',
    isPrivate: true, // Default to private
  });

  const [errors, setErrors] = useState<FormErrors>({});
  const [feeEstimate, setFeeEstimate] = useState<FeeEstimate | null>(null);
  const [isLoadingFee, setIsLoadingFee] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [showConfirmModal, setShowConfirmModal] = useState(false);

  // Calculate available balance based on privacy mode
  const availableBalance = formData.isPrivate 
    ? balance?.shielded || '0'
    : balance?.public || '0';

  useEffect(() => {
    // Update fee estimate when amount or privacy mode changes
    if (formData.amount && parseFloat(formData.amount) > 0) {
      updateFeeEstimate();
    } else {
      setFeeEstimate(null);
    }
  }, [formData.amount, formData.isPrivate]);

  const updateFeeEstimate = async () => {
    if (!formData.amount) return;

    setIsLoadingFee(true);
    try {
      const estimate = await TransactionService.getFeeEstimate(
        formData.isPrivate,
        formData.amount
      );
      setFeeEstimate(estimate);
    } catch (error) {
      console.error('Failed to get fee estimate:', error);
    } finally {
      setIsLoadingFee(false);
    }
  };

  const validateForm = (): boolean => {
    const newErrors: FormErrors = {};

    // Validate recipient
    if (!formData.recipient.trim()) {
      newErrors.recipient = 'Recipient address is required';
    } else if (!TransactionService.validateAddress(formData.recipient)) {
      newErrors.recipient = 'Invalid wallet address format';
    } else if (formData.recipient === selectedWallet?.address) {
      newErrors.recipient = 'Cannot send to your own address';
    }

    // Validate amount
    if (!formData.amount.trim()) {
      newErrors.amount = 'Amount is required';
    } else {
      const validation = TransactionService.validateAmount(
        formData.amount,
        availableBalance
      );
      if (!validation.isValid) {
        newErrors.amount = validation.error;
      }
    }

    // Check if we have enough balance including fees
    if (feeEstimate && formData.amount) {
      const totalRequired = parseFloat(formData.amount) + parseFloat(feeEstimate.totalFee);
      if (totalRequired > parseFloat(availableBalance)) {
        newErrors.amount = 'Insufficient balance including fees';
      }
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handlePasteAddress = async () => {
    try {
      const clipboardText = await Clipboard.getString();
      if (clipboardText && TransactionService.validateAddress(clipboardText)) {
        setFormData(prev => ({ ...prev, recipient: clipboardText }));
        setErrors(prev => ({ ...prev, recipient: undefined }));
      } else {
        Alert.alert('Invalid Address', 'Clipboard does not contain a valid wallet address');
      }
    } catch (error) {
      Alert.alert('Error', 'Failed to read from clipboard');
    }
  };

  const handleMaxAmount = async () => {
    try {
      const maxAmount = await TransactionService.getMaxSendableAmount(
        availableBalance,
        formData.isPrivate
      );
      setFormData(prev => ({ ...prev, amount: maxAmount }));
      setErrors(prev => ({ ...prev, amount: undefined }));
    } catch (error) {
      Alert.alert('Error', 'Failed to calculate maximum amount');
    }
  };

  const handlePrivacyToggle = () => {
    setFormData(prev => ({ ...prev, isPrivate: !prev.isPrivate }));
  };

  const handleSend = () => {
    if (validateForm()) {
      setShowConfirmModal(true);
    }
  };

  const handleConfirmSend = async () => {
    setShowConfirmModal(false);
    setIsSubmitting(true);

    try {
      const request: TransactionRequest = {
        recipient: formData.recipient,
        amount: formData.amount,
        isPrivate: formData.isPrivate,
        memo: formData.memo || undefined,
      };

      const result = await TransactionService.sendTransaction(request);

      Alert.alert(
        'Transaction Sent!',
        `Your ${formData.isPrivate ? 'private' : 'public'} transaction has been submitted.\n\nTransaction Hash: ${truncateAddress(result.hash)}`,
        [
          {
            text: 'View Details',
            onPress: () => {
              // Navigate to transaction details
              navigation.navigate('TransactionDetail', {
                transaction: {
                  hash: result.hash,
                  type: 'send',
                  amount: formData.amount,
                  timestamp: result.timestamp,
                  status: result.status,
                  isPrivate: formData.isPrivate,
                  from: selectedWallet?.address || '',
                  to: formData.recipient,
                  fee: result.fee,
                  memo: formData.memo,
                  blockNumber: result.blockNumber,
                },
              });
            },
          },
          {
            text: 'Done',
            onPress: () => navigation.goBack(),
          },
        ]
      );
    } catch (error) {
      Alert.alert(
        'Transaction Failed',
        error instanceof Error ? error.message : 'An unknown error occurred'
      );
    } finally {
      setIsSubmitting(false);
    }
  };

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
        <Text style={styles.headerTitle}>Send CHML</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView
        style={styles.scrollView}
        contentContainerStyle={styles.scrollContent}
        keyboardShouldPersistTaps="handled"
        showsVerticalScrollIndicator={false}
      >
        {/* Balance Display */}
        <Card style={styles.balanceCard}>
          <Text style={styles.balanceLabel}>
            Available Balance ({formData.isPrivate ? 'Private' : 'Public'})
          </Text>
          <Text style={styles.balanceAmount}>
            {formatCHML(availableBalance)} CHML
          </Text>
          {balance?.usdValue && (
            <Text style={styles.balanceUsd}>
              ≈ ${(parseFloat(availableBalance) * parseFloat(balance.usdValue)).toFixed(2)}
            </Text>
          )}
        </Card>

        {/* Recipient Input */}
        <Card style={styles.inputCard}>
          <Text style={styles.inputLabel}>Recipient Address</Text>
          <View style={styles.inputContainer}>
            <TextInput
              style={[styles.textInput, errors.recipient && styles.inputError]}
              value={formData.recipient}
              onChangeText={(text) => {
                setFormData(prev => ({ ...prev, recipient: text }));
                setErrors(prev => ({ ...prev, recipient: undefined }));
              }}
              placeholder="5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
              placeholderTextColor={colors.textTertiary}
              autoCapitalize="none"
              autoCorrect={false}
              multiline
            />
            <TouchableOpacity style={styles.pasteButton} onPress={handlePasteAddress}>
              <Text style={styles.pasteButtonText}>Paste</Text>
            </TouchableOpacity>
          </View>
          {errors.recipient && (
            <Text style={styles.errorText}>{errors.recipient}</Text>
          )}
        </Card>

        {/* Amount Input */}
        <Card style={styles.inputCard}>
          <Text style={styles.inputLabel}>Amount</Text>
          <View style={styles.amountContainer}>
            <TextInput
              style={[styles.amountInput, errors.amount && styles.inputError]}
              value={formData.amount}
              onChangeText={(text) => {
                // Only allow numbers and decimal point
                const cleanText = text.replace(/[^0-9.]/g, '');
                setFormData(prev => ({ ...prev, amount: cleanText }));
                setErrors(prev => ({ ...prev, amount: undefined }));
              }}
              placeholder="0.00"
              placeholderTextColor={colors.textTertiary}
              keyboardType="decimal-pad"
            />
            <View style={styles.amountSuffix}>
              <TouchableOpacity style={styles.maxButton} onPress={handleMaxAmount}>
                <Text style={styles.maxButtonText}>MAX</Text>
              </TouchableOpacity>
              <Text style={styles.currencyText}>CHML</Text>
            </View>
          </View>
          {errors.amount && (
            <Text style={styles.errorText}>{errors.amount}</Text>
          )}
        </Card>

        {/* Privacy Toggle */}
        <Card style={styles.inputCard}>
          <View style={styles.privacyHeader}>
            <View>
              <Text style={styles.inputLabel}>Transaction Type</Text>
              <Text style={styles.privacyDescription}>
                {formData.isPrivate 
                  ? 'Private transactions are completely anonymous'
                  : 'Public transactions are visible on the blockchain'
                }
              </Text>
            </View>
            <TouchableOpacity
              style={[styles.toggle, formData.isPrivate && styles.toggleActive]}
              onPress={handlePrivacyToggle}
            >
              <View style={[styles.toggleThumb, formData.isPrivate && styles.toggleThumbActive]} />
            </TouchableOpacity>
          </View>
          <View style={styles.privacyOptions}>
            <View style={[styles.privacyOption, !formData.isPrivate && styles.privacyOptionActive]}>
              <Text style={styles.privacyOptionIcon}>🔓</Text>
              <Text style={styles.privacyOptionText}>Public</Text>
            </View>
            <View style={[styles.privacyOption, formData.isPrivate && styles.privacyOptionActive]}>
              <Text style={styles.privacyOptionIcon}>🛡️</Text>
              <Text style={styles.privacyOptionText}>Private</Text>
            </View>
          </View>
        </Card>

        {/* Fee Estimate */}
        {(feeEstimate || isLoadingFee) && (
          <Card style={styles.feeCard}>
            <Text style={styles.inputLabel}>Transaction Fee</Text>
            {isLoadingFee ? (
              <View style={styles.feeLoading}>
                <ActivityIndicator size="small" color={colors.primary} />
                <Text style={styles.feeLoadingText}>Calculating...</Text>
              </View>
            ) : feeEstimate ? (
              <View style={styles.feeDetails}>
                <View style={styles.feeRow}>
                  <Text style={styles.feeLabel}>Network Fee:</Text>
                  <Text style={styles.feeValue}>{formatCHML(feeEstimate.baseFee)} CHML</Text>
                </View>
                {formData.isPrivate && parseFloat(feeEstimate.priorityFee) > 0 && (
                  <View style={styles.feeRow}>
                    <Text style={styles.feeLabel}>Privacy Fee:</Text>
                    <Text style={styles.feeValue}>{formatCHML(feeEstimate.priorityFee)} CHML</Text>
                  </View>
                )}
                <View style={[styles.feeRow, styles.feeTotalRow]}>
                  <Text style={styles.feeTotalLabel}>Total Fee:</Text>
                  <Text style={styles.feeTotalValue}>{formatCHML(feeEstimate.totalFee)} CHML</Text>
                </View>
              </View>
            ) : null}
          </Card>
        )}

        {/* Memo Input */}
        <Card style={styles.inputCard}>
          <Text style={styles.inputLabel}>Memo (Optional)</Text>
          <TextInput
            style={styles.memoInput}
            value={formData.memo}
            onChangeText={(text) => setFormData(prev => ({ ...prev, memo: text }))}
            placeholder="Add a note for this transaction..."
            placeholderTextColor={colors.textTertiary}
            multiline
            maxLength={200}
          />
          <Text style={styles.memoCounter}>{formData.memo.length}/200</Text>
        </Card>

        {/* Send Button */}
        <Button
          title="Send CHML"
          onPress={handleSend}
          disabled={!formData.recipient || !formData.amount || isSubmitting}
          loading={isSubmitting}
          style={styles.sendButton}
        />
      </ScrollView>

      {/* Confirmation Modal */}
      <Modal
        visible={showConfirmModal}
        transparent
        animationType="fade"
        onRequestClose={() => setShowConfirmModal(false)}
      >
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <Text style={styles.modalTitle}>Confirm Transaction</Text>
            
            <View style={styles.confirmDetails}>
              <View style={styles.confirmRow}>
                <Text style={styles.confirmLabel}>To:</Text>
                <Text style={styles.confirmValue}>{truncateAddress(formData.recipient)}</Text>
              </View>
              
              <View style={styles.confirmRow}>
                <Text style={styles.confirmLabel}>Amount:</Text>
                <Text style={styles.confirmValue}>{formatCHML(formData.amount)} CHML</Text>
              </View>
              
              <View style={styles.confirmRow}>
                <Text style={styles.confirmLabel}>Fee:</Text>
                <Text style={styles.confirmValue}>
                  {feeEstimate ? formatCHML(feeEstimate.totalFee) : '...'} CHML
                </Text>
              </View>
              
              <View style={styles.confirmRow}>
                <Text style={styles.confirmLabel}>Type:</Text>
                <Text style={styles.confirmValue}>
                  {formData.isPrivate ? '🛡️ Private' : '🔓 Public'}
                </Text>
              </View>
              
              {formData.memo && (
                <View style={styles.confirmRow}>
                  <Text style={styles.confirmLabel}>Memo:</Text>
                  <Text style={styles.confirmValue}>{formData.memo}</Text>
                </View>
              )}
            </View>
            
            <View style={styles.modalButtons}>
              <Button
                title="Cancel"
                onPress={() => setShowConfirmModal(false)}
                variant="secondary"
                style={styles.modalButton}
              />
              <Button
                title="Confirm"
                onPress={handleConfirmSend}
                style={styles.modalButton}
              />
            </View>
          </View>
        </View>
      </Modal>
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
  
  balanceCard: {
    alignItems: 'center',
    marginBottom: spacing.lg,
  },
  
  balanceLabel: {
    fontSize: fonts.sm,
    fontFamily: fonts.medium,
    color: colors.textSecondary,
    marginBottom: spacing.xs,
  },
  
  balanceAmount: {
    fontSize: fonts['3xl'],
    fontFamily: fonts.bold,
    color: colors.text,
  },
  
  balanceUsd: {
    fontSize: fonts.base,
    fontFamily: fonts.regular,
    color: colors.textSecondary,
    marginTop: spacing.xs,
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
  
  inputContainer: {
    flexDirection: 'row',
    alignItems: 'flex-start',
  },
  
  textInput: {
    flex: 1,
    fontSize: fonts.base,
    fontFamily: fonts.regular,
    color: colors.text,
    backgroundColor: colors.surface,
    borderRadius: borderRadius.md,
    paddingHorizontal: spacing.md,
    paddingVertical: spacing.sm,
    borderWidth: 1,
    borderColor: colors.border,
    minHeight: 48,
    textAlignVertical: 'top',
  },
  
  inputError: {
    borderColor: colors.error,
  },
  
  pasteButton: {
    backgroundColor: colors.primary,
    paddingHorizontal: spacing.md,
    paddingVertical: spacing.sm,
    borderRadius: borderRadius.md,
    marginLeft: spacing.sm,
    height: 48,
    justifyContent: 'center',
  },
  
  pasteButtonText: {
    fontSize: fonts.sm,
    fontFamily: fonts.medium,
    color: colors.white,
  },
  
  amountContainer: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  
  amountInput: {
    flex: 1,
    fontSize: fonts['2xl'],
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
  
  amountSuffix: {
    flexDirection: 'row',
    alignItems: 'center',
    marginLeft: spacing.sm,
  },
  
  maxButton: {
    backgroundColor: colors.secondary,
    paddingHorizontal: spacing.sm,
    paddingVertical: spacing.xs,
    borderRadius: borderRadius.sm,
    marginRight: spacing.sm,
  },
  
  maxButtonText: {
    fontSize: fonts.xs,
    fontFamily: fonts.bold,
    color: colors.white,
  },
  
  currencyText: {
    fontSize: fonts.base,
    fontFamily: fonts.medium,
    color: colors.textSecondary,
  },
  
  privacyHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: spacing.md,
  },
  
  privacyDescription: {
    fontSize: fonts.xs,
    fontFamily: fonts.regular,
    color: colors.textSecondary,
    marginTop: spacing.xs,
  },
  
  toggle: {
    width: 50,
    height: 28,
    borderRadius: 14,
    backgroundColor: colors.border,
    padding: 2,
    justifyContent: 'center',
  },
  
  toggleActive: {
    backgroundColor: colors.primary,
  },
  
  toggleThumb: {
    width: 24,
    height: 24,
    borderRadius: 12,
    backgroundColor: colors.white,
  },
  
  toggleThumbActive: {
    alignSelf: 'flex-end',
  },
  
  privacyOptions: {
    flexDirection: 'row',
    gap: spacing.sm,
  },
  
  privacyOption: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    padding: spacing.sm,
    borderRadius: borderRadius.md,
    backgroundColor: colors.surface,
    borderWidth: 1,
    borderColor: colors.border,
  },
  
  privacyOptionActive: {
    borderColor: colors.primary,
    backgroundColor: colors.primary + '20',
  },
  
  privacyOptionIcon: {
    fontSize: 20,
    marginRight: spacing.sm,
  },
  
  privacyOptionText: {
    fontSize: fonts.sm,
    fontFamily: fonts.medium,
    color: colors.text,
  },
  
  feeCard: {
    marginBottom: spacing.lg,
  },
  
  feeLoading: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  
  feeLoadingText: {
    fontSize: fonts.sm,
    fontFamily: fonts.regular,
    color: colors.textSecondary,
    marginLeft: spacing.sm,
  },
  
  feeDetails: {
    gap: spacing.xs,
  },
  
  feeRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  
  feeLabel: {
    fontSize: fonts.sm,
    fontFamily: fonts.regular,
    color: colors.textSecondary,
  },
  
  feeValue: {
    fontSize: fonts.sm,
    fontFamily: fonts.medium,
    color: colors.text,
  },
  
  feeTotalRow: {
    borderTopWidth: 1,
    borderTopColor: colors.border,
    paddingTop: spacing.xs,
    marginTop: spacing.xs,
  },
  
  feeTotalLabel: {
    fontSize: fonts.sm,
    fontFamily: fonts.bold,
    color: colors.text,
  },
  
  feeTotalValue: {
    fontSize: fonts.sm,
    fontFamily: fonts.bold,
    color: colors.primary,
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
    minHeight: 80,
    textAlignVertical: 'top',
  },
  
  memoCounter: {
    fontSize: fonts.xs,
    fontFamily: fonts.regular,
    color: colors.textTertiary,
    textAlign: 'right',
    marginTop: spacing.xs,
  },
  
  sendButton: {
    marginTop: spacing.lg,
  },
  
  errorText: {
    fontSize: fonts.xs,
    fontFamily: fonts.regular,
    color: colors.error,
    marginTop: spacing.xs,
  },
  
  // Modal styles
  modalOverlay: {
    flex: 1,
    backgroundColor: colors.overlay,
    justifyContent: 'center',
    alignItems: 'center',
    padding: spacing.lg,
  },
  
  modalContent: {
    backgroundColor: colors.surface,
    borderRadius: borderRadius.lg,
    padding: spacing.lg,
    width: '100%',
    maxWidth: 400,
  },
  
  modalTitle: {
    fontSize: fonts.lg,
    fontFamily: fonts.bold,
    color: colors.text,
    textAlign: 'center',
    marginBottom: spacing.lg,
  },
  
  confirmDetails: {
    gap: spacing.md,
    marginBottom: spacing.lg,
  },
  
  confirmRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  
  confirmLabel: {
    fontSize: fonts.sm,
    fontFamily: fonts.medium,
    color: colors.textSecondary,
  },
  
  confirmValue: {
    fontSize: fonts.sm,
    fontFamily: fonts.medium,
    color: colors.text,
    textAlign: 'right',
    flex: 1,
    marginLeft: spacing.md,
  },
  
  modalButtons: {
    flexDirection: 'row',
    gap: spacing.md,
  },
  
  modalButton: {
    flex: 1,
  },
});
