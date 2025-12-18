/**
 * Polkadot.js API connection wrapper for Chameleon Network
 */

import { ApiPromise, WsProvider, HttpProvider } from '@polkadot/api';
import { NETWORK_CONFIG, CONNECTION_CONFIG } from '../config/network';

export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

export interface ApiConnectionState {
  status: ConnectionStatus;
  error?: string;
  blockNumber?: number;
}

class ApiService {
  private static instance: ApiService;
  private api: ApiPromise | null = null;
  private provider: WsProvider | HttpProvider | null = null;
  private connectionState: ApiConnectionState = { status: 'disconnected' };
  private listeners: ((state: ApiConnectionState) => void)[] = [];
  private reconnectAttempts = 0;
  private reconnectTimer: NodeJS.Timeout | null = null;

  private constructor() {}

  public static getInstance(): ApiService {
    if (!ApiService.instance) {
      ApiService.instance = new ApiService();
    }
    return ApiService.instance;
  }

  /**
   * Connect to the Chameleon Network
   */
  public async connect(): Promise<void> {
    if (this.connectionState.status === 'connecting' || this.connectionState.status === 'connected') {
      return;
    }

    this.updateConnectionState({ status: 'connecting' });

    try {
      // Try WebSocket first, fallback to HTTP
      await this.connectWithProvider('ws');
    } catch (wsError) {
      console.warn('WebSocket connection failed, trying HTTP:', wsError);
      try {
        await this.connectWithProvider('http');
      } catch (httpError) {
        console.error('Both WebSocket and HTTP connections failed:', httpError);
        this.updateConnectionState({ 
          status: 'error', 
          error: 'Failed to connect to network' 
        });
        this.scheduleReconnect();
        throw httpError;
      }
    }
  }

  /**
   * Connect with specific provider type
   */
  private async connectWithProvider(type: 'ws' | 'http'): Promise<void> {
    const endpoint = type === 'ws' ? NETWORK_CONFIG.wsEndpoint : NETWORK_CONFIG.httpEndpoint;
    
    this.provider = type === 'ws' 
      ? new WsProvider(endpoint, CONNECTION_CONFIG.reconnectAttempts)
      : new HttpProvider(endpoint);

    // Set up provider event listeners for WebSocket
    if (type === 'ws' && this.provider instanceof WsProvider) {
      this.provider.on('connected', () => {
        console.log('WebSocket connected');
        this.reconnectAttempts = 0;
      });

      this.provider.on('disconnected', () => {
        console.log('WebSocket disconnected');
        this.updateConnectionState({ status: 'disconnected' });
        this.scheduleReconnect();
      });

      this.provider.on('error', (error) => {
        console.error('WebSocket error:', error);
        this.updateConnectionState({ status: 'error', error: error.message });
      });
    }

    // Create API instance
    this.api = await ApiPromise.create({ 
      provider: this.provider,
      throwOnConnect: true,
    });

    // Wait for API to be ready
    await this.api.isReady;

    // Subscribe to new block headers
    await this.api.rpc.chain.subscribeNewHeads((header) => {
      this.updateConnectionState({ 
        status: 'connected', 
        blockNumber: header.number.toNumber() 
      });
    });

    this.updateConnectionState({ status: 'connected' });
    console.log('Connected to Chameleon Network:', endpoint);
  }

  /**
   * Disconnect from the network
   */
  public async disconnect(): Promise<void> {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (this.api) {
      await this.api.disconnect();
      this.api = null;
    }

    if (this.provider) {
      if (this.provider instanceof WsProvider) {
        await this.provider.disconnect();
      }
      this.provider = null;
    }

    this.updateConnectionState({ status: 'disconnected' });
    console.log('Disconnected from Chameleon Network');
  }

  /**
   * Get the API instance
   */
  public getApi(): ApiPromise | null {
    return this.api;
  }

  /**
   * Check if connected
   */
  public isConnected(): boolean {
    return this.connectionState.status === 'connected' && this.api !== null;
  }

  /**
   * Get current connection state
   */
  public getConnectionState(): ApiConnectionState {
    return { ...this.connectionState };
  }

  /**
   * Subscribe to connection state changes
   */
  public onConnectionStateChange(listener: (state: ApiConnectionState) => void): () => void {
    this.listeners.push(listener);
    
    // Return unsubscribe function
    return () => {
      const index = this.listeners.indexOf(listener);
      if (index > -1) {
        this.listeners.splice(index, 1);
      }
    };
  }

  /**
   * Update connection state and notify listeners
   */
  private updateConnectionState(newState: Partial<ApiConnectionState>): void {
    this.connectionState = { ...this.connectionState, ...newState };
    this.listeners.forEach(listener => listener(this.connectionState));
  }

  /**
   * Schedule reconnection attempt
   */
  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= CONNECTION_CONFIG.reconnectAttempts) {
      console.log('Max reconnection attempts reached');
      return;
    }

    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
    }

    this.reconnectTimer = setTimeout(() => {
      this.reconnectAttempts++;
      console.log(`Reconnection attempt ${this.reconnectAttempts}/${CONNECTION_CONFIG.reconnectAttempts}`);
      this.connect().catch(console.error);
    }, CONNECTION_CONFIG.reconnectDelay);
  }
}

// Export singleton instance
export const apiService = ApiService.getInstance();
