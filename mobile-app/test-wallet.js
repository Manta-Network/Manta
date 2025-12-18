// Simple test to verify wallet functionality
const { mnemonicGenerate, mnemonicValidate, cryptoWaitReady } = require('@polkadot/util-crypto');
const { Keyring } = require('@polkadot/keyring');

async function testWallet() {
  console.log('Testing wallet functionality...');
  
  try {
    // Wait for crypto initialization
    await cryptoWaitReady();
    console.log('✅ Crypto initialized');
    
    // Test mnemonic generation
    const mnemonic = mnemonicGenerate(12);
    console.log('✅ Generated mnemonic:', mnemonic);
    
    // Test mnemonic validation
    const isValid = mnemonicValidate(mnemonic);
    console.log('✅ Mnemonic validation:', isValid);
    
    // Test keyring creation
    const keyring = new Keyring({ type: 'sr25519', ss58Format: 42 });
    const pair = keyring.addFromMnemonic(mnemonic);
    console.log('✅ Generated address:', pair.address);
    
    // Test dev account
    const devMnemonic = 'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice';
    const devPair = keyring.addFromMnemonic(devMnemonic);
    console.log('✅ Alice dev address:', devPair.address);
    
    console.log('\n🎉 All wallet tests passed!');
  } catch (error) {
    console.error('❌ Test failed:', error);
  }
}

testWallet();
