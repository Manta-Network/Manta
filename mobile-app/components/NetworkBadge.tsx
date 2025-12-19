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

/**
 * Compact version for headers
 */
export function NetworkBadgeCompact() {
  return (
    <NetworkBadge 
      size="small" 
      showConnectionStatus={false}
      style={{ alignSelf: 'flex-start' }}
    />
  );
}

/**
 * Full version with connection details
 */
export function NetworkBadgeFull() {
  const { connectionState, error } = useApi();
  
  return (
    <View className="bg-gray-900/50 rounded-lg p-3 m-2">
      <NetworkBadge size="medium" showConnectionStatus={true} />
      
      {/* Additional Network Info */}
      <View className="mt-2 space-y-1">
        <Text className="text-xs text-gray-400">
          Network: {NETWORK_CONFIG.name}
        </Text>
        <Text className="text-xs text-gray-400">
          Token: {NETWORK_CONFIG.tokenSymbol} ({NETWORK_CONFIG.tokenDecimals} decimals)
        </Text>
        
        {error && (
          <Text className="text-xs text-red-400">
            Error: {error}
          </Text>
        )}
      </View>
    </View>
  );
}
