/**
 * Send CHML screen for Chameleon Network
 * Updated with new light theme design
 */

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  ScrollView,
  Alert,
  ActivityIndicator,
  Modal,
  StyleSheet,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useRouter } from 'expo-router';
import * as Clipboard from 'expo-clipboard';
import { Ionicons } from '@expo/vector-icons';
import { BN } from '@polkadot/util';
import { useWallet } from '../context/WalletContext';
import { useBalance } from '../hooks/useBalance';
import { transactionService, type TransactionResult, type FeeEstimate } from '../services/transaction';
import { chainService } from '../services/chain';
import { walletService } from '../services/wallet';
import { TransactionStatus } from '../components/TransactionStatus';
import { ScreenContainer } from '../components/ScreenContainer';
import { truncateAddress } from '../utils/address';
import { THEME } from '../constants/theme';

const SendScreen = () => {
  const router = useRouter();
  const { wallet } = useWallet();
  const { formattedFreeBalance, balance } = useBalance(wallet?.address);
  
  // Form state
  const [recipient, setRecipient] = useState('');
  const [amount, setAmount] = useState('');
  const [isValidAddress, setIsValidAddress] = useState(false);
  const [feeEstimate, setFeeEstimate] = useState<FeeEstimate | null>(null);
  const [isEstimatingFee, setIsEstimatingFee] = useState(false);
  
  // Transaction state
  const [showConfirmation, setShowConfirmation] = useState(false);
  const [isSending, setIsSending] = useState(false);
  const [transactionResult, setTransactionResult] = useState<TransactionResult | null>(null);
  const [showResult, setShowResult] = useState(false);

  // Validate recipient address
  useEffect(() => {
    if (recipient.length > 0) {
      const valid = transactionService.validateAddress(recipient);
      setIsValidAddress(valid);
    } else {
      setIsValidAddress(false);
    }
  }, [recipient]);

  // Estimate fee when amount and recipient change
  useEffect(() => {
    if (isValidAddress && amount && wallet?.address && parseFloat(amount) > 0) {
      estimateFee();
    } else {
      setFeeEstimate(null);
    }
  }, [recipient, amount, isValidAddress, wallet?.address]);

  const estimateFee = async () => {
    if (!wallet?.address || !isValidAddress || !amount) return;
    
    setIsEstimatingFee(true);
    try {
      const amountBN = transactionService.parseAmount(amount);
      const estimate = await transactionService.estimateFee(wallet.address, recipient, amountBN);
      setFeeEstimate(estimate);
    } catch (error) {
      console.error('Error estimating fee:', error);
      setFeeEstimate(null);
    } finally {
      setIsEstimatingFee(false);
    }
  };

  const handlePasteAddress = async () => {
    try {
      const clipboardContent = await Clipboard.getStringAsync();
      if (clipboardContent) {
        setRecipient(clipboardContent.trim());
      }
    } catch (error) {
      Alert.alert('Error', 'Failed to paste from clipboard');
    }
  };

  const handleScanQR = () => {
    // Placeholder for QR scanner
    Alert.alert('Coming Soon', 'QR code scanning will be available in a future update');
  };

  const handleMaxAmount = () => {
    if (!balance || !feeEstimate) return;
    
    const balanceBN = new BN(balance.free);
    const feeBN = new BN(feeEstimate.partialFee);
    const maxAmount = balanceBN.sub(feeBN);
    
    if (maxAmount.gt(new BN(0))) {
      const formatted = chainService.formatBalance(maxAmount.toString()).split(' ')[0];
      setAmount(formatted.replace(/,/g, ''));
    }
  };

  const validateTransaction = (): string | null => {
    if (!wallet?.address) return 'No wallet connected';
    if (!recipient) return 'Please enter recipient address';
    if (!isValidAddress) return 'Invalid recipient address';
    if (!amount || parseFloat(amount) <= 0) return 'Please enter a valid amount';
    if (!balance) return 'Unable to check balance';
    if (!feeEstimate) return 'Unable to estimate fee';
    
    const amountBN = transactionService.parseAmount(amount);
    const feeBN = new BN(feeEstimate.partialFee);
    const totalBN = amountBN.add(feeBN);
    const balanceBN = new BN(balance.free);
    
    if (totalBN.gt(balanceBN)) {
      return 'Insufficient balance (including fees)';
    }
    
    return null;
  };

  const handleReview = () => {
    const error = validateTransaction();
    if (error) {
      Alert.alert('Transaction Error', error);
      return;
    }
    
    setShowConfirmation(true);
  };

  const handleConfirmSend = async () => {
    if (!wallet?.address || !feeEstimate) return;
    
    setShowConfirmation(false);
    setIsSending(true);
    
    try {
      const keyPair = walletService.getKeyPair();
      if (!keyPair) {
        throw new Error('Wallet not unlocked');
      }
      
      const amountBN = transactionService.parseAmount(amount);
      
      // Send transaction with monitoring
      await transactionService.sendTransactionWithMonitoring(
        keyPair,
        recipient,
        amountBN,
        (result) => {
          setTransactionResult(result);
          if (result.status === 'finalized' || result.status === 'failed') {
            setIsSending(false);
            setShowResult(true);
          }
        }
      );
    } catch (error) {
      console.error('Error sending transaction:', error);
      setIsSending(false);
      Alert.alert(
        'Transaction Failed',
        error instanceof Error ? error.message : 'Unknown error occurred'
      );
    }
  };

  const handleCloseResult = () => {
    setShowResult(false);
    setTransactionResult(null);
    
    // Reset form if transaction was successful
    if (transactionResult?.status === 'finalized') {
      setRecipient('');
      setAmount('');
      setFeeEstimate(null);
    }
  };

  const renderConfirmationModal = () => {
    if (!feeEstimate) return null;
    
    const amountBN = transactionService.parseAmount(amount);
    const feeBN = new BN(feeEstimate.partialFee);
    const totalBN = amountBN.add(feeBN);
    
    return (
      <Modal
        visible={showConfirmation}
        transparent
        animationType="slide"
        onRequestClose={() => setShowConfirmation(false)}
      >
        <View className="flex-1 bg-black/50 justify-end">
          <View className="bg-[#0C0E12] rounded-t-3xl p-6">
            <Text className="text-white text-xl font-bold text-center mb-6">
              Confirm Transaction
            </Text>
            
            {/* Amount */}
            <View className="mb-6">
              <Text className="text-[#CDCDE0] text-sm mb-2">Sending</Text>
              <Text className="text-white text-3xl font-bold">
                {amount} CHML
              </Text>
            </View>
            
            {/* Recipient */}
            <View className="mb-6">
              <Text className="text-[#CDCDE0] text-sm mb-2">To</Text>
              <Text className="text-white font-mono text-base">
                {truncateAddress(recipient)}
              </Text>
            </View>
            
            {/* Fee */}
            <View className="mb-6">
              <Text className="text-[#CDCDE0] text-sm mb-2">Network Fee</Text>
              <Text className="text-white text-base">
                {feeEstimate.formatted}
              </Text>
            </View>
            
            {/* Divider */}
            <View className="border-t border-[#333] my-4" />
            
            {/* Total */}
            <View className="mb-8">
              <Text className="text-[#CDCDE0] text-sm mb-2">Total</Text>
              <Text className="text-white text-xl font-bold">
                {chainService.formatBalance(totalBN.toString())}
              </Text>
            </View>
            
            {/* Buttons */}
            <View className="gap-4">
              <TouchableOpacity
                className="bg-[#18BB59] rounded-xl py-4 px-6"
                onPress={handleConfirmSend}
              >
                <Text className="text-white text-center text-lg font-semibold">
                  Confirm & Send
                </Text>
              </TouchableOpacity>
              
              <TouchableOpacity
                className="py-4 px-6"
                onPress={() => setShowConfirmation(false)}
              >
                <Text className="text-[#CDCDE0] text-center text-lg">
                  Cancel
                </Text>
              </TouchableOpacity>
            </View>
          </View>
        </View>
      </Modal>
    );
  };

  const renderTransactionResult = () => {
    if (!transactionResult) return null;
    
    return (
      <Modal
        visible={showResult}
        transparent
        animationType="slide"
        onRequestClose={handleCloseResult}
      >
        <View className="flex-1 bg-black/50 justify-center">
          <TransactionStatus
            result={transactionResult}
            onClose={handleCloseResult}
          />
        </View>
      </Modal>
    );
  };

  if (isSending) {
    return (
      <View className="flex-1 bg-[#0C0E12] justify-center items-center">
        <ActivityIndicator size="large" color="#18BB59" />
        <Text className="text-white text-lg mt-4">Sending Transaction...</Text>
        <Text className="text-[#CDCDE0] text-sm mt-2 text-center px-8">
          Please wait while your transaction is processed
        </Text>
      </View>
    );
  }

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
          <Text style={styles.headerTitle}>Send CHML</Text>
        </View>

        <View style={styles.content}>
          {/* Recipient Input */}
          <View style={styles.inputSection}>
            <Text style={styles.inputLabel}>To</Text>
            <View style={styles.inputRow}>
              <View style={[styles.textInputContainer, { flex: 1, marginRight: THEME.spacing.sm }]}>
                <TextInput
                  style={styles.textInput}
                  placeholder="Enter recipient address..."
                  placeholderTextColor={THEME.colors.textMuted}
                  value={recipient}
                  onChangeText={setRecipient}
                  multiline
                  autoCapitalize="none"
                  autoCorrect={false}
                />
              </View>
              <TouchableOpacity
                style={[styles.iconButton, { marginRight: THEME.spacing.xs }]}
                onPress={handlePasteAddress}
              >
                <Ionicons name="clipboard-outline" size={20} color={THEME.colors.textSecondary} />
              </TouchableOpacity>
              <TouchableOpacity
                style={styles.iconButton}
                onPress={handleScanQR}
              >
                <Ionicons name="qr-code-outline" size={20} color={THEME.colors.textSecondary} />
              </TouchableOpacity>
            </View>
            {recipient.length > 0 && !isValidAddress && (
              <Text style={styles.errorText}>Invalid address format</Text>
            )}
          </View>

          {/* Amount Input */}
          <View style={styles.inputSection}>
            <Text style={styles.inputLabel}>Amount</Text>
            <View style={styles.inputRow}>
              <View style={[styles.textInputContainer, { flex: 1, marginRight: THEME.spacing.sm }]}>
                <TextInput
                  style={[styles.textInput, { fontSize: THEME.fontSize.lg }]}
                  placeholder="0.00"
                  placeholderTextColor={THEME.colors.textMuted}
                  value={amount}
                  onChangeText={setAmount}
                  keyboardType="decimal-pad"
                />
              </View>
              <TouchableOpacity
                style={[
                  styles.maxButton,
                  (!balance || !feeEstimate) && styles.maxButtonDisabled
                ]}
                onPress={handleMaxAmount}
                disabled={!balance || !feeEstimate}
              >
                <Text style={styles.maxButtonText}>MAX</Text>
              </TouchableOpacity>
            </View>
            <Text style={styles.availableText}>
              Available: {formattedFreeBalance}
            </Text>
          </View>

          {/* Fee Estimate */}
          <View style={styles.inputSection}>
            <Text style={styles.inputLabel}>Network Fee</Text>
            <View style={styles.feeContainer}>
              {isEstimatingFee ? (
                <View style={styles.feeEstimating}>
                  <ActivityIndicator size="small" color={THEME.colors.primary} />
                  <Text style={styles.feeEstimatingText}>Estimating fee...</Text>
                </View>
              ) : feeEstimate ? (
                <Text style={styles.feeAmount}>
                  ~{feeEstimate.formatted}
                </Text>
              ) : (
                <Text style={styles.feePlaceholder}>
                  Enter amount to estimate fee
                </Text>
              )}
            </View>
          </View>

          {/* Review Button */}
          <TouchableOpacity
            style={[
              styles.reviewButton,
              validateTransaction() && styles.reviewButtonDisabled
            ]}
            onPress={handleReview}
            disabled={!!validateTransaction()}
          >
            <Text style={[
              styles.reviewButtonText,
              validateTransaction() && styles.reviewButtonTextDisabled
            ]}>
              Review Send
            </Text>
          </TouchableOpacity>
        </View>
      </SafeAreaView>

      {renderConfirmationModal()}
      {renderTransactionResult()}
    </ScrollView>
  );
};

export default SendScreen;
