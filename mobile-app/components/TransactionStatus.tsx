/**
 * Transaction status component with animated states
 */

import React from 'react';
import { View, Text, ActivityIndicator, TouchableOpacity } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import type { TransactionStatus as TxStatus, TransactionResult } from '../services/transaction';

interface TransactionStatusProps {
  result: TransactionResult;
  onClose?: () => void;
  showExplorerLink?: boolean;
}

export const TransactionStatus: React.FC<TransactionStatusProps> = ({
  result,
  onClose,
  showExplorerLink = true,
}) => {
  const getStatusConfig = (status: TxStatus) => {
    switch (status) {
      case 'pending':
        return {
          icon: <ActivityIndicator size="large" color="#FFB800" />,
          title: 'Transaction Pending',
          subtitle: 'Broadcasting to network...',
          color: '#FFB800',
          bgColor: '#2A2A2A',
        };
      case 'inBlock':
        return {
          icon: <Ionicons name="time-outline" size={48} color="#13E1BC" />,
          title: 'Transaction In Block',
          subtitle: 'Waiting for finalization...',
          color: '#13E1BC',
          bgColor: '#1B2A2A',
        };
      case 'finalized':
        return {
          icon: <Ionicons name="checkmark-circle" size={48} color="#18BB59" />,
          title: 'Transaction Successful',
          subtitle: 'Your transaction has been confirmed',
          color: '#18BB59',
          bgColor: '#1B2A1B',
        };
      case 'failed':
        return {
          icon: <Ionicons name="close-circle" size={48} color="#FF4444" />,
          title: 'Transaction Failed',
          subtitle: result.error || 'Transaction was rejected',
          color: '#FF4444',
          bgColor: '#2A1B1B',
        };
      default:
        return {
          icon: <Ionicons name="help-circle" size={48} color="#CDCDE0" />,
          title: 'Unknown Status',
          subtitle: 'Please check transaction manually',
          color: '#CDCDE0',
          bgColor: '#1B1B1B',
        };
    }
  };

  const config = getStatusConfig(result.status);
  const truncatedHash = `${result.hash.slice(0, 8)}...${result.hash.slice(-8)}`;

  const handleExplorerLink = () => {
    // Placeholder for explorer link
    // In production, this would open the transaction in a block explorer
    console.log('Opening explorer for tx:', result.hash);
  };

  return (
    <View className="bg-[#0C0E12] p-6">
      <View 
        className="rounded-xl p-6 items-center"
        style={{ backgroundColor: config.bgColor }}
      >
        {/* Status Icon */}
        <View className="mb-4">
          {config.icon}
        </View>

        {/* Status Title */}
        <Text 
          className="text-xl font-bold mb-2 text-center"
          style={{ color: config.color }}
        >
          {config.title}
        </Text>

        {/* Status Subtitle */}
        <Text className="text-[#CDCDE0] text-center mb-6">
          {config.subtitle}
        </Text>

        {/* Transaction Hash */}
        <View className="bg-[#1B1B1B] rounded-lg p-4 w-full mb-4">
          <Text className="text-[#CDCDE0] text-sm mb-1">Transaction Hash</Text>
          <Text className="text-white font-mono text-sm">
            {truncatedHash}
          </Text>
        </View>

        {/* Block Information */}
        {result.blockNumber && (
          <View className="bg-[#1B1B1B] rounded-lg p-4 w-full mb-4">
            <Text className="text-[#CDCDE0] text-sm mb-1">Block Number</Text>
            <Text className="text-white font-mono text-sm">
              #{result.blockNumber.toLocaleString()}
            </Text>
          </View>
        )}

        {/* Block Hash */}
        {result.blockHash && (
          <View className="bg-[#1B1B1B] rounded-lg p-4 w-full mb-6">
            <Text className="text-[#CDCDE0] text-sm mb-1">Block Hash</Text>
            <Text className="text-white font-mono text-sm">
              {`${result.blockHash.slice(0, 8)}...${result.blockHash.slice(-8)}`}
            </Text>
          </View>
        )}

        {/* Action Buttons */}
        <View className="w-full gap-3">
          {showExplorerLink && (
            <TouchableOpacity
              className="border border-[#13E1BC] rounded-xl py-3 px-6"
              onPress={handleExplorerLink}
            >
              <Text className="text-[#13E1BC] text-center font-semibold">
                View in Explorer
              </Text>
            </TouchableOpacity>
          )}

          {onClose && (
            <TouchableOpacity
              className="bg-[#18BB59] rounded-xl py-3 px-6"
              onPress={onClose}
            >
              <Text className="text-white text-center font-semibold">
                {result.status === 'finalized' ? 'Done' : 'Close'}
              </Text>
            </TouchableOpacity>
          )}
        </View>
      </View>
    </View>
  );
};

export default TransactionStatus;
