import React ,{useState}from 'react';
import { ScrollView, View, Text, Image } from 'react-native';
import { Button ,ButtonText} from '@/components/ui/button';
import Svg, { Text as SvgText, Defs, LinearGradient as SvgLinearGradient, Stop } from 'react-native-svg';
import { useRouter } from 'expo-router';
import DrawerComponent from '@/components/DrawerComponent';
import { Box} from '@/components/ui/box';
import { VStack } from '@/components/ui/vstack';
import { Badge, BadgeText } from '@/components/ui/badge';
import { Card } from '@/components/ui/card';



export default function HomeScreen() {
  const [showDrawer, setShowDrawer] = useState(false);
  const router = useRouter(); // Use router for navigation
  const handleOpenDrawer = () => {
    setShowDrawer(true); // Open drawer
  };

  const handleCloseDrawer = () => {
    setShowDrawer(false); // Close drawer
    router.push('/');
  };

  return (
     <ScrollView
     className='bg-[#0C0E12] '
            showsVerticalScrollIndicator={false}
          >
    <View className="flex-1 justify-center items-center bg-[#0C0E12] gap-4 p-10">
      {/* Regular Text */}
      <Text className="text-white text-5xl font-styled font-poppins " >Welcome to</Text>

      {/* Image */}
      <Image source={require("/assets/images/Logo.png")} className="w-[200px] h-[200px]" />
      {/* <Svg height="200" width="200">
        <Image source={require("/assets/images/logo.svg")}className='w-[200px] h-[200px]'/>
      </Svg> */}

<Box className="items-center">
  <VStack>
    <Badge
      className="z-10 self-end  bg-transparent rounded-full -mb-4 -mr-4 border-red-500"
      variant="outline"    >
      <BadgeText className="text-red-500 ">Beta</BadgeText>
    </Badge>
    
     {/* Gradient Text */}
     <Svg height="50" width="300"  >
        <Defs>
          <SvgLinearGradient id="grad" x1="0" y1="0" x2="1" y2="1">
            <Stop offset="0" stopColor="#13e1bc" stopOpacity="1" />
            <Stop offset="0.25" stopColor="#23de2b" stopOpacity="1" />
            <Stop offset="0.5" stopColor="#1ddf61" stopOpacity="1" />
            <Stop offset="0.75" stopColor="#23de2b" stopOpacity="1" />
            <Stop offset="1" stopColor="#13e1bc" stopOpacity="1" />
          </SvgLinearGradient>
        </Defs>
        <SvgText
          fill="url(#grad)"
          x="20"
          y="40"
          fontSize="50"
          fontWeight="bold"
          textAnchor="start"
        >
          Chameleon
        </SvgText>
      </Svg>
  </VStack>
</Box>

     

      {/* Paragraph */}
      <Text className="text-white text-md text-justify">
      Thank you for choosing Chameleon – a privacy-first decentralized ecosystem. Explore our features, stay updated, and join the Chameleon community as we shape the future of private transactions together.
      </Text>

      {/* Navigation Button */}
      <Button size="md" variant="solid" action="primary" className='rounded-full bg-[#18bb59] data-[active=true]:bg-[#18bb59]' onPress={handleOpenDrawer} >
        <ButtonText style={{color: 'white'}}>Explore</ButtonText>
      </Button>
          {/* Drawer Component */}
          {showDrawer && (
        <DrawerComponent isOpen={showDrawer} onClose={handleCloseDrawer} />
      )}
      <Card size="md" variant="elevated" className="bg-[#1b1b1b]  ">
        <Text className='text-gray-400  text-sm font-poppins font-bold  '>
          This app provides information about Chameleon Network. The full-featured Chameleon Wallet App will go live with the mainnet.
        </Text>
      </Card>      
    </View>
    </ScrollView>
  );
}
