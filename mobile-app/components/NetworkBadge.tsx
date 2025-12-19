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
  showConnectionStatus = true,
  style 
}: NetworkBadgeProps) {
  const { isConnected, isConnecting, connectionState } = useApi();

  const getSizeStyles = () => {
    switch (size) {
      case 'small':
        return {
          container: 'px-2 py-1',
          text: 'text-xs',
          dot: 'w-2 h-2',
        };
      case 'large':
        return {
          container: 'px-4 py-2',
          text: 'text-base',
          dot: 'w-3 h-3',
        };
      default: // medium
        return {
          container: 'px-3 py-1.5',
          text: 'text-sm',
          dot: 'w-2.5 h-2.5',
        };
    }
  };

  const getConnectionColor = () => {
    if (isConnecting) return 'bg-yellow-500';
    if (isConnected) return 'bg-green-500';
    return 'bg-red-500';
  };

  const getConnectionText = () => {
    if (isConnecting) return 'Connecting...';
    if (isConnected) return `Block #${connectionState.blockNumber || '---'}`;
    return 'Disconnected';
  };

  const sizeStyles = getSizeStyles();

  return (
    <View 
      className={`
        flex-row items-center 
        bg-gradient-to-r from-teal-600 to-teal-500
        rounded-full 
        ${sizeStyles.container}
      `}
      style={style}
    >
      {/* DEVNET Badge */}
      <Text className={`font-bold text-white ${sizeStyles.text}`}>
        {NETWORK_CONFIG.isTestnet ? 'DEVNET' : NETWORK_CONFIG.name}
      </Text>
      
      {/* Connection Status */}
      {showConnectionStatus && (
        <>
          <View className="mx-2 w-px h-4 bg-white/30" />
          
          <View className="flex-row items-center">
            {/* Status Dot */}
            <View 
              className={`
                ${sizeStyles.dot} 
                rounded-full 
                ${getConnectionColor()}
                mr-1.5
              `}
            />
            
            {/* Status Text */}
            <Text className={`text-white/90 ${sizeStyles.text}`}>
              {getConnectionText()}
            </Text>
          </View>
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
