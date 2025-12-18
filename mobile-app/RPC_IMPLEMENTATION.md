# Chameleon Network RPC Connection Implementation

## Overview

This document describes the RPC connection layer and API services implemented for the Chameleon Network mobile wallet.

## Files Created

### Core Services

#### `/config/network.ts`
- Network configuration constants
- Connection settings
- Chain constants (block time, existential deposit)

#### `/services/api.ts`
- Singleton API service for managing Polkadot.js connections
- WebSocket connection with HTTP fallback
- Auto-reconnection logic
- Connection state management
- Event listeners for connection status

#### `/services/chain.ts`
- Chain query methods (balance, system health, chain info)
- Balance formatting utilities
- Subscription management for real-time updates
- Block information retrieval

### React Hooks

#### `/hooks/useApi.ts`
- React hook for API connection lifecycle
- Returns connection state, API instance, and control methods
- Auto-connects on mount
- Handles reconnection attempts

#### `/hooks/useBalance.ts`
- React hook for balance subscription
- Real-time balance updates via WebSocket
- Formatted balance strings with proper decimals
- Error handling and loading states

### UI Components

#### `/components/NetworkBadge.tsx`
- DEVNET indicator with connection status
- Shows current block number when connected
- Color-coded status (green=connected, red=disconnected, yellow=connecting)
- Multiple size variants (small, medium, large)

#### `/components/ApiConnectionDemo.tsx`
- Comprehensive demo component for testing API functionality
- Chain information display
- Balance testing with custom addresses
- System health monitoring
- Network configuration display

### Utilities

#### `/utils/address.ts`
- Address validation and formatting
- SS58 address encoding for Chameleon Network
- Address truncation for UI display
- Test addresses for development

#### `/utils/format.ts`
- Token amount formatting (planck units to human readable)
- USD value formatting
- Percentage and duration formatting
- Input parsing utilities

## Network Configuration

- **Name**: Chameleon Devnet
- **WebSocket**: `ws://64.23.233.36:9944`
- **HTTP**: `http://64.23.233.36:9933`
- **Token**: CHML (18 decimals)
- **SS58 Prefix**: 42 (Substrate default)
- **Block Time**: 6 seconds

## Usage Examples

### Basic API Connection

```typescript
import { useApi } from '@/hooks/useApi';

function MyComponent() {
  const { api, isConnected, isConnecting, error, retry } = useApi();
  
  if (isConnecting) return <Text>Connecting...</Text>;
  if (error) return <Text>Error: {error}</Text>;
  if (!isConnected) return <Button onPress={retry}>Retry</Button>;
  
  return <Text>Connected to Chameleon Network!</Text>;
}
```

### Balance Subscription

```typescript
import { useBalance } from '@/hooks/useBalance';

function BalanceDisplay({ address }: { address: string }) {
  const { formattedFreeBalance, isLoading, error } = useBalance(address);
  
  if (isLoading) return <Spinner />;
  if (error) return <Text>Error: {error}</Text>;
  
  return <Text>{formattedFreeBalance}</Text>;
}
```

### Network Status Badge

```typescript
import { NetworkBadge } from '@/components/NetworkBadge';

// Simple badge
<NetworkBadge size="small" showConnectionStatus={false} />

// Full badge with connection info
<NetworkBadge size="medium" showConnectionStatus={true} />
```

## Testing

The implementation includes a comprehensive demo component accessible via the Connect tab:

1. **Connection Status**: Shows current connection state and allows retry
2. **Chain Information**: Displays chain name, version, token info, system health
3. **Balance Testing**: Test balance queries with any Substrate address
4. **Network Configuration**: Shows all network settings

### Test Addresses

- **Alice**: `5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY`
- **Bob**: `5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty`
- **Charlie**: `5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y`

## Dependencies Added

```json
{
  "@polkadot/api": "^12.0.0",
  "@polkadot/util": "^12.0.0",
  "@polkadot/util-crypto": "^12.0.0"
}
```

## Error Handling

- **Connection Failures**: Automatic retry with exponential backoff
- **Network Errors**: Graceful degradation with user feedback
- **Invalid Addresses**: Validation before API calls
- **API Errors**: Proper error messages and recovery options

## Performance Considerations

- **Singleton Pattern**: Single API instance shared across app
- **Subscription Management**: Proper cleanup to prevent memory leaks
- **Optimistic Updates**: UI updates before network confirmation
- **Connection Pooling**: Reuse existing connections when possible

## Next Steps (Phase 2)

1. **Wallet Integration**: Add keyring management and account creation
2. **Transaction Signing**: Implement send/receive functionality
3. **Privacy Features**: Add shield/unshield capabilities
4. **Enhanced UI**: Improve user experience and add animations
5. **Error Recovery**: More robust error handling and user guidance

## Security Notes

- All sensitive operations (private keys, seeds) will be handled in Phase 2
- Current implementation only handles read-only operations
- Network connections use standard Polkadot.js security practices
- Address validation prevents malformed input

## Troubleshooting

### Common Issues

1. **Connection Timeout**: Check network connectivity and endpoint availability
2. **Invalid Address**: Ensure address is valid Substrate format
3. **Balance Not Loading**: Verify address exists on network
4. **WebSocket Errors**: Will automatically fallback to HTTP

### Debug Information

The demo component provides detailed debug information including:
- Connection status and error messages
- Chain properties and system health
- Block numbers and timestamps
- Network configuration verification
