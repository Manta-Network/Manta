/**
 * Demo component to showcase API connection and balance fetching
 */

import React, { useState } from 'react';
import { View, Text, ScrollView, Alert } from 'react-native';
import { Button } from '@gluestack-ui/button';
import { Input, InputField } from '@gluestack-ui/input';
import { VStack } from '@gluestack-ui/vstack';
import { HStack } from '@gluestack-ui/hstack';
import { Card } from '@gluestack-ui/card';
import { Spinner } from '@gluestack-ui/spinner';
import { useApi } from '../hooks/useApi';
import { useBalance } from '../hooks/useBalance';
import { NetworkBadge, NetworkBadgeFull } from './NetworkBadge';
import { chainService } from '../services/chain';
import { NETWORK_CONFIG } from '../config/network';

export function ApiConnectionDemo() {
  const { api, isConnected, isConnecting, error, retry } = useApi();
  const [testAddress, setTestAddress] = useState('5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY'); // Alice
  const { balance, formattedFreeBalance, isLoading: balanceLoading, error: balanceError } = useBalance(testAddress);
  const [chainInfo, setChainInfo] = useState<any>(null);
  const [systemHealth, setSystemHealth] = useState<any>(null);
  const [latestBlock, setLatestBlock] = useState<any>(null);
  const [loading, setLoading] = useState(false);

  const fetchChainInfo = async () => {
    if (!isConnected) {
      Alert.alert('Error', 'Not connected to network');
      return;
    }

    setLoading(true);
    try {
      const [info, health, block] = await Promise.all([
        chainService.getChainInfo(),
        chainService.getSystemHealth(),
        chainService.getLatestBlock(),
      ]);
      
      setChainInfo(info);
      setSystemHealth(health);
      setLatestBlock(block);
    } catch (err) {
      console.error('Error fetching chain info:', err);
      Alert.alert('Error', 'Failed to fetch chain information');
    } finally {
      setLoading(false);
    }
  };

  const getStatusColor = () => {
    if (isConnecting) return 'text-yellow-500';
    if (isConnected) return 'text-green-500';
    return 'text-red-500';
  };

  const getStatusText = () => {
    if (isConnecting) return 'Connecting...';
    if (isConnected) return 'Connected';
    return 'Disconnected';
  };

  return (
    <ScrollView className="flex-1 bg-gray-900 p-4">
      <VStack space="md">
        {/* Network Badge */}
        <NetworkBadgeFull />

        {/* Connection Status */}
        <Card className="bg-gray-800 p-4">
          <VStack space="sm">
            <Text className="text-white text-lg font-bold">Connection Status</Text>
            
            <HStack className="items-center justify-between">
              <Text className={`text-base font-medium ${getStatusColor()}`}>
                {getStatusText()}
              </Text>
              
              {!isConnected && (
                <Button size="sm" onPress={retry} className="bg-teal-600">
                  <Text className="text-white">Retry</Text>
                </Button>
              )}
            </HStack>

            {error && (
              <Text className="text-red-400 text-sm">
                Error: {error}
              </Text>
            )}

            <Text className="text-gray-400 text-xs">
              Endpoint: {NETWORK_CONFIG.wsEndpoint}
            </Text>
          </VStack>
        </Card>

        {/* Chain Information */}
        <Card className="bg-gray-800 p-4">
          <VStack space="sm">
            <HStack className="items-center justify-between">
              <Text className="text-white text-lg font-bold">Chain Information</Text>
              
              <Button 
                size="sm" 
                onPress={fetchChainInfo} 
                disabled={!isConnected || loading}
                className="bg-teal-600"
              >
                {loading ? (
                  <Spinner size="small" color="white" />
                ) : (
                  <Text className="text-white">Fetch</Text>
                )}
              </Button>
            </HStack>

            {chainInfo && (
              <VStack space="xs">
                <Text className="text-gray-300">
                  <Text className="font-medium">Name:</Text> {chainInfo.name}
                </Text>
                <Text className="text-gray-300">
                  <Text className="font-medium">Version:</Text> {chainInfo.version}
                </Text>
                <Text className="text-gray-300">
                  <Text className="font-medium">Token:</Text> {chainInfo.properties.tokenSymbol[0]} 
                  ({chainInfo.properties.tokenDecimals[0]} decimals)
                </Text>
              </VStack>
            )}

            {systemHealth && (
              <VStack space="xs">
                <Text className="text-gray-300">
                  <Text className="font-medium">Peers:</Text> {systemHealth.peers}
                </Text>
                <Text className="text-gray-300">
                  <Text className="font-medium">Syncing:</Text> {systemHealth.isSyncing ? 'Yes' : 'No'}
                </Text>
              </VStack>
            )}

            {latestBlock && (
              <VStack space="xs">
                <Text className="text-gray-300">
                  <Text className="font-medium">Latest Block:</Text> #{latestBlock.number}
                </Text>
                <Text className="text-gray-300 text-xs">
                  <Text className="font-medium">Hash:</Text> {latestBlock.hash.substring(0, 20)}...
                </Text>
              </VStack>
            )}
          </VStack>
        </Card>

        {/* Balance Testing */}
        <Card className="bg-gray-800 p-4">
          <VStack space="sm">
            <Text className="text-white text-lg font-bold">Balance Testing</Text>
            
            <VStack space="xs">
              <Text className="text-gray-300 text-sm">Test Address:</Text>
              <Input className="bg-gray-700">
                <InputField
                  value={testAddress}
                  onChangeText={setTestAddress}
                  placeholder="Enter Substrate address..."
                  className="text-white"
                />
              </Input>
            </VStack>

            {balanceLoading ? (
              <HStack className="items-center">
                <Spinner size="small" color="white" />
                <Text className="text-gray-300 ml-2">Loading balance...</Text>
              </HStack>
            ) : balance ? (
              <VStack space="xs">
                <Text className="text-green-400 text-lg font-bold">
                  {formattedFreeBalance}
                </Text>
                <Text className="text-gray-400 text-sm">
                  Free: {formattedFreeBalance}
                </Text>
                {balance.reserved !== '0' && (
                  <Text className="text-gray-400 text-sm">
                    Reserved: {chainService.formatBalance(balance.reserved)}
                  </Text>
                )}
              </VStack>
            ) : balanceError ? (
              <Text className="text-red-400 text-sm">
                Error: {balanceError}
              </Text>
            ) : (
              <Text className="text-gray-400 text-sm">
                Enter an address to check balance
              </Text>
            )}
          </VStack>
        </Card>

        {/* Network Configuration */}
        <Card className="bg-gray-800 p-4">
          <VStack space="sm">
            <Text className="text-white text-lg font-bold">Network Configuration</Text>
            
            <VStack space="xs">
              <Text className="text-gray-300 text-sm">
                <Text className="font-medium">Name:</Text> {NETWORK_CONFIG.name}
              </Text>
              <Text className="text-gray-300 text-sm">
                <Text className="font-medium">WebSocket:</Text> {NETWORK_CONFIG.wsEndpoint}
              </Text>
              <Text className="text-gray-300 text-sm">
                <Text className="font-medium">HTTP:</Text> {NETWORK_CONFIG.httpEndpoint}
              </Text>
              <Text className="text-gray-300 text-sm">
                <Text className="font-medium">Token:</Text> {NETWORK_CONFIG.tokenSymbol} ({NETWORK_CONFIG.tokenDecimals} decimals)
              </Text>
              <Text className="text-gray-300 text-sm">
                <Text className="font-medium">SS58 Prefix:</Text> {NETWORK_CONFIG.ss58Prefix}
              </Text>
              <Text className="text-gray-300 text-sm">
                <Text className="font-medium">Testnet:</Text> {NETWORK_CONFIG.isTestnet ? 'Yes' : 'No'}
              </Text>
            </VStack>
          </VStack>
        </Card>
      </VStack>
    </ScrollView>
  );
}
