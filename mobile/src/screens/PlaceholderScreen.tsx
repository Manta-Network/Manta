/**
 * Chameleon Wallet - Placeholder Screen
 * Used for screens that are not yet implemented
 */

import React from 'react';
import {
  View,
  Text,
  StyleSheet,
  SafeAreaView,
  StatusBar,
} from 'react-native';
import { Button } from '@/components/Button';
import { colors, fonts, spacing } from '@/theme';
import type { NativeStackScreenProps } from '@react-navigation/native-stack';
import type { RootStackParamList } from '@/types';

type Props = NativeStackScreenProps<RootStackParamList, any>;

export function PlaceholderScreen({ navigation, route }: Partial<Props> = {}) {
  const screenName = route?.name || 'Screen';

  const handleGoBack = () => {
    if (navigation?.canGoBack()) {
      navigation.goBack();
    }
  };

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="light-content" backgroundColor={colors.background} />
      
      <View style={styles.content}>
        <View style={styles.iconContainer}>
          <Text style={styles.icon}>🚧</Text>
        </View>
        
        <Text style={styles.title}>{screenName}</Text>
        <Text style={styles.subtitle}>
          This screen is coming soon!
        </Text>
        
        <Text style={styles.description}>
          We're working hard to bring you this feature. 
          Check back in future updates.
        </Text>
      </View>

      <View style={styles.actions}>
        {navigation?.canGoBack() && (
          <Button
            title="Go Back"
            onPress={handleGoBack}
            variant="primary"
            size="large"
            fullWidth
          />
        )}
      </View>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: colors.background,
  },
  
  content: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    paddingHorizontal: spacing.lg,
  },
  
  iconContainer: {
    width: 120,
    height: 120,
    borderRadius: 60,
    backgroundColor: colors.surface,
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: spacing.xl,
  },
  
  icon: {
    fontSize: 60,
  },
  
  title: {
    fontSize: fonts['2xl'],
    fontFamily: fonts.bold,
    color: colors.text,
    marginBottom: spacing.sm,
    textAlign: 'center',
  },
  
  subtitle: {
    fontSize: fonts.lg,
    fontFamily: fonts.medium,
    color: colors.primary,
    marginBottom: spacing.lg,
    textAlign: 'center',
  },
  
  description: {
    fontSize: fonts.base,
    fontFamily: fonts.regular,
    color: colors.textSecondary,
    textAlign: 'center',
    lineHeight: 24,
    maxWidth: 300,
  },
  
  actions: {
    paddingHorizontal: spacing.lg,
    paddingBottom: spacing.lg,
  },
});
