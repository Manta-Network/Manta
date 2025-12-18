import React, { useState, useEffect } from 'react';
import { View, Text, Image, ScrollView, Linking, TouchableOpacity } from 'react-native';
import Svg, { Defs, LinearGradient as SvgLinearGradient, Stop, Rect } from 'react-native-svg';
import { Button } from '@/components/ui/button';
import AsyncStorage from '@react-native-async-storage/async-storage'; // To store data persistently

const data = [
  {
    title: 'PRESALE',
    description: 'LAUNCHING SOON!',
    link: 'https://tinyurl.com/chmlPresale', // Add the link to your data
  },
];

const Updates = () => {
  const [timeLeft, setTimeLeft] = useState(0); // Will store remaining time in seconds
  const [isTimerOver, setIsTimerOver] = useState(false);

  // Set the target date (e.g., 5 days from now or a specific date)
  const targetDate = Date.UTC(2025, 0, 22, 16, 0, 10); // January 22, 2025, 9:30 PM UTC
  
  

  // Effect hook to set the time when the user first opens the app
  useEffect(() => {
    const setStartTime = async () => {
      const startDate = await AsyncStorage.getItem('startDate');
      if (!startDate) {
        // Store the current date/time when the user first uses the app
        await AsyncStorage.setItem('startDate', new Date().toString());
      }
    };

    setStartTime();

    const updateTimeLeft = () => {
      const currentTime = new Date().getTime(); // Current time in milliseconds
      const timeRemaining = targetDate - currentTime; // Time difference in milliseconds

      if (timeRemaining <= 0) {
        setIsTimerOver(true); // Mark timer as over when it reaches 0
      } else {
        setTimeLeft(Math.floor(timeRemaining / 1000)); // Set the time remaining in seconds
      }
    };

    const timer = setInterval(updateTimeLeft, 1000); // Update every second

    return () => clearInterval(timer); // Cleanup timer on component unmount
  }, [targetDate]);

  // Convert seconds to DD:HH:MM:SS format
  const formatTime = () => {
    const days = String(Math.floor(timeLeft / (3600 * 24))).padStart(2, '0'); // Pad days
    const hours = String(Math.floor((timeLeft % (3600 * 24)) / 3600)).padStart(2, '0'); // Pad hours
    const minutes = String(Math.floor((timeLeft % 3600) / 60)).padStart(2, '0'); // Pad minutes
    const seconds = String(timeLeft % 60).padStart(2, '0'); // Pad seconds

    // Return the time
    return { days, hours, minutes, seconds };
  };

  return (
    <View className="flex-1 bg-[#0C0E12]">
      <ScrollView contentContainerStyle={{ alignItems: 'center', paddingBottom: 20 }}>
        {data.map((item, index) => (
          <TouchableOpacity
            key={index}
            className="bg-[#0C0E12] mb-6 p-4 w-full"
            onPress={() => Linking.openURL(item.link)} // Open the link when tapped
            activeOpacity={0.9} // Adjust touch opacity for better UX
          >
            {/* SVG Background with Gradient */}
            <Svg height="150" width="100%">
              <Defs>
                <SvgLinearGradient id="grad" x1="0" y1="0" x2="1" y2="1">
                  <Stop offset="0" stopColor="#13e1bc" stopOpacity="1" />
                  <Stop offset="0.25" stopColor="#23de2b" stopOpacity="1" />
                  <Stop offset="0.5" stopColor="#1ddf61" stopOpacity="1" />
                  <Stop offset="0.75" stopColor="#23de2b" stopOpacity="1" />
                </SvgLinearGradient>
              </Defs>
              <Rect
                x="0"
                y="0"
                width="100%"
                height="100%"
                fill="url(#grad)"
                rx="15" // Border radius for the background
                ry="15"
              />
            </Svg>

            {/* Card Content (Left side: Text, Description, Timer, or Button) */}
            <View className="absolute flex-row w-full h-full items-center justify-between px-6 gap-3 mt-4">
              {!isTimerOver ? (
                // Show the timer while countdown is active
                <View className="flex-1 pl-4 gap-2">
                  <Text className="text-black text-4xl font-bold">{item.title}</Text>
                  <Text className="text-black text-xl">{item.description}</Text>
                  <View className="flex-row gap-1 font-bold text-2xl items-center align-middle">
                    <Text className="text-black text-3xl font-bold">{formatTime().days}</Text>
                    <Text className="text-black text-3xl font-bold">:</Text>
                    <Text className="text-black text-3xl font-bold">{formatTime().hours}</Text>
                    <Text className="text-black text-3xl font-bold">:</Text>
                    <Text className="text-black text-3xl font-bold">{formatTime().minutes}</Text>
                    <Text className="text-black text-3xl font-bold">:</Text>
                    <Text className="text-black text-3xl font-bold">{formatTime().seconds}</Text>
                  </View>
                </View>
              ) : (
                // Show the "Buy Now" button and update description when the timer is over
                <View className="flex-1 pl-4 gap-2">
                  <Text className="text-black text-4xl font-bold">{item.title}</Text>
                  <Text className="text-black text-xl">IS LIVE!</Text>
                  <Button
                    className="mt-4 w-32 h-8 bg-none bg-transparent border-2 border-black rounded-full data-[active=true]:bg-transparent"
                    onPress={() => Linking.openURL('https://tinyurl.com/chmlPresale')}
                  >
                    <Text className="color-black font-bold ">Buy CHML</Text>
                  </Button>

                </View>
              )}

              {/* Right side: Image */}
              <Image
                source={require('/assets/images/Logo_black.png')} // Using dynamic image source from the input array
                className="w-32 h-32 rounded-full"
              />
            </View>
          </TouchableOpacity>
        ))}
      </ScrollView>
    </View>
  );
};

export default Updates;
