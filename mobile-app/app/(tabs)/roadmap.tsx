import React, { useState, useRef, useEffect } from 'react';
import { View, Text, ScrollView, TouchableOpacity } from 'react-native';
import clsx from 'clsx';

const Timeline = () => {
  const [selectedYear, setSelectedYear] = useState('Q1 2025');

  const scrollViewRef = useRef<ScrollView>(null);

  const timelineData = [
    {
      id: '1',
      year: 'Q1 2025',
      title: 'Testnet & App Launch',
      description: [
        'Token presale, fundraising & Airdrop',
        'DAO creation & Uniswap listing',
        'Launch Testnet & Chameleon app',
      ],
    },
    {
      id: '2',
      year: 'Q2 2025',
      title: 'Mainnet Launch',
      description: [
        'Code refactoring & security audits',
        'Ramp up staking infrastructure (nodes)',
        ' Launch Mainnet & Update app',
      ],
    },
    {
      id: '3',
      year: 'Q3 2025',
      title: 'Ecosystem Expansion',
      description: [
        'Enhance & Deliver Key Security Modules',
        'Cross-Chain Interoperability / Scaling',
        'Marketing & Strategic Partnerships',
      ],
    },
    {
      id: '4',
      year: 'Q4 2025',
      title: 'Governance & Community',
      description: [
        'Governance Proposals for Voting',
        'Community and Developer Growth',
        'Hardware Manufacturing Partnerships',
      ],
    },
    {
      id: '5',
      year: 'Year 2026',
      title: 'Strategic Objectives',
      description: [
        'Advanced privacy features',
        'Decentralization & reducing fixed nodes',
        'Partnerships & scaling solutions',
      ],
    },
  ];

  const handleSelectYear = (year) => {
    setSelectedYear(year);
    const index = timelineData.findIndex((item) => item.year === year);
    if (scrollViewRef.current && index !== -1) {
      scrollViewRef.current.scrollTo({ x: 0, y: index * 200, animated: true });
    }
  };

  const handleScroll = (event) => {
    const yOffset = event.nativeEvent.contentOffset.y;
    const index = Math.round(yOffset / 200);
    if (timelineData[index] && timelineData[index].year !== selectedYear) {
      setSelectedYear(timelineData[index].year);
    }
  };

  useEffect(() => {
    if (scrollViewRef.current) {
      scrollViewRef.current.scrollTo({ x: 0, y: 0, animated: true });
    }
  }, []);

  return (
    <View className="flex-1 bg-[#0C0E12] justify-center items-center p-5 gap-10 ">
      <ScrollView
        className="mb-5 px-10"
        ref={scrollViewRef}
        showsVerticalScrollIndicator={false}
        onScroll={handleScroll}
        scrollEventThrottle={16}
      >
        <View className="relative items-center pb-20">
          <View className="absolute top-0 left-[-4] h-full w-[2px] bg-[#777] z-0" />
          {timelineData.map((data) => (
            <View key={data.year} className="relative items-center mb-5">
              <TouchableOpacity
                onPress={() => handleSelectYear(data.year)}
                className={clsx(
                  'absolute top-24 left-[-4%] translate-x-0 w-5 h-5 rounded-full bg-[#444] z-10 ',
                  selectedYear === data.year && 'bg-[rgb(24,187,89)]'
                )}
                style={
                  selectedYear === data.year
                    ? {
                        boxShadow:
                          'rgba(24,187,89, 0.5) 0px 0px 0px, rgba(24,187,89, 0.5) 0px -12px 15px, rgba(24,187,89, 0.5) 0px 4px 15px, rgba(24,187,89, 0.5) 0px 12px 15px, rgba(24,187,89, 0.5) 0px -3px 15px, rgba(24,187,89, 0.5) 0px 0px 15px',
                      }
                    : {}
                }
              />
              <TouchableOpacity
                onPress={() => handleSelectYear(data.year)}
                className={clsx(
                  'items-center mb-5 rounded-lg p-4 ml-10',
                  selectedYear === data.year && 'bg-[#1b1b1b]'
                )}
              >
                <Text className="text-gray-200 text-md font-bold">{data.year}</Text>
                <Text className="text-white text-xl font-bold">{data.title}</Text>
                {data.description.map((point, index) => (
                  <View key={index} className="flex-row items-start">
                    <Text className="text-gray-400 text-xl mr-2">•</Text>
                    <Text className="text-gray-400 text-xl flex-1">{point}</Text>
                  </View>
                ))}
              </TouchableOpacity>
            </View>
          ))}
        </View>
      </ScrollView>

      <View className="relative w-full items-center bg-[#1b1b1b] rounded-2xl">
        <View className="absolute top-1/2 left-12 right-12 h-[2px] bg-[#777] z-0" />
        <ScrollView horizontal className="flex-row">
          {timelineData.map((data) => (
            <TouchableOpacity
              key={data.year}
              className={clsx(
                'bg-[#444] w-10 h-10 rounded-full mx-5 my-3 z-10 flex items-center justify-center',
                selectedYear === data.year && 'bg-[rgb(24,187,89)]'
              )}
              style={
                selectedYear === data.year
                  ? {
                      boxShadow:
                        'rgba(24,187,89, 0.4) 0px 0px 0px, rgba(24,187,89, 0.4) 0px -2px 5px, rgba(24,187,89, 0.4) 0px 2px 5px, rgba(24,187,89, 0.4) 0px 2px 5px, rgba(24,187,89, 0.4) 0px -2px 5px, rgba(24,187,89, 0.4) 0px 0px 5px',
                    }
                  : {}
              }
              onPress={() => handleSelectYear(data.year)}
            >
              <Text className="color-white text-md font-bold m-1 text-center">{data.id}</Text>
            </TouchableOpacity>
          ))}
        </ScrollView>
      </View>
    </View>
  );
};

export default Timeline;
