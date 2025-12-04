// Copyright 2020-2024 Manta Network.
// Chameleon Bridge Tests - Production-Grade Cross-Chain Bridge Testing

use crate::pallet::*;
use frame_support::
    assert_noop, assert_ok,
    pallet_prelude::*,
    traits::{ConstU32, ConstU64, ConstU128, tokens::fungibles::{Inspect, Mutate}},
};
use sp_core::{H160, H256};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Mock runtime construction
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Assets: pallet_assets,
        ChameleonBridge: crate,
    }
);

// System configuration
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type RuntimeTask = ();
}

// Balances configuration
impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
}

// Assets configuration
impl pallet_assets::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u128;
    type AssetId = u128;
    type AssetIdParameter = u128;
    type Currency = Balances;
    type CreateOrigin = frame_support::traits::AsEnsureOriginWithArg<frame_system::EnsureSigned<u64>>;
    type ForceOrigin = frame_system::EnsureRoot<u64>;
    type AssetDeposit = ConstU128<100>;
    type AssetAccountDeposit = ConstU128<10>;
    type MetadataDepositBase = ConstU128<10>;
    type MetadataDepositPerByte = ConstU128<1>;
    type ApprovalDeposit = ConstU128<1>;
    type StringLimit = ConstU32<50>;
    type Freezer = ();
    type Extra = ();
    type CallbackHandle = ();
    type WeightInfo = ();
    type RemoveItemsLimit = ConstU32<1000>;
}

// Bridge configuration
impl crate::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AssetId = u128;
    type Balance = u128;
    type Assets = Assets;
    type MinConfirmations = ConstU32<3>;
    type SignatureThreshold = ConstU32<5>; // 5-of-9 validators
}

// Test constants
const ALICE: u64 = 1;
const BOB: u64 = 2;
const CHARLIE: u64 = 3;
const VALIDATOR_1: u64 = 10;
const VALIDATOR_2: u64 = 11;
const VALIDATOR_3: u64 = 12;
const VALIDATOR_4: u64 = 13;
const VALIDATOR_5: u64 = 14;
const VALIDATOR_6: u64 = 15;
const VALIDATOR_7: u64 = 16;
const VALIDATOR_8: u64 = 17;
const VALIDATOR_9: u64 = 18;

// Test Ethereum addresses
fn eth_address_alice() -> H160 {
    H160::from_slice(&[0x11; 20])
}

fn eth_address_bob() -> H160 {
    H160::from_slice(&[0x22; 20])
}

// Test Ethereum transaction hashes
fn eth_tx_hash_1() -> H256 {
    H256::from([0x01; 32])
}

fn eth_tx_hash_2() -> H256 {
    H256::from([0x02; 32])
}

// Helper function to create test environment
fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    
    // Initialize balances for test accounts
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, 1_000_000_000_000), // 1M native tokens
            (BOB, 1_000_000_000_000),
            (CHARLIE, 1_000_000_000_000),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        
        // Create wrapped assets (ETH, USDC, USDT, WBTC)
        for asset_id in [WRAPPED_ETH_ASSET_ID, WRAPPED_USDC_ASSET_ID, WRAPPED_USDT_ASSET_ID, WRAPPED_WBTC_ASSET_ID] {
            assert_ok!(Assets::create(
                RuntimeOrigin::signed(ALICE),
                asset_id,
                ALICE, // admin
                1000, // min_balance
            ));
        }
        
        // Register validators
        for validator in [VALIDATOR_1, VALIDATOR_2, VALIDATOR_3, VALIDATOR_4, VALIDATOR_5, 
                         VALIDATOR_6, VALIDATOR_7, VALIDATOR_8, VALIDATOR_9] {
            assert_ok!(ChameleonBridge::add_validator(
                RuntimeOrigin::root(),
                validator,
            ));
        }
    });
    
    ext
}

// Helper function to get asset balance
fn get_asset_balance(asset_id: u128, account: u64) -> u128 {
    Assets::balance(asset_id, &account)
}

// Helper function to get total supply
fn get_total_supply(asset_id: u128) -> u128 {
    Assets::total_supply(asset_id)
}

// ============================================================================
// REQUIRED TESTS (Minimum 3)
// ============================================================================

