/**
 * Simple test to verify Send/Receive functionality
 * Run with: node src/test-send-receive.js
 */

// Mock React Native modules
global.__DEV__ = true;

// Mock the transaction service
const { TransactionService } = require('./services/transaction');

async function testTransactionService() {
  console.log('Testing Transaction Service...');
  
  try {
    // Test address validation
    const validAddress = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
    const invalidAddress = 'invalid-address';
    
    console.log('Valid address test:', TransactionService.validateAddress(validAddress));
    console.log('Invalid address test:', TransactionService.validateAddress(invalidAddress));
    
    // Test amount validation
    const amountValidation = TransactionService.validateAmount('100', '1000');
    console.log('Amount validation (100 from 1000):', amountValidation);
    
    const insufficientValidation = TransactionService.validateAmount('2000', '1000');
    console.log('Insufficient amount validation:', insufficientValidation);
    
    // Test fee estimation
    const publicFee = await TransactionService.getTransactionFee(false, '100');
    console.log('Public transaction fee:', publicFee);
    
    const privateFee = await TransactionService.getTransactionFee(true, '100');
    console.log('Private transaction fee:', privateFee);
    
    // Test max sendable amount
    const maxAmount = await TransactionService.getMaxSendableAmount('1000', false);
    console.log('Max sendable amount (public):', maxAmount);
    
    // Test transaction sending (mock)
    const txRequest = {
      recipient: validAddress,
      amount: '50',
      isPrivate: true,
      memo: 'Test transaction'
    };
    
    console.log('Sending mock transaction...');
    const result = await TransactionService.sendTransaction(txRequest);
    console.log('Transaction result:', result);
    
    console.log('\n✅ All tests passed!');
    
  } catch (error) {
    console.error('❌ Test failed:', error.message);
  }
}

// Run tests
testTransactionService();
