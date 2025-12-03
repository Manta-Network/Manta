/**
 * Chameleon Wallet - Theme Configuration
 */

// Color palette
export const colors = {
  // Primary colors
  primary: '#7B61FF',      // Purple (Chameleon brand)
  secondary: '#00D4AA',    // Teal
  
  // Background colors
  background: '#0F0F23',   // Dark navy
  surface: '#1A1A2E',     // Slightly lighter
  card: '#252545',        // Card background
  
  // Text colors
  text: '#FFFFFF',        // Primary text
  textSecondary: '#A0A0B8', // Secondary text
  textTertiary: '#6B6B8A', // Tertiary text
  
  // Status colors
  success: '#00D4AA',     // Success/positive
  error: '#FF4757',       // Error/negative
  warning: '#FFA502',     // Warning
  info: '#3742FA',        // Info
  
  // Utility colors
  white: '#FFFFFF',
  black: '#000000',
  transparent: 'transparent',
  
  // Border colors
  border: '#2A2A3E',
  borderLight: '#3A3A4E',
  
  // Overlay colors
  overlay: 'rgba(0, 0, 0, 0.5)',
  overlayLight: 'rgba(0, 0, 0, 0.3)',
  
  // Gradient colors
  gradientStart: '#7B61FF',
  gradientEnd: '#00D4AA',
};

// Typography
export const fonts = {
  // Font families (iOS/Android compatible)
  regular: 'System',
  medium: 'System',
  bold: 'System',
  light: 'System',
  
  // Font sizes
  xs: 12,
  sm: 14,
  base: 16,
  lg: 18,
  xl: 20,
  '2xl': 24,
  '3xl': 30,
  '4xl': 36,
  '5xl': 48,
};

// Spacing
export const spacing = {
  xs: 4,
  sm: 8,
  md: 16,
  lg: 24,
  xl: 32,
  '2xl': 48,
  '3xl': 64,
};

// Border radius
export const borderRadius = {
  sm: 4,
  md: 8,
  lg: 12,
  xl: 16,
  '2xl': 20,
  '3xl': 24,
  full: 9999,
};

// Shadows
export const shadows = {
  sm: {
    shadowColor: colors.black,
    shadowOffset: {
      width: 0,
      height: 1,
    },
    shadowOpacity: 0.1,
    shadowRadius: 2,
    elevation: 2,
  },
  md: {
    shadowColor: colors.black,
    shadowOffset: {
      width: 0,
      height: 2,
    },
    shadowOpacity: 0.15,
    shadowRadius: 4,
    elevation: 4,
  },
  lg: {
    shadowColor: colors.black,
    shadowOffset: {
      width: 0,
      height: 4,
    },
    shadowOpacity: 0.2,
    shadowRadius: 8,
    elevation: 8,
  },
};

// Animation durations
export const animations = {
  fast: 150,
  normal: 300,
  slow: 500,
};

// Screen dimensions helpers
export const layout = {
  window: {
    width: 0, // Will be set by useWindowDimensions
    height: 0,
  },
  isSmallDevice: false, // Will be calculated
};

// Theme object
export const theme = {
  colors,
  fonts,
  spacing,
  borderRadius,
  shadows,
  animations,
  layout,
};

export type Theme = typeof theme;

// Dark theme (default)
export const darkTheme = theme;

// Light theme (for future implementation)
export const lightTheme: Theme = {
  ...theme,
  colors: {
    ...colors,
    background: '#FFFFFF',
    surface: '#F8F9FA',
    card: '#FFFFFF',
    text: '#1A1A2E',
    textSecondary: '#6B6B8A',
    textTertiary: '#A0A0B8',
    border: '#E5E5E5',
    borderLight: '#F0F0F0',
  },
};
