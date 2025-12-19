/**
 * DEVNET indicator component with connection status
 * Updated for new light theme design
 */

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { useApi } from '../hooks/useApi';
import { NETWORK_CONFIG } from '../config/network';
import { THEME } from '../constants/theme';

interface NetworkBadgeProps {
  size?: 'small' | 'medium' | 'large';
  showConnectionStatus?: boolean;
  style?: any;
}

export function NetworkBadge({ 
  size = 'medium', 
  showConnectionStatus = false,
  style 
}: NetworkBadgeProps) {
  const { isConnected, isConnecting, connectionState } = useApi();

  const getConnectionColor = () => {
    if (isConnecting) return THEME.colors.warning;
    if (isConnected) return THEME.colors.success;
    return THEME.colors.error;
  };

  const badgeStyle = [
    styles.badge,
    size === 'small' && styles.badgeSmall,
    size === 'large' && styles.badgeLarge,
    style,
  ];

  const textStyle = [
    styles.badgeText,
    size === 'small' && styles.badgeTextSmall,
    size === 'large' && styles.badgeTextLarge,
  ];

  return (
    <View style={badgeStyle}>
      <Text style={textStyle}>
        {NETWORK_CONFIG.isTestnet ? 'DEVNET' : NETWORK_CONFIG.name}
      </Text>
      
      {showConnectionStatus && (
        <>
          <View style={styles.separator} />
          <View style={[styles.statusDot, { backgroundColor: getConnectionColor() }]} />
        </>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  badge: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: THEME.colors.primary,
    paddingHorizontal: THEME.spacing.sm,
    paddingVertical: THEME.spacing.xs,
    borderRadius: THEME.borderRadius.full,
  },
  badgeSmall: {
    paddingHorizontal: THEME.spacing.xs,
    paddingVertical: 2,
  },
  badgeLarge: {
    paddingHorizontal: THEME.spacing.md,
    paddingVertical: THEME.spacing.sm,
  },
  badgeText: {
    fontSize: THEME.fontSize.xs,
    fontWeight: THEME.fontWeight.bold,
    color: THEME.colors.white,
  },
  badgeTextSmall: {
    fontSize: 10,
  },
  badgeTextLarge: {
    fontSize: THEME.fontSize.sm,
  },
  separator: {
    width: 1,
    height: 12,
    backgroundColor: 'rgba(255,255,255,0.3)',
    marginHorizontal: THEME.spacing.xs,
  },
  statusDot: {
    width: 6,
    height: 6,
    borderRadius: 3,
  },
});

/**
 * Compact version for headers
 */
export function NetworkBadgeCompact() {
  return (
    <NetworkBadge 
      size="small" 
      showConnectionStatus={false}
    />
  );
}
