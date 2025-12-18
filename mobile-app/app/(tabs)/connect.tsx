import React, { useState } from 'react';
import { SafeAreaView, ScrollView, TouchableOpacity, View, Text, Image } from 'react-native';
import { Linking } from 'react-native';
import CustomCard from '@/components/CustomCard';
import MaterialCommunityIcons from '@expo/vector-icons/MaterialCommunityIcons';
import AntDesign from '@expo/vector-icons/AntDesign';
import FontAwesome5 from '@expo/vector-icons/FontAwesome5';
import FontAwesome6 from '@expo/vector-icons/FontAwesome6';
import { ApiConnectionDemo } from '@/components/ApiConnectionDemo';
import { NetworkBadge } from '@/components/NetworkBadge';

const data = [
  {
    icon: { component: FontAwesome5, name: 'forumbee', size: 30, color: '#18bb59' },
    title: 'Forum',
    description: 'Join discussions and contribute to shaping Chameleon.',
    url: 'https://forum.chml.network/', // URL to navigate to
  },
  {
    icon:  { component: FontAwesome5, name: 'telegram-plane', size: 30, color: '#18bb59' },
    title: 'Telegram',
    description: 'Stay updated with real-time announcements and chats',
    url: 'https://telegram.me/Chameleon_Network',
  },
  {
    icon:  { component: FontAwesome6, name: 'x-twitter', size: 30, color: '#18bb59' },
    title: 'Twitter (X)',
    description: 'Follow us for the latest news and updates.',
    url: 'https://x.com/CHMLnetwork',
  },
  {
    icon:  { component: FontAwesome6, name: 'discord', size: 30, color: '#18bb59' },
    title: 'Discord',
    description: 'Connect with developers, users, and community members',
    url: 'https://discord.gg/gV79PXJQGA',
  },
];

const Connect = () => {
  const [showApiDemo, setShowApiDemo] = useState(false);
  
  const handlePress = (url: string) => {
    Linking.openURL(url).catch((err) => {
      console.error('Failed to open URL:', err);
    });
  };

  const renderIcon = (icon: any) => {
    if (icon.component) {
      // Dynamic icon rendering
      const IconComponent = icon.component;
      return <IconComponent name={icon.name} size={icon.size} color={icon.color} />;
    } else if (typeof icon === 'number') {
      // Static image rendering
      return <Image source={icon} style={{ width: 30, height: 30 }} />;
    }
    return null;
  };

  return (
    <SafeAreaView className="flex-1 bg-[#0C0E12] font-poppins">
      {/* Scrollable List */}
      <ScrollView
        contentContainerStyle={{ paddingTop: 20, paddingLeft: 20, paddingRight: 20 }}
        showsVerticalScrollIndicator={false}
      >
        {/* Main Chameleon Network Card */}
        <TouchableOpacity
          onPress={() => handlePress('https://chml.network/')}
          className="font-poppins"
        >
          <View className="bg-[#18bb59] rounded-xl p-7 mb-4 shadow-md h-36 items-center justify-center gap-2">
            <Text className="text-white text-3xl font-poppins font-bold">Chameleon Network</Text>
            <View className="flex-row gap-1">
              <MaterialCommunityIcons name="web" size={24} color="white" />
              <Text className="text-md font-bold text-white pt-1 font-poppins">https://chml.network/</Text>
            </View>
          </View>
        </TouchableOpacity>

        {/* Network Badge */}
        <View className="mb-4">
          <NetworkBadge size="medium" showConnectionStatus={true} />
        </View>

        {/* API Connection Demo Toggle */}
        <TouchableOpacity
          onPress={() => setShowApiDemo(!showApiDemo)}
          className="mb-4"
        >
          <View className="bg-[#1B1B1B] rounded-xl p-4 border border-[#13E1BC]/20">
            <View className="flex-row items-center justify-between">
              <View className="flex-1">
                <Text className="text-white text-lg font-bold font-poppins">
                  🔗 Blockchain Connection
                </Text>
                <Text className="text-[#CDCDE0] text-sm font-poppins">
                  Test RPC connection and balance queries
                </Text>
              </View>
              <MaterialCommunityIcons 
                name={showApiDemo ? "chevron-up" : "chevron-down"} 
                size={24} 
                color="#13E1BC" 
              />
            </View>
          </View>
        </TouchableOpacity>

        {/* API Demo Content */}
        {showApiDemo && (
          <View className="mb-4">
            <ApiConnectionDemo />
          </View>
        )}

        {/* Community Links */}
        {data.map((item, index) => (
          <TouchableOpacity key={index} onPress={() => handlePress(item.url)}>
            <CustomCard
              title={item.title}
              description={item.description}
              icon={renderIcon(item.icon)}
              link={item.url}
            />
          </TouchableOpacity>
        ))}
      </ScrollView>
    </SafeAreaView>
  );
};

export default Connect;
