/**
 * QR Code component for displaying wallet addresses
 */

import React from 'react';
import { View } from 'react-native';
import QRCodeSVG from 'react-native-qrcode-svg';

interface QRCodeProps {
  value: string;
  size?: number;
  backgroundColor?: string;
  color?: string;
  logo?: any;
  logoSize?: number;
}

export const QRCode: React.FC<QRCodeProps> = ({
  value,
  size = 200,
  backgroundColor = '#FFFFFF',
  color = '#000000',
  logo,
  logoSize = 40,
}) => {
  return (
    <View 
      className="items-center justify-center"
      style={{
        backgroundColor,
        padding: 16,
        borderRadius: 12,
      }}
    >
      <QRCodeSVG
        value={value}
        size={size}
        backgroundColor={backgroundColor}
        color={color}
        logo={logo}
        logoSize={logoSize}
        logoBackgroundColor={backgroundColor}
        logoMargin={2}
        logoBorderRadius={4}
        quietZone={10}
      />
    </View>
  );
};

export default QRCode;
