import React from 'react';
import { SafeAreaView, View, Text ,Image} from 'react-native';
import { Drawer, DrawerBackdrop, DrawerBody, DrawerHeader, DrawerContent, DrawerFooter } from '@/components/ui/drawer';
import { Heading } from '@/components/ui/heading';
import { Button as UIButton, ButtonText } from '@/components/ui/button';
import CustomCard from '@/components/CustomCard';
import AntDesign from '@expo/vector-icons/AntDesign';
import Ionicons from '@expo/vector-icons/Ionicons';
import FontAwesome from '@expo/vector-icons/FontAwesome';
import FontAwesome6 from '@expo/vector-icons/FontAwesome6';
import MaterialCommunityIcons from '@expo/vector-icons/MaterialCommunityIcons';

const data = [
  {
    icon:  { component: Ionicons, name: 'shield-checkmark', size: 28, color: '#27d769' },
    title: 'Shielding/Unshielding',
    discription: 'Protect your assets and maintain your privacy with one-click shielding and unshielding of transactions.',
  },
  {
    icon:  { component: FontAwesome, name: 'line-chart', size: 28, color: '#27d769' },  
    title: 'Privacy Trading (pDEX)',
    discription: 'Trade your assets securely and privately on Chameleon’s Privacy Decentralized Exchange (pDEX).',
  },
  {
    icon:  { component: FontAwesome6, name: 'filter-circle-dollar', size: 28, color: '#27d769' },
    title: 'Provide Liquidity',
    discription: 'Contribute liquidity to earn CHML rewards while supporting the Chameleon ecosystem.',
  },
  {
    icon:  { component: FontAwesome6, name: 'award', size: 28, color: '#27d769' },
    title: 'Staking Rewards',
    discription: 'Stake your CHML tokens to participate in network validation and earn sustainable staking rewards.',
  },
  {
    icon:  { component: MaterialCommunityIcons, name: 'transit-connection-variant', size: 28, color: '#27d769' },
    title: 'P-Node Onboarding',
    discription: 'Run plug-and-play Physical Nodes (P-Nodes) effortlessly to contribute to the network and earn validator rewards.',
  },
  {
    icon:  { component: FontAwesome, name: 'chain', size: 24, color: '#27d769' },
    title: 'Cross-Chain Bridges',
    discription: 'Transfer assets privately between Bitcoin, Ethereum, and other major blockchains through secure bridges.',
  },
  {
    icon:  { component: MaterialCommunityIcons, name: 'file-certificate', size: 28, color: '#27d769' },
    title: 'Governance Participation',
    discription: 'Have your say! Participate in DAO governance to shape the Chameleon network’s future.',
  },
  
];
const renderIcon = (icon: any) => {
  if (icon.component) {
    // Dynamic icon rendering
    const IconComponent = icon.component;
    return <IconComponent name={icon.name} size={icon.size} color={icon.color}  />;
  } else if (typeof icon === 'number') {
    // Static image rendering
    return <Image source={icon} style={{ width: 30, height: 30 }} />;
  }
  return null;
};

const DrawerComponent = ({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) => {
  return (
    <Drawer isOpen={isOpen} onClose={onClose} size="full" anchor="right">
      {/* <DrawerBackdrop /> */}
      <DrawerContent className="bg-[#0C0E12] border-outline-0">
        <SafeAreaView className='flex-1 '>
          {/* Drawer Header */}
          <DrawerHeader className='pt-10 relative justify-center items-center '>
            <UIButton onPress={onClose} className="bg-transparent top-8 active:bg-transparent focus:bg-transparent absolute left-0">
              <ButtonText onPress={onClose} className='bg-transparent p-0 active:bg-transparent focus:bg-transparent pt-2 hover:bg-transparent'>
                <AntDesign name="left" size={24} color="white" /></ButtonText>
            </UIButton>
            <Text className='text-white text-3xl font-poppins text-center font-bold '> Chameleon</Text>
          </DrawerHeader>

          {/* Drawer Body */}
          <DrawerBody className='flex-1'>
            {data.map((item, index) => (
              <CustomCard key={index} title={item.title} description={item.discription}  icon={renderIcon(item.icon)}  link={''} />
            ))}
          </DrawerBody>

      
        </SafeAreaView>
      </DrawerContent>
    </Drawer>
  );
};

export default DrawerComponent;
