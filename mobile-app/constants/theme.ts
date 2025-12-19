/**
 * New Design System Theme Constants for Chameleon Wallet
 * Based on the improved UI design with light theme and green accents
 */

export const THEME = {
  colors: {
    // Primary Colors
    primary: '#22B958',        // Main green for primary actions
    primaryLight: '#23BD5E33', // Light green with opacity
    green2: '#64A121',         // Alternative green
    secondary: '#1A73E8',      // Blue for links and secondary actions
    
    // Backgrounds (Light Theme)
    background: '#FFFFFF',     // Main background
    backgroundGradientStart: '#E8F5E9', // Light green gradient start
    backgroundGradientEnd: '#FFFFFF',   // White gradient end
    card: '#F8FFFB',          // Cards with slight green tint
    cardFrosted: 'rgba(255,255,255,0.5)', // Frosted glass effect
    cardBorder: 'rgba(255,255,255,0.75)', // Card borders
    lightGrey: '#F2F4F5',     // Secondary backgrounds
    
    // Text Colors
    text: '#111111',          // Primary text (dark)
    textSecondary: '#858383', // Secondary text
    textMuted: '#9C9C9C',     // Muted text
    
    // Status Colors
    success: '#09A552',
    successBg: '#09A5521A',   // Success background with opacity
    warning: '#F2B609',
    warningBg: '#F2B6091A',   // Warning background with opacity
    error: '#D32F2F',
    errorBg: '#DF26381A',     // Error background with opacity
    errorAlt: '#F6465D',      // Alternative error color
    
    // Additional UI Colors
    white: '#FFFFFF',
    black: '#000000',
    grey: '#9C9C9C',
    border: '#E0E0E0',
    shadow: 'rgba(0,0,0,0.1)',
  },
  
  borderRadius: {
    small: 8,
    medium: 16,
    large: 22,
    full: 32,
  },
  
  spacing: {
    xs: 4,
    sm: 8,
    md: 16,
    lg: 24,
    xl: 32,
  },
  
  fontSize: {
    xs: 12,
    sm: 14,
    base: 16,
    lg: 18,
    xl: 20,
    '2xl': 24,
    '3xl': 30,
    '4xl': 36,
  },
  
  fontWeight: {
    normal: '400' as const,
    medium: '500' as const,
    semibold: '600' as const,
    bold: '700' as const,
  },
  
  shadows: {
    small: {
      shadowColor: '#000',
      shadowOffset: { width: 0, height: 2 },
      shadowOpacity: 0.1,
      shadowRadius: 4,
      elevation: 2,
    },
    medium: {
      shadowColor: '#000',
      shadowOffset: { width: 0, height: 4 },
      shadowOpacity: 0.15,
      shadowRadius: 8,
      elevation: 4,
    },
    large: {
      shadowColor: '#000',
      shadowOffset: { width: 0, height: 8 },
      shadowOpacity: 0.2,
      shadowRadius: 16,
      elevation: 8,
    },
  },
};

// Gradient configurations
export const GRADIENTS = {
  background: {
    colors: [THEME.colors.backgroundGradientStart, THEME.colors.backgroundGradientEnd],
    start: { x: 0, y: 0 },
    end: { x: 0, y: 1 },
  },
  card: {
    colors: ['rgba(255,255,255,0.8)', 'rgba(255,255,255,0.4)'],
    start: { x: 0, y: 0 },
    end: { x: 0, y: 1 },
  },
};

// Action grid icons mapping
export const ACTION_ICONS = {
  send: require('../assets/images/papp/Send.png'),
  receive: require('../assets/images/papp/Receive.png'),
  shield: require('../assets/images/papp/Shield.png'),
  trade: require('../assets/images/papp/trade.png'),
  buy: require('../assets/images/papp/buyCHML.png'),
  mint: require('../assets/images/papp/mint.png'),
  power: require('../assets/images/papp/power.png'),
  provide: require('../assets/images/papp/provide.png'),
};

export default THEME;
