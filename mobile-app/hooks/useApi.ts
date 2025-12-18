/**
 * React hook for managing API connection lifecycle
 */

import { useState, useEffect, useCallback } from 'react';
import { ApiPromise } from '@polkadot/api';
import { apiService, type ApiConnectionState } from '../services/api';

export interface UseApiReturn {
  api: ApiPromise | null;
  isConnected: boolean;
  isConnecting: boolean;
  error: string | null;
  connectionState: ApiConnectionState;
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
  retry: () => Promise<void>;
}

/**
 * Hook for managing API connection to Chameleon Network
 */
export function useApi(): UseApiReturn {
  const [connectionState, setConnectionState] = useState<ApiConnectionState>(
    apiService.getConnectionState()
  );
  const [isRetrying, setIsRetrying] = useState(false);

  // Connect function
  const connect = useCallback(async () => {
    try {
      await apiService.connect();
    } catch (error) {
      console.error('Failed to connect to API:', error);
    }
  }, []);

  // Disconnect function
  const disconnect = useCallback(async () => {
    try {
      await apiService.disconnect();
    } catch (error) {
      console.error('Failed to disconnect from API:', error);
    }
  }, []);

  // Retry connection
  const retry = useCallback(async () => {
    if (isRetrying) return;
    
    setIsRetrying(true);
    try {
      await apiService.disconnect();
      await new Promise(resolve => setTimeout(resolve, 1000)); // Wait 1 second
      await apiService.connect();
    } catch (error) {
      console.error('Failed to retry connection:', error);
    } finally {
      setIsRetrying(false);
    }
  }, [isRetrying]);

  // Subscribe to connection state changes
  useEffect(() => {
    const unsubscribe = apiService.onConnectionStateChange((state) => {
      setConnectionState(state);
    });

    // Auto-connect on mount
    if (connectionState.status === 'disconnected') {
      connect();
    }

    return unsubscribe;
  }, [connect, connectionState.status]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      // Don't disconnect on unmount as other components might be using the API
      // The API service is a singleton and manages its own lifecycle
    };
  }, []);

  return {
    api: apiService.getApi(),
    isConnected: connectionState.status === 'connected',
    isConnecting: connectionState.status === 'connecting' || isRetrying,
    error: connectionState.error || null,
    connectionState,
    connect,
    disconnect,
    retry,
  };
}
