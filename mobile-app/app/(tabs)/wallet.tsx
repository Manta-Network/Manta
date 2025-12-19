/**
 * Main Wallet screen for Chameleon Network
 * Updated with new light theme design system
 */

import React, { useEffect } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  ScrollView,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { useRouter } from 'expo-router';
import * as Clipboard from 'expo-clipboard';
import { Ionicons } from '@expo/vector-icons';
import { useWallet } from '../../context/WalletContext';
import { useBalance } from '../../hooks/useBalance';
import { useApi } from '../../hooks/useApi';
import { ScreenContainer } from '../../components/ScreenContainer';
import { MainHeader } from '../../components/MainHeader';
import { AccountCard } from '../../components/AccountCard';
import { ActionGrid } from '../../components/ActionGrid';
import { ActivityCard } from '../../components/ActivityCard';
import { NetworkBadge } from '../../components/NetworkBadge';
import { truncateAddress } from '../../utils/address';
import { THEME } from '../../constants/theme';

const WalletScreen = () => {
  const router = useRouter();
  const { wallet, isLoading: walletLoading, importDevAccount } = useWallet();
  const { connectionState, connect } = useApi();
  const { formattedFreeBalance, isLoading: balanceLoading, error: balanceError } = useBalance(
    wallet?.address
  );

  // Auto-connect to network on mount
  useEffect(() => {
    if (connectionState.status === 'disconnected') {
      connect();
    }
  }, [connectionState.status, connect]);

  const handleCopyAddress = async () => {
    if (wallet?.address) {
      await Clipboard.setStringAsync(wallet.address);
      Alert.alert('Copied!', 'Address copied to clipboard');
    }
  };

  const handleDevAccountImport = async (accountName) => {
    try {
      await importDevAccount(accountName);
    } catch (error) {
      Alert.alert('Error', 'Failed to import dev account');
    }
  };

  const renderNoWalletState = () => (
    <ScreenContainer scrollable showGradient>
      <View style={{ padding: THEME.spacing.lg }}>
        {/* Network Badge */}
        <View style={{ alignItems: 'flex-end', marginBottom: THEME.spacing.lg }}>
          <NetworkBadge />
        </View>

        {/* Header */}
        <View style={{ alignItems: 'center', marginBottom: THEME.spacing.xl * 2 }}>
          <Text style={{ fontSize: 64, marginBottom: THEME.spacing.md }}>🦎</Text>
          <Text style={{ 
            fontSize: THEME.fontSize['2xl'], 
            fontWeight: THEME.fontWeight.bold, 
            color: THEME.colors.text,
            marginBottom: THEME.spacing.sm 
          }}>
            Chameleon Wallet
          </Text>
          <Text style={{ 
            color: THEME.colors.textSecondary, 
            textAlign: 'center', 
            fontSize: THEME.fontSize.base,
            lineHeight: 24
          }}>
            Create a new wallet or{"\n"}import an existing one
          </Text>
        </View>

        {/* Main Actions */}
        <View style={{ marginBottom: THEME.spacing.xl }}>
          <TouchableOpacity
            style={{
              backgroundColor: THEME.colors.primary,
              borderRadius: THEME.borderRadius.medium,
              paddingVertical: THEME.spacing.md,
              paddingHorizontal: THEME.spacing.lg,
              marginBottom: THEME.spacing.md,
            }}
            onPress={() => router.push('/create-wallet')}
          >
            <Text style={{
              color: THEME.colors.white,
              textAlign: 'center',
              fontSize: THEME.fontSize.lg,
              fontWeight: THEME.fontWeight.semibold,
            }}>
              Create New Wallet
            </Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={{
              borderWidth: 2,
              borderColor: THEME.colors.primary,
              borderRadius: THEME.borderRadius.medium,
              paddingVertical: THEME.spacing.md,
              paddingHorizontal: THEME.spacing.lg,
            }}
            onPress={() => router.push('/import-wallet')}
          >
            <Text style={{
              color: THEME.colors.primary,
              textAlign: 'center',
              fontSize: THEME.fontSize.lg,
              fontWeight: THEME.fontWeight.semibold,
            }}>
              Import Wallet
            </Text>
          </TouchableOpacity>
        </View>

        {/* Dev Accounts Section */}
        <View style={{ alignItems: 'center' }}>
          <Text style={{ 
            color: THEME.colors.textSecondary, 
            fontSize: THEME.fontSize.sm, 
            marginBottom: THEME.spacing.md 
          }}>
            ── Or use dev account ──
          </Text>
          
          <View style={{ 
            flexDirection: 'row', 
            flexWrap: 'wrap', 
            justifyContent: 'center', 
            gap: THEME.spacing.sm 
          }}>
            {['alice', 'bob', 'charlie', 'dave', 'eve'].map((account) => (
              <TouchableOpacity
                key={account}
                style={{
                  backgroundColor: THEME.colors.card,
                  borderRadius: THEME.borderRadius.small,
                  paddingVertical: THEME.spacing.sm,
                  paddingHorizontal: THEME.spacing.md,
                  borderWidth: 1,
                  borderColor: THEME.colors.border,
                }}
                onPress={() => handleDevAccountImport(account)}
              >
                <Text style={{
                  color: THEME.colors.secondary,
                  fontSize: THEME.fontSize.sm,
                  fontWeight: THEME.fontWeight.medium,
                  textTransform: 'capitalize',
                }}>
                  {account}
                </Text>
              </TouchableOpacity>
            ))}
          </View>
        </View>
      </View>
    </ScreenContainer>
  );

  const renderWalletState = () => (
    <ScreenContainer scrollable showGradient>
      {/* Main Header */}
      <MainHeader 
        onNotificationPress={() => Alert.alert('Notifications', 'Coming soon!')}
      />

      {/* Account Card */}
      <AccountCard
        balance={balanceLoading ? '...' : formattedFreeBalance.split(' ')[0] || '0'}
        currency="USD"
        growthPercentage="+3.75%"
        isPositiveGrowth={true}
      />

      {/* Action Grid */}
      <ActionGrid 
        onActionPress={(actionId) => {
          console.log('Action pressed:', actionId);
        }}
      />

      {/* Recent Activity Section */}
      <View style={{ 
        marginHorizontal: THEME.spacing.md, 
        marginTop: THEME.spacing.lg 
      }}>
        <View style={{ 
          flexDirection: 'row', 
          justifyContent: 'space-between', 
          alignItems: 'center',
          marginBottom: THEME.spacing.md 
        }}>
          <Text style={{
            fontSize: THEME.fontSize.lg,
            fontWeight: THEME.fontWeight.semibold,
            color: THEME.colors.text,
          }}>
            Recent Activity
          </Text>
          <TouchableOpacity>
            <Text style={{
              fontSize: THEME.fontSize.sm,
              color: THEME.colors.secondary,
              fontWeight: THEME.fontWeight.medium,
            }}>
              View all →
            </Text>
          </TouchableOpacity>
        </View>

        {/* Activity Cards or Empty State */}
        {balanceError ? (
          <View style={{
            backgroundColor: THEME.colors.white,
            borderRadius: THEME.borderRadius.medium,
            padding: THEME.spacing.lg,
            alignItems: 'center',
            ...THEME.shadows.small,
          }}>
            <Ionicons name="warning-outline" size={48} color={THEME.colors.error} />
            <Text style={{
              color: THEME.colors.error,
              fontSize: THEME.fontSize.base,
              fontWeight: THEME.fontWeight.medium,
              marginTop: THEME.spacing.sm,
            }}>
              Connection Error
            </Text>
            <Text style={{
              color: THEME.colors.textSecondary,
              fontSize: THEME.fontSize.sm,
              textAlign: 'center',
              marginTop: THEME.spacing.xs,
            }}>
              {balanceError}
            </Text>
          </View>
        ) : (
          <>
            {/* Sample Activity Cards */}
            <ActivityCard
              type="deposit"
              transactionId="6671a2b3c4d5e6f7890123456789abcd"
              amount="12,000.00"
              status="success"
              date="2 hours ago"
            />
            <ActivityCard
              type="withdrawal"
              transactionId="6672b3c4d5e6f7890123456789abcdef"
              amount="1,000.00"
              status="pending"
              date="1 day ago"
            />
            
            {/* Empty State for More Transactions */}
            <View style={{
              backgroundColor: THEME.colors.white,
              borderRadius: THEME.borderRadius.medium,
              padding: THEME.spacing.lg,
              alignItems: 'center',
              marginTop: THEME.spacing.sm,
              ...THEME.shadows.small,
            }}>
              <Ionicons name="time-outline" size={32} color={THEME.colors.textMuted} />
              <Text style={{
                color: THEME.colors.textSecondary,
                fontSize: THEME.fontSize.sm,
                marginTop: THEME.spacing.sm,
              }}>
                More transactions will appear here
              </Text>
            </View>
          </>
        )}
      </View>

      {/* Bottom Spacing */}
      <View style={{ height: THEME.spacing.xl }} />
    </ScreenContainer>
  );

  if (walletLoading) {
    return (
      <ScreenContainer showGradient>
        <View style={{ 
          flex: 1, 
          justifyContent: 'center', 
          alignItems: 'center' 
        }}>
          <ActivityIndicator size="large" color={THEME.colors.primary} />
          <Text style={{
            color: THEME.colors.text,
            fontSize: THEME.fontSize.lg,
            marginTop: THEME.spacing.md,
          }}>
            Loading wallet...
          </Text>
        </View>
      </ScreenContainer>
    );
  }

  return wallet ? renderWalletState() : renderNoWalletState();
};

export default WalletScreen;
