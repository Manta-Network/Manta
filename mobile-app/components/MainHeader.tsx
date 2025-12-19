/**
 * MainHeader component with user avatar, name, and notification bell
 */

import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet, Image } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { THEME } from '../constants/theme';
import { NetworkBadge } from './NetworkBadge';

interface MainHeaderProps {
  userName?: string;
  onNotificationPress?: () => void;
  hasNotifications?: boolean;
}

const DEFAULT_USER = {
  name: 'Eleanor Pena',
  avatar: require('../assets/images/avatar.png'),
};

export const MainHeader: React.FC<MainHeaderProps> = ({
  userName = DEFAULT_USER.name,
  onNotificationPress,
  hasNotifications = true,
}) => {
  return (
    <View style={styles.container}>
      <View style={styles.headerRow}>
        {/* User Avatar */}
        <Image source={DEFAULT_USER.avatar} style={styles.avatar} />
        
        {/* User Name */}
        <Text style={styles.userName}>{userName}</Text>
        
        {/* Right Side - Notification and Network Badge */}
        <View style={styles.rightSection}>
          {/* Notification Bell */}
          <TouchableOpacity
            style={styles.bellContainer}
            onPress={onNotificationPress}
            activeOpacity={0.7}
          >
            <View style={styles.bellIconContainer}>
              <Ionicons
                name="notifications-outline"
                size={20}
                color={THEME.colors.text}
              />
              {hasNotifications && <View style={styles.notificationBadge} />}
            </View>
          </TouchableOpacity>
          
          {/* Network Badge */}
          <NetworkBadge />
        </View>
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    paddingHorizontal: THEME.spacing.md,
    paddingVertical: THEME.spacing.sm,
  },
  headerRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  avatar: {
    width: 32,
    height: 32,
    borderRadius: 16,
    marginRight: THEME.spacing.sm,
  },
  userName: {
    flex: 1,
    fontSize: THEME.fontSize.base,
    fontWeight: THEME.fontWeight.semibold,
    color: THEME.colors.text,
  },
  rightSection: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: THEME.spacing.sm,
  },
  bellContainer: {
    padding: THEME.spacing.xs,
  },
  bellIconContainer: {
    backgroundColor: 'rgba(255, 255, 255, 0.5)',
    borderRadius: 20,
    padding: THEME.spacing.sm,
    position: 'relative',
  },
  notificationBadge: {
    position: 'absolute',
    top: 4,
    right: 4,
    width: 12,
    height: 12,
    borderRadius: 6,
    backgroundColor: THEME.colors.error,
    borderWidth: 2,
    borderColor: THEME.colors.white,
  },
});

export default MainHeader;
