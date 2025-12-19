/**
 * Receive CHML screen for Chameleon Network
 * Updated with new light theme design
 */

import React from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  ScrollView,
  Alert,
  Share,
  StyleSheet,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useRouter } from 'expo-router';
import * as Clipboard from 'expo-clipboard';
import { Ionicons } from '@expo/vector-icons';
import { useWallet } from '../context/WalletContext';
import { QRCode } from '../components/QRCode';
import { ScreenContainer } from '../components/ScreenContainer';
import { truncateAddress } from '../utils/address';
import { THEME } from '../constants/theme';

const ReceiveScreen = () => {
  const router = useRouter();
  const { wallet } = useWallet();

  if (!wallet?.address) {
    return (
      <ScreenContainer showGradient={false}>
        <View style={styles.noWalletContainer}>
          <Ionicons name="wallet-outline" size={64} color={THEME.colors.textMuted} />
          <Text style={styles.noWalletTitle}>No Wallet Found</Text>
          <Text style={styles.noWalletSubtitle}>
            Please create or import a wallet first
          </Text>
          <TouchableOpacity
            style={styles.goBackButton}
            onPress={() => router.back()}
          >
            <Text style={styles.goBackButtonText}>Go Back</Text>
          </TouchableOpacity>
        </View>
      </ScreenContainer>
    );
  }

  const handleCopyAddress = async () => {
    try {
      await Clipboard.setStringAsync(wallet.address);
      Alert.alert('Copied!', 'Address copied to clipboard');
    } catch (error) {
      Alert.alert('Error', 'Failed to copy address');
    }
  };

  const handleShareAddress = async () => {
    try {
      await Share.share({
        message: `My Chameleon Network address: ${wallet.address}`,
        title: 'Chameleon Wallet Address',
      });
    } catch (error) {
      console.error('Error sharing address:', error);
    }
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
          <Text style={styles.headerTitle}>Receive CHML</Text>
        </View>

        <View style={styles.content}>
          {/* QR Code Section */}
          <View style={styles.qrSection}>
            <View style={styles.qrContainer}>
              <QRCode
                value={wallet.address}
                size={200}
                backgroundColor={THEME.colors.white}
                color={THEME.colors.primary}
              />
            </View>
            
            <Text style={styles.qrDescription}>
              Scan this QR code to get my address
            </Text>
          </View>

          {/* Address Section */}
          <View style={styles.addressSection}>
            <Text style={styles.addressTitle}>
              Your CHML Address
            </Text>
            
            <View style={styles.addressContainer}>
              <Text style={styles.addressText}>
                {wallet.address}
              </Text>
            </View>
            
            {/* Action Buttons */}
            <View style={styles.buttonRow}>
              <TouchableOpacity
                style={[styles.actionButton, styles.copyButton]}
                onPress={handleCopyAddress}
              >
                <View style={styles.buttonContent}>
                  <Ionicons name="copy-outline" size={20} color={THEME.colors.white} />
                  <Text style={styles.copyButtonText}>Copy</Text>
              </View>
            </TouchableOpacity>
            
              <TouchableOpacity
                style={[styles.actionButton, styles.shareButton]}
                onPress={handleShareAddress}
              >
                <View style={styles.buttonContent}>
                  <Ionicons name="share-outline" size={20} color={THEME.colors.primary} />
                  <Text style={styles.shareButtonText}>Share</Text>
                </View>
              </TouchableOpacity>
            </View>
          </View>

          {/* Warning Section */}
          <View style={styles.warningSection}>
            <View style={styles.warningContent}>
              <Ionicons name="warning" size={20} color={THEME.colors.error} />
              <View style={styles.warningTextContainer}>
                <Text style={styles.warningTitle}>
                  ⚠️ DEVNET ADDRESS
                </Text>
                <Text style={styles.warningDescription}>
                  This is a development network address. Only send test CHML tokens to this address.
                </Text>
              </View>
            </View>
          </View>

          {/* Tips Section */}
          <View style={styles.tipsSection}>
            <Text style={styles.tipsTitle}>💡 Tips</Text>
            
            <View style={styles.tipsList}>
              <View style={styles.tipItem}>
                <Text style={styles.tipBullet}>•</Text>
                <Text style={styles.tipText}>
                  Only send CHML tokens to this address
              </Text>
            </View>
            
              <View style={styles.tipItem}>
                <Text style={styles.tipBullet}>•</Text>
                <Text style={styles.tipText}>
                  Double-check the address before sharing
                </Text>
              </View>
              
              <View style={styles.tipItem}>
                <Text style={styles.tipBullet}>•</Text>
                <Text style={styles.tipText}>
                  Transactions on devnet are for testing only
                </Text>
              </View>
              
              <View style={styles.tipItem}>
                <Text style={styles.tipBullet}>•</Text>
                <Text style={styles.tipText}>
                  Your address is public and safe to share
                </Text>
              </View>
            </View>
          </View>

          {/* Additional Info */}
          <View style={styles.footerInfo}>
            <Text style={styles.footerText}>
              Powered by Chameleon Network
            </Text>
          </View>
        </View>
      </SafeAreaView>
    </ScreenContainer>
  );
};

