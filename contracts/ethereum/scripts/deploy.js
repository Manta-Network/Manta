const { ethers } = require("hardhat");

async function main() {
  console.log("Deploying ChameleonBridge contract...");
  
  // Get signers
  const [deployer] = await ethers.getSigners();
  console.log("Deploying with account:", deployer.address);
  console.log("Account balance:", (await deployer.getBalance()).toString());
  
  // Example validator addresses (replace with actual validator addresses)
  const validators = [
    "0x1234567890123456789012345678901234567890",
    "0x2345678901234567890123456789012345678901",
    "0x3456789012345678901234567890123456789012",
    "0x4567890123456789012345678901234567890123",
    "0x5678901234567890123456789012345678901234",
    "0x6789012345678901234567890123456789012345",
    "0x7890123456789012345678901234567890123456",
    "0x8901234567890123456789012345678901234567",
    "0x9012345678901234567890123456789012345678"
  ];
  
  // Deploy contract
  const ChameleonBridge = await ethers.getContractFactory("ChameleonBridge");
  const bridge = await ChameleonBridge.deploy(validators);
  
  await bridge.deployed();
  
  console.log("ChameleonBridge deployed to:", bridge.address);
  console.log("Transaction hash:", bridge.deployTransaction.hash);
  
  // Verify deployment
  console.log("\nVerifying deployment...");
  console.log("Validator count:", await bridge.validatorCount());
  console.log("Threshold:", await bridge.THRESHOLD());
  console.log("ETH supported:", await bridge.supportedTokens("0x0000000000000000000000000000000000000000"));
  console.log("USDC supported:", await bridge.supportedTokens("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"));
  
  console.log("\nDeployment completed successfully!");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });