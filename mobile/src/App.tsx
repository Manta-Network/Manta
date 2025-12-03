/**
 * Chameleon Wallet - Main App Component
 */

import React, { useEffect } from 'react';
import { StatusBar, LogBox } from 'react-native';
import { GestureHandlerRootView } from 'react-native-gesture-handler';
import { WalletProvider } from '@/context/WalletContext';
import RootNavigator from '@/navigation';
import { colors } from '@/theme';

// Ignore specific warnings for development
if (__DEV__) {
  LogBox.ignoreLogs([
    'Non-serializable values were found in the navigation state',
    'VirtualizedLists should never be nested',
  ]);
}

function App(): JSX.Element {
  useEffect(() => {
    // Set status bar style
    StatusBar.setBarStyle('light-content', true);
    StatusBar.setBackgroundColor(colors.background, true);
  }, []);

  return (
    <GestureHandlerRootView style={{ flex: 1 }}>
      <StatusBar
        barStyle="light-content"
        backgroundColor={colors.background}
        translucent={false}
      />
      
      <WalletProvider>
        <RootNavigator />
      </WalletProvider>
    </GestureHandlerRootView>
  );
}

export default App;