#[test]
fn test_deposit_mints_wrapped_tokens() {
    new_test_ext().execute_with(|| {
        let eth_tx_hash = eth_tx_hash_1();
        let recipient = ALICE;
        let asset = BridgeableAsset::ETH;
        let amount = 1_000_000_000_000u128; // 1 ETH (18 decimals)
        let wrapped_asset_id = WRAPPED_ETH_ASSET_ID;
        
        // Check initial state
        assert_eq!(get_asset_balance(wrapped_asset_id, recipient), 0);
        assert_eq!(get_total_supply(wrapped_asset_id), 0);
        
        // Validators report deposit (need 5 approvals for threshold)
        for validator in [VALIDATOR_1, VALIDATOR_2, VALIDATOR_3, VALIDATOR_4] {
            assert_ok!(ChameleonBridge::report_deposit(
                RuntimeOrigin::signed(validator),
                eth_tx_hash,
                recipient,
                asset,
                amount,
            ));
            
            // Should not mint yet (threshold not reached)
            assert_eq!(get_asset_balance(wrapped_asset_id, recipient), 0);
        }
        
        // 5th validator approval should trigger minting
        assert_ok!(ChameleonBridge::report_deposit(
            RuntimeOrigin::signed(VALIDATOR_5),
            eth_tx_hash,
            recipient,
            asset,
            amount,
        ));
        
        // Verify wrapped tokens minted to recipient
        assert_eq!(get_asset_balance(wrapped_asset_id, recipient), amount);
        
        // Verify total supply increased
        assert_eq!(get_total_supply(wrapped_asset_id), amount);
        
        // Verify deposit marked as processed
        assert!(ProcessedDeposits::<Test>::get(eth_tx_hash));
        
        // Verify total bridged updated
        assert_eq!(ChameleonBridge::total_bridged(asset), amount);
        
        // Verify events emitted
        System::assert_has_event(RuntimeEvent::ChameleonBridge(Event::DepositMinted {
            eth_tx_hash,
            recipient,
            asset,
            amount,
        }));
    });
}

#[test]
fn test_withdrawal_burns_wrapped_tokens() {
    new_test_ext().execute_with(|| {
        let asset = BridgeableAsset::USDC;
        let amount = 1000_000_000u128; // 1000 USDC (6 decimals)
        let wrapped_asset_id = WRAPPED_USDC_ASSET_ID;
        let eth_destination = eth_address_alice();
        
        // First mint some wrapped tokens to Alice
        assert_ok!(Assets::mint(
            RuntimeOrigin::signed(ALICE),
            wrapped_asset_id,
            ALICE,
            amount,
        ));
        
        // Verify initial balance
        assert_eq!(get_asset_balance(wrapped_asset_id, ALICE), amount);
        assert_eq!(get_total_supply(wrapped_asset_id), amount);
        
        // Initiate withdrawal (should burn tokens immediately)
        assert_ok!(ChameleonBridge::initiate_withdrawal(
            RuntimeOrigin::signed(ALICE),
            asset,
            amount,
            eth_destination,
        ));
        
        // Verify wrapped tokens burned
        assert_eq!(get_asset_balance(wrapped_asset_id, ALICE), 0);
        
        // Verify total supply decreased
        assert_eq!(get_total_supply(wrapped_asset_id), 0);
        
        // Verify withdrawal record created
        let withdrawal_id = 0; // First withdrawal
        let withdrawal = ChameleonBridge::pending_withdrawals(withdrawal_id).unwrap();
        assert_eq!(withdrawal.from, ALICE);
        assert_eq!(withdrawal.asset, asset);
        assert_eq!(withdrawal.amount, amount);
        assert_eq!(withdrawal.eth_destination, eth_destination);
        assert_eq!(withdrawal.status, WithdrawalStatus::Pending);
        
        // Verify event emitted
        System::assert_has_event(RuntimeEvent::ChameleonBridge(Event::WithdrawalInitiated {
            withdrawal_id,
            from: ALICE,
            eth_destination,
            asset,
            amount,
        }));
    });
}

#[test]
fn test_threshold_approval_required() {
    new_test_ext().execute_with(|| {
        let eth_tx_hash = eth_tx_hash_2();
        let recipient = BOB;
        let asset = BridgeableAsset::WBTC;
        let amount = 100_000_000u128; // 1 WBTC (8 decimals)
        let wrapped_asset_id = WRAPPED_WBTC_ASSET_ID;
        
        // Test: Less than 5 approvals should be insufficient
        for validator in [VALIDATOR_1, VALIDATOR_2, VALIDATOR_3, VALIDATOR_4] {
            assert_ok!(ChameleonBridge::report_deposit(
                RuntimeOrigin::signed(validator),
                eth_tx_hash,
                recipient,
                asset,
                amount,
            ));
        }
        
        // Should not mint yet (only 4 approvals, need 5)
        assert_eq!(get_asset_balance(wrapped_asset_id, recipient), 0);
        assert_eq!(get_total_supply(wrapped_asset_id), 0);
        assert!(!ProcessedDeposits::<Test>::get(eth_tx_hash));
        
        // Test: 5th approval should be sufficient
        assert_ok!(ChameleonBridge::report_deposit(
            RuntimeOrigin::signed(VALIDATOR_5),
            eth_tx_hash,
            recipient,
            asset,
            amount,
        ));
        
        // Now should mint (threshold reached)
        assert_eq!(get_asset_balance(wrapped_asset_id, recipient), amount);
        assert_eq!(get_total_supply(wrapped_asset_id), amount);
        assert!(ProcessedDeposits::<Test>::get(eth_tx_hash));
        
        // Test: Additional approvals should be rejected (already processed)
        assert_noop!(
            ChameleonBridge::report_deposit(
                RuntimeOrigin::signed(VALIDATOR_6),
                eth_tx_hash,
                recipient,
                asset,
                amount,
            ),
            Error::<Test>::DepositAlreadyProcessed
        );
    });
}

