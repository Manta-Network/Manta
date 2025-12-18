import { Tabs } from 'expo-router';
import React, { useState } from 'react';
import { Platform, View, Text, SafeAreaView } from 'react-native';
import AntDesign from '@expo/vector-icons/AntDesign';
import Entypo from '@expo/vector-icons/Entypo';
import Fontisto from '@expo/vector-icons/Fontisto';
import MaterialCommunityIcons from '@expo/vector-icons/MaterialCommunityIcons';
import Feather from '@expo/vector-icons/Feather';
import DrawerComponent from '@/components/DrawerComponent';
import { useRouter } from 'expo-router'; // Import useRouter from expo-router
import FontAwesome5 from '@expo/vector-icons/FontAwesome5';



export default function TabLayout() { 
  const [showDrawer, setShowDrawer] = useState(false);
  const router = useRouter(); // Access router

  // Handle closing the drawer and redirecting to 'index' page (home)
  const handleCloseDrawer = () => {
    setShowDrawer(false);
    router.push('/'); // Use router.push() to navigate to the home screen
  };

  return (
    <>
      {/* Tabs Navigation */}
      <Tabs
        screenOptions={{
          tabBarShowLabel: false,
          tabBarActiveTintColor: '#18bb59',
          tabBarInactiveTintColor: '#CDCDE0',
          tabBarStyle: {
            backgroundColor: '#0c0e12',
            borderTopWidth: 0,
            height: Platform.OS === 'ios' ? 100 : 70,
            paddingVertical:10,
          },
          tabBarLabelStyle: {
            fontSize: 11,
            fontFamily: 'Poppins',
            marginTop: 5,
          },
          headerShown: true,
        }}
      >
        {/* Home */}
        <Tabs.Screen
          name="index"
          options={{
            title: '',
            tabBarIcon: ({ color, focused }) => (
              // <View style={{ borderTopWidth: focused ? 2 : 0, borderColor: '#23DE2B', alignItems: 'center', gap:1}}>
                <View className='flex-column items-center gap-0  w-100%'>
                  <View
      className={`h-[2px] w-[40px] ${focused ? 'bg-[#23DE2B]' : 'bg-transparent'}`}
    />
                <AntDesign name="home" paddingTop={3} size={26} color={color} />
                <Text className={`${focused ? 'text-[#18bb59] block' : 'text-[#CDCDE0] hidden'} text-[11px] text-center font-poppins w-[100%] }`} numberOfLines={1}>Home</Text>
                </View>
              // </View>
            ),
          }}
        />

      
        
        {/* Roadmap*/}
        <Tabs.Screen
         name="roadmap"
         options={{
           title: 'Roadmap',
           tabBarIcon: ({ color, focused }) => (
            <View className='flex-column items-center gap-0  w-100%'>
            <View
className={`h-[2px] w-[40px] ${focused ? 'bg-[#23DE2B]' : 'bg-transparent'}`}
/>
               <MaterialCommunityIcons name="map-marker-path" paddingTop={3} size={26} color={color} />
               <Text className={`${focused ? 'text-[#18bb59] block' : 'text-[#CDCDE0] hidden'} text-[11px] text-center font-poppins w-[100%] }`} numberOfLines={1}>Roadmap</Text>
             </View>

           ),
         }}
       />
        
        {/* Connect */}
        <Tabs.Screen
          name="connect"
          options={{
            title: 'Join the Community',
            tabBarIcon: ({ color, focused }) => (
                     <View className='flex-column items-center gap-0  w-100%'>
                  <View
      className={`h-[2px] w-[40px] ${focused ? 'bg-[#23DE2B]' : 'bg-transparent'}`}
    />
                <MaterialCommunityIcons name="web" paddingTop={3} size={26} color={color} />
                <Text className={`${focused ? 'text-[#18bb59] block' : 'text-[#CDCDE0] hidden'} text-[11px] text-center font-poppins w-[100%] }`} numberOfLines={1}>Connect</Text>
              </View>
            ),
          }}
        />
        {/* Updates */}
        <Tabs.Screen
          name="updates"
          options={{
            title: 'Updates',
            tabBarIcon: ({ color, focused }) => (
              <View className='flex-column items-center gap-0  w-100%'>
              <View
  className={`h-[2px] w-[40px] ${focused ? 'bg-[#23DE2B]' : 'bg-transparent'}`}
/>
                <Fontisto name="bell-alt" paddingTop={3} size={26} color={color}  />
                <Text className={`${focused ? 'text-[#18bb59] block' : 'text-[#CDCDE0] hidden'} text-[11px] text-center font-poppins w-[100%] }`} numberOfLines={1}>Updates</Text>
              </View>
            ),
          }}
        />




        {/* More (Only opens Drawer, no content displayed) */}
        <Tabs.Screen
          name="explore"
          options={{
            title: 'Explore',
            tabBarIcon: ({ color, focused }) => (
              <View className='flex-column items-center gap-0  w-100%'>
              <View
  className={`h-[2px] w-[40px] ${focused ? 'bg-[#23DE2B]' : 'bg-transparent'}`}
/>
                <FontAwesome5 name="compass" paddingTop={3} size={26} color={color} />
                <Text className={`${focused ? 'text-[#18bb59] block' : 'text-[#CDCDE0] hidden'} text-[11px] text-center font-poppins w-[100%] }`} numberOfLines={1}>Explore</Text>
              </View>
            ),
          }}
          listeners={{
            tabPress: (e) => {
              e.preventDefault(); // Prevent the default tab switching
              setShowDrawer(true); // Open the Drawer
            },
          }}
        />
      </Tabs>

      {/* Drawer Component */}
      <DrawerComponent isOpen={showDrawer} onClose={handleCloseDrawer} />
    </>
  );
}