const styles = StyleSheet.create({
  noWalletContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: THEME.spacing.lg,
  },
  noWalletTitle: {
    fontSize: THEME.fontSize.xl,
    fontWeight: THEME.fontWeight.bold,
    color: THEME.colors.text,
    marginTop: THEME.spacing.md,
    marginBottom: THEME.spacing.sm,
  },
  noWalletSubtitle: {
    color: THEME.colors.textSecondary,
    textAlign: 'center',
    fontSize: THEME.fontSize.base,
  },
  goBackButton: {
    backgroundColor: THEME.colors.primary,
    borderRadius: THEME.borderRadius.medium,
    paddingVertical: THEME.spacing.sm,
    paddingHorizontal: THEME.spacing.lg,
    marginTop: THEME.spacing.lg,
  },
  goBackButtonText: {
    color: THEME.colors.white,
    fontWeight: THEME.fontWeight.semibold,
  },
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
  qrSection: {
    alignItems: 'center',
    marginBottom: THEME.spacing.xl,
  },
  qrContainer: {
    backgroundColor: THEME.colors.white,
    padding: THEME.spacing.md,
    borderRadius: THEME.borderRadius.large,
    marginBottom: THEME.spacing.md,
    ...THEME.shadows.medium,
  },
  qrDescription: {
    color: THEME.colors.textSecondary,
    fontSize: THEME.fontSize.sm,
    textAlign: 'center',
  },
  addressSection: {
    marginBottom: THEME.spacing.lg,
  },
  addressTitle: {
    fontSize: THEME.fontSize.lg,
    fontWeight: THEME.fontWeight.semibold,
    color: THEME.colors.text,
    textAlign: 'center',
    marginBottom: THEME.spacing.md,
  },
  addressContainer: {
    backgroundColor: THEME.colors.white,
    borderRadius: THEME.borderRadius.medium,
    padding: THEME.spacing.md,
    marginBottom: THEME.spacing.md,
    borderWidth: 1,
    borderColor: THEME.colors.border,
    ...THEME.shadows.small,
  },
  addressText: {
    color: THEME.colors.text,
    fontSize: THEME.fontSize.sm,
    textAlign: 'center',
    lineHeight: 20,
    fontFamily: 'monospace',
  },
  buttonRow: {
    flexDirection: 'row',
    gap: THEME.spacing.md,
  },
  actionButton: {
    flex: 1,
    borderRadius: THEME.borderRadius.medium,
    paddingVertical: THEME.spacing.md,
    paddingHorizontal: THEME.spacing.lg,
  },
  copyButton: {
    backgroundColor: THEME.colors.primary,
  },
  shareButton: {
    borderWidth: 2,
    borderColor: THEME.colors.primary,
    backgroundColor: 'transparent',
  },
  buttonContent: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
  },
  copyButtonText: {
    color: THEME.colors.white,
    fontWeight: THEME.fontWeight.semibold,
    marginLeft: THEME.spacing.xs,
  },
  shareButtonText: {
    color: THEME.colors.primary,
    fontWeight: THEME.fontWeight.semibold,
    marginLeft: THEME.spacing.xs,
  },
  warningSection: {
    backgroundColor: THEME.colors.errorBg,
    borderWidth: 1,
    borderColor: THEME.colors.error,
    borderRadius: THEME.borderRadius.medium,
    padding: THEME.spacing.md,
    marginBottom: THEME.spacing.md,
  },
  warningContent: {
    flexDirection: 'row',
    alignItems: 'flex-start',
  },
  warningTextContainer: {
    flex: 1,
    marginLeft: THEME.spacing.sm,
  },
  warningTitle: {
    color: THEME.colors.error,
    fontWeight: THEME.fontWeight.semibold,
    marginBottom: THEME.spacing.xs,
  },
  warningDescription: {
    color: THEME.colors.textSecondary,
    fontSize: THEME.fontSize.sm,
    lineHeight: 18,
  },
  tipsSection: {
    backgroundColor: THEME.colors.white,
    borderRadius: THEME.borderRadius.medium,
    padding: THEME.spacing.md,
    ...THEME.shadows.small,
  },
  tipsTitle: {
    color: THEME.colors.text,
    fontWeight: THEME.fontWeight.semibold,
    marginBottom: THEME.spacing.sm,
  },
  tipsList: {
    gap: THEME.spacing.sm,
  },
  tipItem: {
    flexDirection: 'row',
    alignItems: 'flex-start',
  },
  tipBullet: {
    color: THEME.colors.primary,
    marginRight: THEME.spacing.xs,
    fontSize: THEME.fontSize.sm,
  },
  tipText: {
    color: THEME.colors.textSecondary,
    fontSize: THEME.fontSize.sm,
    flex: 1,
    lineHeight: 18,
  },
  footerInfo: {
    alignItems: 'center',
    paddingVertical: THEME.spacing.md,
  },
  footerText: {
    color: THEME.colors.textMuted,
    fontSize: THEME.fontSize.xs,
    textAlign: 'center',
  },
});

export default ReceiveScreen;
