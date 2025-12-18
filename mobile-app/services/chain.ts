/**
 * Chain query methods for Chameleon Network
 */

import { ApiPromise } from '@polkadot/api';
import { BN } from '@polkadot/util';
import { formatBalance as polkadotFormatBalance } from '@polkadot/util';
import { apiService } from './api';
import { NETWORK_CONFIG } from '../config/network';

export interface SystemHealth {
  peers: number;
  isSyncing: boolean;
  shouldHavePeers: boolean;
}

export interface ChainInfo {
  name: string;
  version: string;
  properties: {
    ss58Format: number;
    tokenDecimals: number[];
    tokenSymbol: string[];
  };
}

export interface BlockInfo {
  number: number;
  hash: string;
  parentHash: string;
  timestamp?: number;
}

export interface BalanceInfo {
  free: string;
  reserved: string;
  frozen: string;
  total: string;
}

class ChainService {
  private static instance: ChainService;
  
  private constructor() {}

  public static getInstance(): ChainService {
    if (!ChainService.instance) {
      ChainService.instance = new ChainService();
    }
    return ChainService.instance;
  }

  /**
   * Get system health status
   */
  public async getSystemHealth(): Promise<SystemHealth> {
    const api = apiService.getApi();
    if (!api) {
      throw new Error('API not connected');
    }

    const health = await api.rpc.system.health();
    
    return {
      peers: (health.peers as any).toNumber(),
      isSyncing: (health.isSyncing as any).isTrue,
      shouldHavePeers: (health.shouldHavePeers as any).isTrue,
    };
  }

  /**
   * Get chain information
   */
  public async getChainInfo(): Promise<ChainInfo> {
    const api = apiService.getApi();
    if (!api) {
      throw new Error('API not connected');
    }

    const [name, version, properties] = await Promise.all([
      api.rpc.system.chain(),
      api.rpc.system.version(),
      api.rpc.system.properties(),
    ]);

    return {
      name: name.toString(),
      version: version.toString(),
      properties: {
        ss58Format: properties.ss58Format.unwrapOr(new BN(42)).toNumber(),
        tokenDecimals: properties.tokenDecimals.unwrapOr([new BN(18)]).map(d => d.toNumber()),
        tokenSymbol: properties.tokenSymbol.unwrapOr(['CHML']).map(s => s.toString()),
      },
    };
  }

  /**
   * Get latest block information
   */
  public async getLatestBlock(): Promise<BlockInfo> {
    const api = apiService.getApi();
    if (!api) {
      throw new Error('API not connected');
    }

    const header = await api.rpc.chain.getHeader();
    const hash = await api.rpc.chain.getBlockHash();
    
    // Try to get timestamp from block
    let timestamp: number | undefined;
    try {
      const block = await api.rpc.chain.getBlock(hash);
      const timestampExtrinsic = block.block.extrinsics.find(
        ext => ext.method.section === 'timestamp' && ext.method.method === 'set'
      );
      if (timestampExtrinsic) {
        timestamp = timestampExtrinsic.method.args[0].toNumber();
      }
    } catch (error) {
      console.warn('Could not get block timestamp:', error);
    }

    return {
      number: header.number.toNumber(),
      hash: hash.toString(),
      parentHash: header.parentHash.toString(),
      timestamp,
    };
  }

  /**
   * Get balance for an address
   */
  public async getBalance(address: string): Promise<BalanceInfo> {
    const api = apiService.getApi();
    if (!api) {
      throw new Error('API not connected');
    }

    const account = await api.query.system.account(address);
    const balance = (account as any).data;
    
    const free = balance.free.toString();
    const reserved = balance.reserved.toString();
    const frozen = balance.frozen.toString();
    const total = new BN(free).add(new BN(reserved)).toString();

    return {
      free,
      reserved,
      frozen,
      total,
    };
  }

  /**
   * Format balance with proper decimals and symbol
   */
  public formatBalance(raw: string | BN, decimals: number = NETWORK_CONFIG.tokenDecimals): string {
    const balance = new BN(raw.toString());
    
    // Convert from planck units to token units
    const divisor = new BN(10).pow(new BN(decimals));
    const tokenAmount = balance.div(divisor);
    const remainder = balance.mod(divisor);
    
    // Format with up to 6 decimal places for display
    let formatted = tokenAmount.toString();
    
    if (!remainder.isZero()) {
      const remainderStr = remainder.toString().padStart(decimals, '0');
      // Remove trailing zeros and add decimal part
      const decimalPart = remainderStr.replace(/0+$/, '');
      if (decimalPart.length > 0) {
        // Limit to 6 decimal places for display
        const displayDecimals = decimalPart.substring(0, 6);
        formatted += '.' + displayDecimals;
      }
    }
    
    // Add thousand separators
    const parts = formatted.split('.');
    parts[0] = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, ',');
    
    return parts.join('.') + ' ' + NETWORK_CONFIG.tokenSymbol;
  }

  /**
   * Subscribe to balance changes for an address
   */
  public subscribeToBalance(
    address: string, 
    callback: (balance: BalanceInfo) => void
  ): () => void {
    const api = apiService.getApi();
    if (!api) {
      throw new Error('API not connected');
    }

    let unsubscribe: (() => void) | null = null;

    api.query.system.account(address, (account: any) => {
      const balance = account.data;
      const free = balance.free.toString();
      const reserved = balance.reserved.toString();
      const frozen = balance.frozen.toString();
      const total = new BN(free).add(new BN(reserved)).toString();

      callback({
        free,
        reserved,
        frozen,
        total,
      });
    }).then((unsub) => {
      unsubscribe = unsub;
    }).catch(console.error);

    // Return unsubscribe function
    return () => {
      if (unsubscribe) {
        unsubscribe();
      }
    };
  }

  /**
   * Subscribe to new block headers
   */
  public subscribeToBlocks(callback: (block: BlockInfo) => void): () => void {
    const api = apiService.getApi();
    if (!api) {
      throw new Error('API not connected');
    }

    let unsubscribe: (() => void) | null = null;

    api.rpc.chain.subscribeNewHeads((header) => {
      callback({
        number: header.number.toNumber(),
        hash: header.hash.toString(),
        parentHash: header.parentHash.toString(),
      });
    }).then((unsub) => {
      unsubscribe = unsub;
    }).catch(console.error);

    return () => {
      if (unsubscribe) {
        unsubscribe();
      }
    };
  }
}

// Export singleton instance
export const chainService = ChainService.getInstance();