// ============================================================================
// ADDITIONAL COMPREHENSIVE TESTS
// ============================================================================

#[test]
fn test_withdrawal_signature_threshold() {
    new_test_ext().execute_with(|| {
        let asset = BridgeableAsset::USDT;
        let amount = 1000_000_000u128; // 1000 USDT (6 decimals)
        let wrapped_asset_id = WRAPPED_USDT_ASSET_ID;
        
        // Mint tokens and initiate withdrawal
        assert_ok!(Assets::mint(
            RuntimeOrigin::signed(ALICE),
            wrapped_asset_id,
            BOB,
            amount,
        ));
        
        assert_ok!(ChameleonBridge::initiate_withdrawal(
            RuntimeOrigin::signed(BOB),
            asset,
            amount,
            eth_address_bob(),
        ));
        
        let withdrawal_id = 0;
        
        // Test: Less than 5 signatures insufficient
        for validator in [VALIDATOR_1, VALIDATOR_2, VALIDATOR_3, VALIDATOR_4] {
            assert_ok!(ChameleonBridge::sign_withdrawal(
                RuntimeOrigin::signed(validator),
                withdrawal_id,
            ));
        }
        
        let withdrawal = ChameleonBridge::pending_withdrawals(withdrawal_id).unwrap();
        assert_eq!(withdrawal.signature_count, 4);
        assert_eq!(withdrawal.status, WithdrawalStatus::Pending);
        
        // Test: 5th signature should make it ready
        assert_ok!(ChameleonBridge::sign_withdrawal(
            RuntimeOrigin::signed(VALIDATOR_5),
            withdrawal_id,
        ));
        
        let withdrawal = ChameleonBridge::pending_withdrawals(withdrawal_id).unwrap();
        assert_eq!(withdrawal.signature_count, 5);
        assert_eq!(withdrawal.status, WithdrawalStatus::ReadyToExecute);
        
        // Verify event emitted
        System::assert_has_event(RuntimeEvent::ChameleonBridge(Event::WithdrawalReady {
            withdrawal_id,
        }));
    });
}

#[test]
fn test_duplicate_validator_approval_rejected() {
    new_test_ext().execute_with(|| {
        let eth_tx_hash = H256::from([0x03; 32]);
        let recipient = CHARLIE;
        let asset = BridgeableAsset::ETH;
        let amount = 500_000_000_000u128;
        
        // First approval succeeds
        assert_ok!(ChameleonBridge::report_deposit(
            RuntimeOrigin::signed(VALIDATOR_1),
            eth_tx_hash,
            recipient,
            asset,
            amount,
        ));
        
        // Duplicate approval from same validator should fail
        assert_noop!(
            ChameleonBridge::report_deposit(
                RuntimeOrigin::signed(VALIDATOR_1),
                eth_tx_hash,
                recipient,
                asset,
                amount,
            ),
            Error::<Test>::AlreadyApproved
        );
    });
}

#[test]
fn test_non_validator_cannot_report() {
    new_test_ext().execute_with(|| {
        let eth_tx_hash = H256::from([0x04; 32]);
        let recipient = ALICE;
        let asset = BridgeableAsset::ETH;
        let amount = 1_000_000_000_000u128;
        
        // Non-validator (regular user) cannot report deposits
        assert_noop!(
            ChameleonBridge::report_deposit(
                RuntimeOrigin::signed(ALICE), // Not a validator
                eth_tx_hash,
                recipient,
                asset,
                amount,
            ),
            Error::<Test>::NotValidator
        );
    });
}

#[test]
fn test_insufficient_balance_withdrawal_fails() {
    new_test_ext().execute_with(|| {
        let asset = BridgeableAsset::ETH;
        let amount = 1_000_000_000_000u128;
        
        // Alice has no wrapped ETH, withdrawal should fail
        assert_noop!(
            ChameleonBridge::initiate_withdrawal(
                RuntimeOrigin::signed(ALICE),
                asset,
                amount,
                eth_address_alice(),
            ),
            Error::<Test>::TokenOperationFailed
        );
    });
}

