const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("ChameleonBridge", function () {
  let bridge;
  let owner;
  let user1;
  let user2;
  let validators;
  
  beforeEach(async function () {
    [owner, user1, user2, ...validators] = await ethers.getSigners();
    
    // Use first 9 accounts as validators
    const validatorAddresses = validators.slice(0, 9).map(v => v.address);
    
    const ChameleonBridge = await ethers.getContractFactory("ChameleonBridge");
    bridge = await ChameleonBridge.deploy(validatorAddresses);
    await bridge.deployed();
  });
  
  describe("Deployment", function () {
    it("Should set the correct validator count", async function () {
      expect(await bridge.validatorCount()).to.equal(9);
    });
    
    it("Should set the correct threshold", async function () {
      expect(await bridge.THRESHOLD()).to.equal(5);
    });
    
    it("Should support core tokens", async function () {
      expect(await bridge.supportedTokens("0x0000000000000000000000000000000000000000")).to.be.true;
      expect(await bridge.supportedTokens("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")).to.be.true; // USDC
    });
  });
  
  describe("Lock ETH", function () {
    it("Should lock ETH successfully", async function () {
      const amount = ethers.utils.parseEther("1.0");
      const recipient = ethers.utils.formatBytes32String("chameleon_address");
      
      await expect(bridge.connect(user1).lockETH(recipient, { value: amount }))
        .to.emit(bridge, "Locked")
        .withArgs(
          "0x0000000000000000000000000000000000000000",
          user1.address,
          recipient,
          amount,
          1
        );
      
      expect(await bridge.nonce()).to.equal(1);
    });
    
    it("Should reject amounts below minimum", async function () {
      const amount = ethers.utils.parseEther("0.0001"); // Below 0.001 ETH minimum
      const recipient = ethers.utils.formatBytes32String("chameleon_address");
      
      await expect(bridge.connect(user1).lockETH(recipient, { value: amount }))
        .to.be.revertedWith("Amount below minimum");
    });
    
    it("Should reject zero recipient", async function () {
      const amount = ethers.utils.parseEther("1.0");
      const recipient = "0x0000000000000000000000000000000000000000000000000000000000000000";
      
      await expect(bridge.connect(user1).lockETH(recipient, { value: amount }))
        .to.be.revertedWith("Invalid recipient");
    });
  });
  
  describe("Pause/Unpause", function () {
    it("Should pause and unpause correctly", async function () {
      await bridge.pause();
      
      const amount = ethers.utils.parseEther("1.0");
      const recipient = ethers.utils.formatBytes32String("chameleon_address");
      
      await expect(bridge.connect(user1).lockETH(recipient, { value: amount }))
        .to.be.revertedWith("Pausable: paused");
      
      await bridge.unpause();
      
      await expect(bridge.connect(user1).lockETH(recipient, { value: amount }))
        .to.emit(bridge, "Locked");
    });
  });
  
  describe("Validator Management", function () {
    it("Should add validator correctly", async function () {
      const newValidator = user2.address;
      
      await expect(bridge.addValidator(newValidator))
        .to.emit(bridge, "ValidatorAdded")
        .withArgs(newValidator);
      
      expect(await bridge.validators(newValidator)).to.be.true;
      expect(await bridge.validatorCount()).to.equal(10);
    });
    
    it("Should remove validator correctly", async function () {
      const validatorToRemove = validators[0].address;
      
      await expect(bridge.removeValidator(validatorToRemove))
        .to.emit(bridge, "ValidatorRemoved")
        .withArgs(validatorToRemove);
      
      expect(await bridge.validators(validatorToRemove)).to.be.false;
      expect(await bridge.validatorCount()).to.equal(8);
    });
  });
});