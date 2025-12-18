import React, { useState } from 'react';
import { View, Text, TouchableOpacity, LayoutAnimation,Linking } from 'react-native';

// Custom Card Component
const CustomCard = ({ title, description, icon,link }: { title: string; description: string; icon: React.ReactNode ;link:string}) => {
  const [isExpanded, setIsExpanded] = useState(false); // State to manage expansion

  const toggleCardSize = () => {
    LayoutAnimation.configureNext(LayoutAnimation.Presets.easeInEaseOut); // Smooth animation
    setIsExpanded(!isExpanded); // Toggle expanded state
  };
  const handleCardPress = () => {
    if (link) { // Check if link is provided
      Linking.openURL(link).catch((err) => console.error('Failed to open URL:', err));
    } else {
      console.log('No link provided.');
    }
  };


  return (
    <TouchableOpacity
      activeOpacity={0.9}
      onPress={() => {
        toggleCardSize(); // Toggle expansion
        handleCardPress(); // Open the link when the card is pressed
      }} // Toggle size on press
      className={`bg-[#1b1b1b] rounded-xl p-4 mb-4 shadow-md flex-row items-center gap-5 ${
        isExpanded ? 'py-4 px-4' : ''
      }`} // Adjust padding if expanded
    >
      <View className="w-16 h-16 rounded-full flex items-center justify-center mx-auto bg-[#383838] bg-opacity-50">
        <View>{icon}</View>
      </View>

      <View className="flex-1 bg-[#1b1b1b]">
        <Text className="text-2xl text-white font-bold mb-2 font-poppins">{title}</Text>
        <Text
          numberOfLines={isExpanded ? 0 : 2} // Show full text if expanded
          className={`text-sm font-poppins ${isExpanded ? 'text-white' : 'text-white'}`} // Adjust color if expanded
        >
          {description}
        </Text>
      </View>
    </TouchableOpacity>
  );
};

export default CustomCard;