#[test]
fn test_bridge_pause_functionality() {
    new_test_ext().execute_with(|| {
        // Pause bridge
        assert_ok!(ChameleonBridge::pause_bridge(RuntimeOrigin::root()));
        
        // Operations should fail when paused
        assert_noop!(
            ChameleonBridge::report_deposit(
                RuntimeOrigin::signed(VALIDATOR_1),
                H256::from([0x05; 32]),
                ALICE,
                BridgeableAsset::ETH,
                1_000_000_000_000u128,
            ),
            Error::<Test>::BridgePaused
        );
        
        assert_noop!(
            ChameleonBridge::initiate_withdrawal(
                RuntimeOrigin::signed(ALICE),
                BridgeableAsset::ETH,
                1_000_000_000_000u128,
                eth_address_alice(),
            ),
            Error::<Test>::BridgePaused
        );
        
        // Resume bridge
        assert_ok!(ChameleonBridge::resume_bridge(RuntimeOrigin::root()));
        
        // Operations should work again
        assert_ok!(ChameleonBridge::report_deposit(
            RuntimeOrigin::signed(VALIDATOR_1),
            H256::from([0x06; 32]),
            ALICE,
            BridgeableAsset::ETH,
            1_000_000_000_000u128,
        ));
    });
}

#[test]
fn test_all_bridgeable_assets() {
    new_test_ext().execute_with(|| {
        let assets = [
            (BridgeableAsset::ETH, WRAPPED_ETH_ASSET_ID, 1_000_000_000_000u128), // 1 ETH
            (BridgeableAsset::USDC, WRAPPED_USDC_ASSET_ID, 1000_000_000u128),    // 1000 USDC
            (BridgeableAsset::USDT, WRAPPED_USDT_ASSET_ID, 500_000_000u128),     // 500 USDT
            (BridgeableAsset::WBTC, WRAPPED_WBTC_ASSET_ID, 100_000_000u128),     // 1 WBTC
        ];
        
        for (i, (asset, wrapped_id, amount)) in assets.iter().enumerate() {
            let eth_tx_hash = H256::from([i as u8 + 0x10; 32]);
            
            // Test deposit flow for each asset
            for validator in [VALIDATOR_1, VALIDATOR_2, VALIDATOR_3, VALIDATOR_4, VALIDATOR_5] {
                assert_ok!(ChameleonBridge::report_deposit(
                    RuntimeOrigin::signed(validator),
                    eth_tx_hash,
                    ALICE,
                    *asset,
                    *amount,
                ));
            }
            
            // Verify minting worked
            assert_eq!(get_asset_balance(*wrapped_id, ALICE), *amount);
            
            // Test withdrawal flow
            assert_ok!(ChameleonBridge::initiate_withdrawal(
                RuntimeOrigin::signed(ALICE),
                *asset,
                *amount,
                eth_address_alice(),
            ));
            
            // Verify burning worked
            assert_eq!(get_asset_balance(*wrapped_id, ALICE), 0);
        }
    });
}

#[test]
fn test_validator_management() {
    new_test_ext().execute_with(|| {
        let new_validator = 100u64;
        
        // Initially not a validator
        assert!(!ChameleonBridge::validators(new_validator));
        
        // Add validator (root only)
        assert_ok!(ChameleonBridge::add_validator(
            RuntimeOrigin::root(),
            new_validator,
        ));
        
        // Now is a validator
        assert!(ChameleonBridge::validators(new_validator));
        
        // Can report deposits
        assert_ok!(ChameleonBridge::report_deposit(
            RuntimeOrigin::signed(new_validator),
            H256::from([0x07; 32]),
            ALICE,
            BridgeableAsset::ETH,
            1_000_000_000_000u128,
        ));
        
        // Remove validator
        assert_ok!(ChameleonBridge::remove_validator(
            RuntimeOrigin::root(),
            new_validator,
        ));
        
        // No longer a validator
        assert!(!ChameleonBridge::validators(new_validator));
        
        // Cannot report deposits anymore
        assert_noop!(
            ChameleonBridge::report_deposit(
                RuntimeOrigin::signed(new_validator),
                H256::from([0x08; 32]),
                ALICE,
                BridgeableAsset::ETH,
                1_000_000_000_000u128,
            ),
            Error::<Test>::NotValidator
        );
    });
}

#[test]
fn test_zero_amount_rejected() {
    new_test_ext().execute_with(|| {
        // Zero amount deposit should fail
        assert_noop!(
            ChameleonBridge::report_deposit(
                RuntimeOrigin::signed(VALIDATOR_1),
                H256::from([0x09; 32]),
                ALICE,
                BridgeableAsset::ETH,
                0u128,
            ),
            Error::<Test>::InvalidAmount
        );
        
        // Zero amount withdrawal should fail
        assert_noop!(
            ChameleonBridge::initiate_withdrawal(
                RuntimeOrigin::signed(ALICE),
                BridgeableAsset::ETH,
                0u128,
                eth_address_alice(),
            ),
            Error::<Test>::InvalidAmount
        );
    });
}
