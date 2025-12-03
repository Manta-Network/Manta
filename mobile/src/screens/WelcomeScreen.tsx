/**
 * Chameleon Wallet - Welcome Screen
 */

import React from 'react';
import {
  View,
  Text,
  StyleSheet,
  SafeAreaView,
  Image,
  StatusBar,
} from 'react-native';
import { Button } from '@/components/Button';
import { colors, fonts, spacing } from '@/theme';
import type { NativeStackScreenProps } from '@react-navigation/native-stack';
import type { RootStackParamList } from '@/types';

type Props = NativeStackScreenProps<RootStackParamList, 'Welcome'>;

export function WelcomeScreen({ navigation }: Props) {
  const handleCreateWallet = () => {
    navigation.navigate('CreateWallet');
  };

  const handleImportWallet = () => {
    navigation.navigate('ImportWallet');
  };

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="light-content" backgroundColor={colors.background} />
      
      <View style={styles.content}>
        {/* Logo and branding */}
        <View style={styles.logoContainer}>
          <View style={styles.logoPlaceholder}>
            <Text style={styles.logoText}>🦎</Text>
          </View>
          <Text style={styles.title}>Chameleon Wallet</Text>
          <Text style={styles.subtitle}>
            Your gateway to private, secure blockchain transactions
          </Text>
        </View>

        {/* Features */}
        <View style={styles.featuresContainer}>
          <View style={styles.feature}>
            <Text style={styles.featureIcon}>🔒</Text>
            <Text style={styles.featureTitle}>Privacy First</Text>
            <Text style={styles.featureDescription}>
              Shield your transactions with zero-knowledge proofs
            </Text>
          </View>

          <View style={styles.feature}>
            <Text style={styles.featureIcon}>⚡</Text>
            <Text style={styles.featureTitle}>Lightning Fast</Text>
            <Text style={styles.featureDescription}>
              6-second block times with instant confirmations
            </Text>
          </View>

          <View style={styles.feature}>
            <Text style={styles.featureIcon}>🛡️</Text>
            <Text style={styles.featureTitle}>MEV Protected</Text>
            <Text style={styles.featureDescription}>
              No front-running or sandwich attacks
            </Text>
          </View>
        </View>
      </View>

      {/* Action buttons */}
      <View style={styles.actions}>
        <Button
          title="Create New Wallet"
          onPress={handleCreateWallet}
          variant="primary"
          size="large"
          fullWidth
        />
        
        <Button
          title="Import Existing Wallet"
          onPress={handleImportWallet}
          variant="outline"
          size="large"
          fullWidth
          style={styles.importButton}
        />
      </View>

      {/* Footer */}
      <View style={styles.footer}>
        <Text style={styles.footerText}>
          By continuing, you agree to our Terms of Service and Privacy Policy
        </Text>
      </View>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: colors.background,
  },
  
  content: {
    flex: 1,
    paddingHorizontal: spacing.lg,
    justifyContent: 'center',
  },
  
  logoContainer: {
    alignItems: 'center',
    marginBottom: spacing['3xl'],
  },
  
  logoPlaceholder: {
    width: 120,
    height: 120,
    borderRadius: 60,
    backgroundColor: colors.surface,
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: spacing.lg,
  },
  
  logoText: {
    fontSize: 60,
  },
  
  title: {
    fontSize: fonts['3xl'],
    fontFamily: fonts.bold,
    color: colors.text,
    marginBottom: spacing.sm,
    textAlign: 'center',
  },
  
  subtitle: {
    fontSize: fonts.lg,
    fontFamily: fonts.regular,
    color: colors.textSecondary,
    textAlign: 'center',
    lineHeight: 24,
  },
  
  featuresContainer: {
    gap: spacing.lg,
  },
  
  feature: {
    alignItems: 'center',
    paddingHorizontal: spacing.md,
  },
  
  featureIcon: {
    fontSize: 32,
    marginBottom: spacing.sm,
  },
  
  featureTitle: {
    fontSize: fonts.lg,
    fontFamily: fonts.medium,
    color: colors.text,
    marginBottom: spacing.xs,
    textAlign: 'center',
  },
  
  featureDescription: {
    fontSize: fonts.sm,
    fontFamily: fonts.regular,
    color: colors.textSecondary,
    textAlign: 'center',
    lineHeight: 20,
  },
  
  actions: {
    paddingHorizontal: spacing.lg,
    paddingBottom: spacing.lg,
    gap: spacing.md,
  },
  
  importButton: {
    marginTop: spacing.sm,
  },
  
  footer: {
    paddingHorizontal: spacing.lg,
    paddingBottom: spacing.md,
  },
  
  footerText: {
    fontSize: fonts.xs,
    fontFamily: fonts.regular,
    color: colors.textTertiary,
    textAlign: 'center',
    lineHeight: 16,
  },
});
