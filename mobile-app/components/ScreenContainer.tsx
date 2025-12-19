/**
 * ScreenContainer with gradient background for the new design system
 */

import React from 'react';
import { View, ScrollView, StyleSheet, ImageBackground } from 'react-native';
import { LinearGradient } from 'expo-linear-gradient';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { THEME, GRADIENTS } from '../constants/theme';

interface ScreenContainerProps {
  children: React.ReactNode;
  style?: any;
  contentContainerStyle?: any;
  scrollable?: boolean;
  showGradient?: boolean;
}

export const ScreenContainer: React.FC<ScreenContainerProps> = ({
  children,
  style,
  contentContainerStyle,
  scrollable = false,
  showGradient = true,
}) => {
  const insets = useSafeAreaInsets();

  const containerStyle = [
    styles.container,
    {
      paddingTop: insets.top,
      paddingBottom: insets.bottom,
      paddingLeft: insets.left,
      paddingRight: insets.right,
    },
    style,
  ];

  const content = scrollable ? (
    <ScrollView
      style={styles.scrollView}
      contentContainerStyle={[styles.scrollContent, contentContainerStyle]}
      showsVerticalScrollIndicator={false}
    >
      {children}
    </ScrollView>
  ) : (
    <View style={[styles.content, contentContainerStyle]}>
      {children}
    </View>
  );

  if (showGradient) {
    return (
      <LinearGradient
        colors={GRADIENTS.background.colors}
        start={GRADIENTS.background.start}
        end={GRADIENTS.background.end}
        style={containerStyle}
      >
        {content}
      </LinearGradient>
    );
  }

  return (
    <View style={[containerStyle, { backgroundColor: THEME.colors.background }]}>
      {content}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  scrollView: {
    flex: 1,
  },
  scrollContent: {
    flexGrow: 1,
  },
  content: {
    flex: 1,
  },
});

export default ScreenContainer;
