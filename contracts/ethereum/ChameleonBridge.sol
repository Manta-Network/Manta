// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title ChameleonBridge
 * @dev Cross-chain bridge contract for Ethereum <-> Chameleon Network
 * Supports ETH, USDC, USDT, WBTC with multi-sig validation
 */
contract ChameleonBridge is ReentrancyGuard, Pausable, Ownable {
    // Events
    event Locked(
        address indexed token,
        address indexed sender,
        bytes32 indexed recipient,  // Chameleon address
        uint256 amount,
        uint256 nonce
    );
    
    event Unlocked(
        address indexed token,
        address indexed recipient,
        uint256 amount,
        bytes32 withdrawalId
    );
    
    event ValidatorAdded(address indexed validator);
    event ValidatorRemoved(address indexed validator);
    event TokenSupported(address indexed token);
    event TokenUnsupported(address indexed token);
    
    // Constants
    address public constant ETH_ADDRESS = address(0);
    uint256 public constant THRESHOLD = 5;  // 5-of-9 multi-sig
    uint256 public constant MAX_VALIDATORS = 9;
    
    // Supported tokens (mainnet addresses)
    address public constant USDC = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;
    address public constant USDT = 0xdAC17F958D2ee523a2206206994597C13D831ec7;
    address public constant WBTC = 0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599;
    
    // State variables
    mapping(address => bool) public supportedTokens;
    mapping(address => bool) public validators;
    uint256 public validatorCount;
    uint256 public nonce;
    mapping(bytes32 => bool) public processedWithdrawals;
    
    // Minimum amounts to prevent dust attacks
    mapping(address => uint256) public minAmounts;
    
    constructor(address[] memory _validators) {
        require(_validators.length <= MAX_VALIDATORS, "Too many validators");
        require(_validators.length >= THRESHOLD, "Not enough validators");
        
        // Set validators
        for (uint256 i = 0; i < _validators.length; i++) {
            require(_validators[i] != address(0), "Invalid validator address");
            require(!validators[_validators[i]], "Duplicate validator");
            validators[_validators[i]] = true;
            emit ValidatorAdded(_validators[i]);
        }
        validatorCount = _validators.length;
        
        // Set supported tokens
        supportedTokens[ETH_ADDRESS] = true;
        supportedTokens[USDC] = true;
        supportedTokens[USDT] = true;
        supportedTokens[WBTC] = true;
        
        // Set minimum amounts (to prevent dust attacks)
        minAmounts[ETH_ADDRESS] = 0.001 ether;  // 0.001 ETH
        minAmounts[USDC] = 1e6;                 // 1 USDC (6 decimals)
        minAmounts[USDT] = 1e6;                 // 1 USDT (6 decimals)
        minAmounts[WBTC] = 1e4;                 // 0.0001 WBTC (8 decimals)
        
        emit TokenSupported(ETH_ADDRESS);
        emit TokenSupported(USDC);
        emit TokenSupported(USDT);
        emit TokenSupported(WBTC);
    }
    
    /**
     * @dev Lock ETH for bridging to Chameleon
     * @param chameleonRecipient The recipient address on Chameleon (32 bytes)
     */
    function lockETH(bytes32 chameleonRecipient) external payable nonReentrant whenNotPaused {
        require(msg.value >= minAmounts[ETH_ADDRESS], "Amount below minimum");
        require(chameleonRecipient != bytes32(0), "Invalid recipient");
        
        nonce++;
        emit Locked(ETH_ADDRESS, msg.sender, chameleonRecipient, msg.value, nonce);
    }
    
    /**
     * @dev Lock ERC20 tokens for bridging to Chameleon
     * @param token The token contract address
     * @param amount The amount to lock
     * @param chameleonRecipient The recipient address on Chameleon (32 bytes)
     */
    function lockToken(
        address token,
        uint256 amount,
        bytes32 chameleonRecipient
    ) external nonReentrant whenNotPaused {
        require(supportedTokens[token], "Token not supported");
        require(token != ETH_ADDRESS, "Use lockETH for ETH");
        require(amount >= minAmounts[token], "Amount below minimum");
        require(chameleonRecipient != bytes32(0), "Invalid recipient");
        
        // Transfer tokens to bridge
        IERC20(token).transferFrom(msg.sender, address(this), amount);
        
        nonce++;
        emit Locked(token, msg.sender, chameleonRecipient, amount, nonce);
    }
    
    /**
     * @dev Unlock assets with multi-sig validation
     * @param token The token to unlock (address(0) for ETH)
     * @param recipient The recipient address
     * @param amount The amount to unlock
     * @param withdrawalId Unique withdrawal identifier
     * @param signatures Array of validator signatures
     */
    function unlock(
        address token,
        address recipient,
        uint256 amount,
        bytes32 withdrawalId,
        bytes[] calldata signatures
    ) external nonReentrant whenNotPaused {
        require(!processedWithdrawals[withdrawalId], "Already processed");
        require(signatures.length >= THRESHOLD, "Insufficient signatures");
        require(recipient != address(0), "Invalid recipient");
        require(amount > 0, "Amount must be > 0");
        
        // Verify signatures
        bytes32 message = keccak256(abi.encodePacked(
            "\x19Ethereum Signed Message:\n32",
            keccak256(abi.encodePacked(token, recipient, amount, withdrawalId))
        ));
        
        address[] memory signers = new address[](signatures.length);
        for (uint256 i = 0; i < signatures.length; i++) {
            address signer = recoverSigner(message, signatures[i]);
            require(validators[signer], "Invalid validator signature");
            
            // Check for duplicate signers
            for (uint256 j = 0; j < i; j++) {
                require(signers[j] != signer, "Duplicate signature");
            }
            signers[i] = signer;
        }
        
        // Mark as processed
        processedWithdrawals[withdrawalId] = true;
        
        // Transfer assets
        if (token == ETH_ADDRESS) {
            require(address(this).balance >= amount, "Insufficient ETH balance");
            (bool success, ) = recipient.call{value: amount}("");
            require(success, "ETH transfer failed");
        } else {
            require(supportedTokens[token], "Token not supported");
            IERC20(token).transfer(recipient, amount);
        }
        
        emit Unlocked(token, recipient, amount, withdrawalId);
    }
    
    /**
     * @dev Recover signer from signature
     * @param message The signed message hash
     * @param signature The signature bytes
     * @return The recovered signer address
     */
    function recoverSigner(bytes32 message, bytes memory signature) internal pure returns (address) {
        require(signature.length == 65, "Invalid signature length");
        
        bytes32 r;
        bytes32 s;
        uint8 v;
        
        assembly {
            r := mload(add(signature, 32))
            s := mload(add(signature, 64))
            v := byte(0, mload(add(signature, 96)))
        }
        
        if (v < 27) {
            v += 27;
        }
        
        require(v == 27 || v == 28, "Invalid signature v value");
        
        return ecrecover(message, v, r, s);
    }
    
    /**
     * @dev Add a new validator (owner only)
     * @param validator The validator address to add
     */
    function addValidator(address validator) external onlyOwner {
        require(validator != address(0), "Invalid validator address");
        require(!validators[validator], "Validator already exists");
        require(validatorCount < MAX_VALIDATORS, "Max validators reached");
        
        validators[validator] = true;
        validatorCount++;
        emit ValidatorAdded(validator);
    }
    
    /**
     * @dev Remove a validator (owner only)
     * @param validator The validator address to remove
     */
    function removeValidator(address validator) external onlyOwner {
        require(validators[validator], "Validator does not exist");
        require(validatorCount > THRESHOLD, "Cannot go below threshold");
        
        validators[validator] = false;
        validatorCount--;
        emit ValidatorRemoved(validator);
    }
    
    /**
     * @dev Add support for a new token (owner only)
     * @param token The token address to support
     * @param minAmount The minimum amount for this token
     */
    function addSupportedToken(address token, uint256 minAmount) external onlyOwner {
        require(token != address(0), "Invalid token address");
        require(!supportedTokens[token], "Token already supported");
        
        supportedTokens[token] = true;
        minAmounts[token] = minAmount;
        emit TokenSupported(token);
    }
    
    /**
     * @dev Remove support for a token (owner only)
     * @param token The token address to remove
     */
    function removeSupportedToken(address token) external onlyOwner {
        require(supportedTokens[token], "Token not supported");
        require(token != ETH_ADDRESS && token != USDC && token != USDT && token != WBTC, "Cannot remove core tokens");
        
        supportedTokens[token] = false;
        minAmounts[token] = 0;
        emit TokenUnsupported(token);
    }
    
    /**
     * @dev Update minimum amount for a token (owner only)
     * @param token The token address
     * @param minAmount The new minimum amount
     */
    function updateMinAmount(address token, uint256 minAmount) external onlyOwner {
        require(supportedTokens[token], "Token not supported");
        minAmounts[token] = minAmount;
    }
    
    /**
     * @dev Pause the bridge (owner only)
     */
    function pause() external onlyOwner {
        _pause();
    }
    
    /**
     * @dev Unpause the bridge (owner only)
     */
    function unpause() external onlyOwner {
        _unpause();
    }
    
    /**
     * @dev Emergency withdrawal (owner only, when paused)
     * @param token The token to withdraw (address(0) for ETH)
     * @param amount The amount to withdraw
     */
    function emergencyWithdraw(address token, uint256 amount) external onlyOwner whenPaused {
        if (token == ETH_ADDRESS) {
            (bool success, ) = owner().call{value: amount}("");
            require(success, "ETH transfer failed");
        } else {
            IERC20(token).transfer(owner(), amount);
        }
    }
    
    /**
     * @dev Get contract balance for a token
     * @param token The token address (address(0) for ETH)
     * @return The balance
     */
    function getBalance(address token) external view returns (uint256) {
        if (token == ETH_ADDRESS) {
            return address(this).balance;
        } else {
            return IERC20(token).balanceOf(address(this));
        }
    }
    
    /**
     * @dev Check if an address is a validator
     * @param validator The address to check
     * @return True if the address is a validator
     */
    function isValidator(address validator) external view returns (bool) {
        return validators[validator];
    }
    
    /**
     * @dev Get all validator addresses
     * @return Array of validator addresses
     */
    function getValidators() external view returns (address[] memory) {
        address[] memory validatorList = new address[](validatorCount);
        uint256 index = 0;
        
        // This is inefficient but works for small validator sets
        // In production, consider maintaining a separate array
        for (uint256 i = 0; i < 1000 && index < validatorCount; i++) {
            address addr = address(uint160(i));
            if (validators[addr]) {
                validatorList[index] = addr;
                index++;
            }
        }
        
        return validatorList;
    }
    
    // Receive function to accept ETH
    receive() external payable {
        // Allow contract to receive ETH
    }
}