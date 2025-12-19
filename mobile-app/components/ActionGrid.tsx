/**
 * ActionGrid component with 4x2 grid of action buttons
 */

import React, { useState } from 'react';
import { View, Text, TouchableOpacity, StyleSheet, Image, Alert } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useRouter } from 'expo-router';
import { THEME, ACTION_ICONS } from '../constants/theme';

interface ActionItem {
  id: string;
  label: string;
  icon: any;
  route?: string;
  onPress?: () => void;
  disabled?: boolean;
}

interface ActionGridProps {
  onActionPress?: (actionId: string) => void;
}

const PRIMARY_ACTIONS: ActionItem[] = [
  {
    id: 'send',
    label: 'Send',
    icon: ACTION_ICONS.send,
    route: '/send',
  },
  {
    id: 'receive',
    label: 'Receive',
    icon: ACTION_ICONS.receive,
    route: '/receive',
  },
  {
    id: 'shield',
    label: 'Shield',
    icon: ACTION_ICONS.shield,
    disabled: true, // Coming soon
  },
  {
    id: 'trade',
    label: 'Trade',
    icon: ACTION_ICONS.trade,
    disabled: true, // Coming soon
  },
  {
    id: 'buy',
    label: 'Buy CHML',
    icon: ACTION_ICONS.buy,
    disabled: true, // Coming soon
  },
  {
    id: 'mint',
    label: 'Mint',
    icon: ACTION_ICONS.mint,
    disabled: true, // Coming soon
  },
  {
    id: 'power',
    label: 'Power',
    icon: ACTION_ICONS.power,
    disabled: true, // Coming soon
  },
];

const MORE_ACTIONS: ActionItem[] = [
  {
    id: 'provide',
    label: 'Provide',
    icon: ACTION_ICONS.provide,
    disabled: true,
  },
  {
    id: 'stake',
    label: 'Stake',
    icon: null, // Use Ionicon
    disabled: true,
  },
  {
    id: 'bridge',
    label: 'Bridge',
    icon: null, // Use Ionicon
    disabled: true,
  },
];

export const ActionGrid: React.FC<ActionGridProps> = ({ onActionPress }) => {
  const router = useRouter();
  const [showMore, setShowMore] = useState(false);

  const handleActionPress = (action: ActionItem) => {
    if (action.disabled) {
      Alert.alert('Coming Soon', `${action.label} feature will be available in a future update.`);
      return;
    }

    if (onActionPress) {
      onActionPress(action.id);
    }

    if (action.route) {
      router.push(action.route as any);
    } else if (action.onPress) {
      action.onPress();
    }
  };

  const handleMorePress = () => {
    setShowMore(!showMore);
  };

  const renderActionButton = (action: ActionItem, index: number) => {
    const isDisabled = action.disabled;
    
    return (
      <TouchableOpacity
        key={action.id}
        style={[
          styles.actionButton,
          isDisabled && styles.actionButtonDisabled,
        ]}
        onPress={() => handleActionPress(action)}
        activeOpacity={0.7}
        disabled={isDisabled}
      >
        <View style={[
          styles.iconContainer,
          isDisabled && styles.iconContainerDisabled,
        ]}>
          {action.icon ? (
            <Image
              source={action.icon}
              style={[
                styles.actionIcon,
                isDisabled && styles.actionIconDisabled,
              ]}
              resizeMode="contain"
            />
          ) : (
            <Ionicons
              name={getIoniconName(action.id)}
              size={24}
              color={isDisabled ? THEME.colors.textMuted : THEME.colors.primary}
            />
          )}
        </View>
        <Text style={[
          styles.actionLabel,
          isDisabled && styles.actionLabelDisabled,
        ]}>
          {action.label}
        </Text>
      </TouchableOpacity>
    );
  };

  const getIoniconName = (actionId: string): any => {
    switch (actionId) {
      case 'stake':
        return 'trending-up';
      case 'bridge':
        return 'swap-horizontal';
      default:
        return 'ellipsis-horizontal';
    }
  };

  const displayActions = PRIMARY_ACTIONS.slice(0, 7); // Show first 7 actions
  const moreAction = {
    id: 'more',
    label: showMore ? 'Less' : 'More',
    icon: null,
    onPress: handleMorePress,
  };

  return (
    <View style={styles.container}>
      {/* Primary Grid */}
      <View style={styles.grid}>
        {displayActions.map((action, index) => renderActionButton(action, index))}
        
        {/* More Button */}
        <TouchableOpacity
          style={styles.actionButton}
          onPress={handleMorePress}
          activeOpacity={0.7}
        >
          <View style={styles.iconContainer}>
            <Ionicons
              name={showMore ? 'chevron-up' : 'ellipsis-horizontal'}
              size={24}
              color={THEME.colors.primary}
            />
          </View>
          <Text style={styles.actionLabel}>{showMore ? 'Less' : 'More'}</Text>
        </TouchableOpacity>
      </View>

      {/* Expanded Actions */}
      {showMore && (
        <View style={styles.expandedGrid}>
          {MORE_ACTIONS.map((action, index) => renderActionButton(action, index))}
        </View>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    marginHorizontal: THEME.spacing.md,
    marginVertical: THEME.spacing.sm,
  },
  grid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'space-between',
    backgroundColor: THEME.colors.white,
    borderRadius: THEME.borderRadius.large,
    padding: THEME.spacing.lg,
    ...THEME.shadows.small,
  },
  expandedGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'flex-start',
    backgroundColor: THEME.colors.white,
    borderRadius: THEME.borderRadius.large,
    padding: THEME.spacing.lg,
    marginTop: THEME.spacing.sm,
    ...THEME.shadows.small,
  },
  actionButton: {
    width: '22%', // 4 columns with some spacing
    alignItems: 'center',
    marginBottom: THEME.spacing.lg,
  },
  actionButtonDisabled: {
    opacity: 0.5,
  },
  iconContainer: {
    width: 48,
    height: 48,
    borderRadius: 24,
    backgroundColor: THEME.colors.primaryLight,
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: THEME.spacing.xs,
  },
  iconContainerDisabled: {
    backgroundColor: THEME.colors.lightGrey,
  },
  actionIcon: {
    width: 28,
    height: 28,
  },
  actionIconDisabled: {
    opacity: 0.5,
  },
  actionLabel: {
    fontSize: THEME.fontSize.xs,
    fontWeight: THEME.fontWeight.medium,
    color: THEME.colors.text,
    textAlign: 'center',
  },
  actionLabelDisabled: {
    color: THEME.colors.textMuted,
  },
});

export default ActionGrid;
