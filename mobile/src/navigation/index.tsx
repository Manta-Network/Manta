/**
 * Chameleon Wallet - Navigation Setup
 */

import React from 'react';
import { View, Text } from 'react-native';
import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import { useWalletContext } from '@/context/WalletContext';
import { colors, fonts } from '@/theme';

// Import screens
import { WelcomeScreen } from '@/screens/WelcomeScreen';
import { CreateWalletScreen } from '@/screens/CreateWalletScreen';
import { ImportWalletScreen } from '@/screens/ImportWalletScreen';
import { HomeScreen } from '@/screens/HomeScreen';
import { SendScreen } from '@/screens/SendScreen';
import { ReceiveScreen } from '@/screens/ReceiveScreen';

// Placeholder screens (to be implemented)
import { PlaceholderScreen } from '@/screens/PlaceholderScreen';

import type { RootStackParamList, TabParamList } from '@/types';

const Stack = createNativeStackNavigator<RootStackParamList>();
const Tab = createBottomTabNavigator<TabParamList>();

/**
 * Main tab navigator for authenticated users
 */
function MainTabNavigator() {
  return (
    <Tab.Navigator
      screenOptions={{
        headerShown: false,
        tabBarStyle: {
          backgroundColor: colors.surface,
          borderTopColor: colors.border,
          borderTopWidth: 1,
          paddingTop: 8,
          paddingBottom: 8,
          height: 80,
        },
        tabBarActiveTintColor: colors.primary,
        tabBarInactiveTintColor: colors.textSecondary,
        tabBarLabelStyle: {
          fontSize: 12,
          fontFamily: fonts.medium,
          marginTop: 4,
        },
      }}
    >
      <Tab.Screen
        name="Home"
        component={HomeScreen}
        options={{
          tabBarIcon: ({ color, size }) => (
            <TabIcon icon="🏠" color={color} size={size} />
          ),
        }}
      />
      <Tab.Screen
        name="Staking"
        component={PlaceholderScreen}
        options={{
          tabBarIcon: ({ color, size }) => (
            <TabIcon icon="🏦" color={color} size={size} />
          ),
        }}
      />
      <Tab.Screen
        name="DEX"
        component={PlaceholderScreen}
        options={{
          tabBarIcon: ({ color, size }) => (
            <TabIcon icon="🔄" color={color} size={size} />
          ),
        }}
      />
      <Tab.Screen
        name="Settings"
        component={PlaceholderScreen}
        options={{
          tabBarIcon: ({ color, size }) => (
            <TabIcon icon="⚙️" color={color} size={size} />
          ),
        }}
      />
    </Tab.Navigator>
  );
}

/**
 * Tab icon component
 */
function TabIcon({ icon, color, size }: { icon: string; color: string; size: number }) {
  return (
    <Text style={{ fontSize: size, color }}>
      {icon}
    </Text>
  );
}

/**
 * Root navigation stack
 */
function RootNavigator() {
  const { state } = useWalletContext();
  const { isInitialized, hasWallet, isLocked } = state;

  // Show loading screen while initializing
  if (!isInitialized) {
    return (
      <NavigationContainer>
        <Stack.Navigator screenOptions={{ headerShown: false }}>
          <Stack.Screen name="Loading" component={LoadingScreen} />
        </Stack.Navigator>
      </NavigationContainer>
    );
  }

  return (
    <NavigationContainer>
      <Stack.Navigator
        screenOptions={{
          headerShown: false,
          gestureEnabled: true,
          animation: 'slide_from_right',
        }}
      >
        {!hasWallet ? (
          // Onboarding flow
          <>
            <Stack.Screen name="Welcome" component={WelcomeScreen} />
            <Stack.Screen name="CreateWallet" component={CreateWalletScreen} />
            <Stack.Screen name="ImportWallet" component={ImportWalletScreen} />
            <Stack.Screen name="BackupSeed" component={PlaceholderScreen} />
            <Stack.Screen name="ConfirmSeed" component={PlaceholderScreen} />
            <Stack.Screen name="SetupPin" component={PlaceholderScreen} />
            <Stack.Screen name="SetupBiometric" component={PlaceholderScreen} />
          </>
        ) : isLocked ? (
          // Locked state
          <Stack.Screen name="Unlock" component={PlaceholderScreen} />
        ) : (
          // Main app flow
          <>
            <Stack.Screen name="Main" component={MainTabNavigator} />
            <Stack.Screen name="Send" component={PlaceholderScreen} />
            <Stack.Screen name="Receive" component={PlaceholderScreen} />
            <Stack.Screen name="TransactionDetail" component={PlaceholderScreen} />
            <Stack.Screen name="ValidatorDetail" component={PlaceholderScreen} />
            <Stack.Screen name="Stake" component={PlaceholderScreen} />
            <Stack.Screen name="Swap" component={PlaceholderScreen} />
            <Stack.Screen name="LiquidityAdd" component={PlaceholderScreen} />
            <Stack.Screen name="LiquidityRemove" component={PlaceholderScreen} />
            <Stack.Screen name="Security" component={PlaceholderScreen} />
            <Stack.Screen name="Network" component={PlaceholderScreen} />
            <Stack.Screen name="About" component={PlaceholderScreen} />
          </>
        )}
      </Stack.Navigator>
    </NavigationContainer>
  );
}

/**
 * Loading screen component
 */
function LoadingScreen() {
  return (
    <View style={{
      flex: 1,
      backgroundColor: colors.background,
      alignItems: 'center',
      justifyContent: 'center',
    }}>
      <Text style={{
        fontSize: 60,
        marginBottom: 24,
      }}>
        🦎
      </Text>
      <Text style={{
        fontSize: 24,
        fontFamily: fonts.bold,
        color: colors.text,
        marginBottom: 8,
      }}>
        Chameleon Wallet
      </Text>
      <Text style={{
        fontSize: 16,
        fontFamily: fonts.regular,
        color: colors.textSecondary,
      }}>
        Initializing...
      </Text>
    </View>
  );
}

export default RootNavigator;
