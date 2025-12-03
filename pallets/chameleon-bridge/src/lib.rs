// Copyright 2020-2024 Manta Network.
// This file is part of Manta.
//
// Manta is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Manta is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Manta.  If not, see <http://www.gnu.org/licenses/>.

//! # Chameleon Bridge Pallet
//!
//! The Chameleon Bridge pallet enables secure cross-chain asset transfers between
//! Ethereum and the Chameleon Network. It supports bridging of ETH, USDC, USDT,
//! and WBTC through a trustless multi-signature validation system.
//!
//! ## Overview
//!
//! The bridge operates in two directions:
//!
//! ### Lock & Mint (Ethereum → Chameleon)
//! 1. User locks assets on Ethereum smart contract
//! 2. Ethereum emits `AssetLocked` event
//! 3. Chameleon validators observe and verify the event
//! 4. 5-of-9 validators must confirm the deposit
//! 5. Wrapped tokens are minted to user's Chameleon account
//!
//! ### Burn & Unlock (Chameleon → Ethereum)
//! 1. User burns wrapped tokens on Chameleon
//! 2. Withdrawal request is created
//! 3. 5-of-9 validators sign the unlock transaction
//! 4. Multi-sig transaction unlocks original assets on Ethereum
//!
//! ## Security Features
//!
//! - **Multi-Signature Validation**: 5-of-9 validator threshold
//! - **Fraud Proof System**: 6-hour challenge window for disputed transactions
//! - **Confirmation Requirements**: Multiple block confirmations before processing
//! - **Emergency Pause**: Ability to halt bridge operations if needed
//!
//! ## Supported Assets
//!
//! - **ETH** → **wETH** (Wrapped Ethereum)
//! - **USDC** → **cUSDC** (Chameleon USDC)
//! - **USDT** → **cUSDT** (Chameleon USDT)
//! - **WBTC** → **cWBTC** (Chameleon Wrapped Bitcoin)

#![cfg_attr(not(feature = "std"), no_std)]

pub mod ethereum;
pub mod types;
pub mod verification;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet::*;
pub use types::{
    BridgeableAsset, BridgeDeposit, BridgeWithdrawal, BridgeValidator, 
    ValidatorReputation, BridgeFee, BridgeStats, BridgeError,
    DepositStatus, WithdrawalStatus, FraudType, FraudProof,
    DefaultBridgeFeeCalculator, BridgeFeeCalculator,
};
pub use weights::WeightInfo;

/// Chameleon Bridge Pallet
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::{
        ethereum::*,
        types::*,
        verification::*,
        weights::WeightInfo,
    };
    use frame_support::{
        pallet_prelude::*,
        traits::{
            fungibles::{Create, Inspect, Mutate},
            tokens::Preservation,
            Get, StorageVersion,
        },
        PalletId,
    };
    use frame_system::pallet_prelude::*;
    use manta_primitives::{
        types::{Balance, BlockNumber},
    };
    use sp_core::H256;
    use sp_runtime::{
        traits::{AccountIdConversion, Saturating, Zero},
    };
    use sp_std::vec::Vec;

    /// Storage Version
    pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    /// Pallet Configuration
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The currency used for fees and rewards
        type Currency: frame_support::traits::Currency<Self::AccountId>;

        /// The assets pallet for managing wrapped tokens
        type Assets: Create<Self::AccountId, AssetId = AssetId, Balance = Balance>
            + Inspect<Self::AccountId, AssetId = AssetId, Balance = Balance>
            + Mutate<Self::AccountId, AssetId = AssetId, Balance = Balance>;

        /// The bridge pallet identifier
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Minimum number of Ethereum confirmations required
        #[pallet::constant]
        type ConfirmationBlocks: Get<u32>;

        /// Number of validator signatures required for withdrawals (5-of-9)
        #[pallet::constant]
        type ValidatorThreshold: Get<u32>;

        /// Challenge period for fraud proofs (in blocks)
        #[pallet::constant]
        type ChallengePeriod: Get<BlockNumber>;

        /// Weight information for extrinsics
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// Pending deposits awaiting confirmation
    #[pallet::storage]
    #[pallet::getter(fn pending_deposits)]
    pub type PendingDeposits<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        EthereumTxHash,
        BridgeDeposit<T::AccountId>,
    >;

    /// Pending withdrawals awaiting validator signatures
    #[pallet::storage]
    #[pallet::getter(fn pending_withdrawals)]
    pub type PendingWithdrawals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        WithdrawalId,
        BridgeWithdrawal<T::AccountId>,
    >;

    /// Next withdrawal ID
    #[pallet::storage]
    #[pallet::getter(fn next_withdrawal_id)]
    pub type NextWithdrawalId<T: Config> = StorageValue<_, WithdrawalId, ValueQuery>;

    /// Bridge validators
    #[pallet::storage]
    #[pallet::getter(fn bridge_validators)]
    pub type BridgeValidators<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BridgeValidator<T::AccountId>,
    >;

    /// Validator reputations
    #[pallet::storage]
    #[pallet::getter(fn validator_reputations)]
    pub type ValidatorReputations<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ValidatorReputation,
    >;

    /// Bridge configuration
    #[pallet::storage]
    #[pallet::getter(fn bridge_config)]
    pub type BridgeConfig<T: Config> = StorageValue<_, types::BridgeConfig, ValueQuery>;

    /// Ethereum confirmation tracker
    #[pallet::storage]
    #[pallet::getter(fn ethereum_confirmations)]
    pub type EthereumConfirmations<T: Config> = StorageValue<_, EthereumConfirmationTracker, ValueQuery>;

    /// Processed deposits (to prevent double processing)
    #[pallet::storage]
    #[pallet::getter(fn processed_deposits)]
    pub type ProcessedDeposits<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        EthereumTxHash,
        bool,
        ValueQuery,
    >;

    /// Executed withdrawals
    #[pallet::storage]
    #[pallet::getter(fn executed_withdrawals)]
    pub type ExecutedWithdrawals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        WithdrawalId,
        bool,
        ValueQuery,
    >;

    /// Fraud proofs
    #[pallet::storage]
    #[pallet::getter(fn fraud_proofs)]
    pub type FraudProofs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256,
        FraudProof<T::AccountId>,
    >;

    /// Bridge statistics
    #[pallet::storage]
    #[pallet::getter(fn bridge_stats)]
    pub type BridgeStats<T: Config> = StorageValue<_, types::BridgeStats, ValueQuery>;

    /// Genesis configuration
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// Initial bridge validators
        pub validators: Vec<(T::AccountId, EthereumAddress)>,
        /// Initial bridge configuration
        pub bridge_config: types::BridgeConfig,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                validators: Vec::new(),
                bridge_config: types::BridgeConfig {
                    min_confirmations: 12,
                    signature_threshold: 5,
                    max_single_deposit: 1000 * 1_000_000_000_000_000_000, // 1000 ETH equivalent
                    daily_limit: 10000 * 1_000_000_000_000_000_000,       // 10000 ETH equivalent
                    is_paused: false,
                    challenge_period: 3600, // 6 hours in blocks (6s per block)
                },
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // Set bridge configuration
            BridgeConfig::<T>::put(&self.bridge_config);

            // Initialize Ethereum confirmation tracker
            let tracker = EthereumConfirmationTracker::new(self.bridge_config.min_confirmations);
            EthereumConfirmations::<T>::put(tracker);

            // Add initial validators
            for (account, eth_address) in &self.validators {
                let validator = BridgeValidator {
                    account: account.clone(),
                    eth_address: *eth_address,
                    is_active: true,
                    added_at: Zero::zero(),
                };
                BridgeValidators::<T>::insert(account, validator);

                // Initialize reputation
                ValidatorReputations::<T>::insert(account, ValidatorReputation::default());
            }
        }
    }

    /// Bridge events
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Lock event reported by validator
        LockEventReported {
            tx_hash: EthereumTxHash,
            validator: T::AccountId,
            recipient: T::AccountId,
            asset: BridgeableAsset,
            amount: Balance,
        },
        /// Deposit confirmed and tokens minted
        DepositConfirmed {
            tx_hash: EthereumTxHash,
            recipient: T::AccountId,
            asset: BridgeableAsset,
            amount: Balance,
            wrapped_asset_id: AssetId,
        },
        /// Withdrawal initiated
        WithdrawalInitiated {
            withdrawal_id: WithdrawalId,
            from: T::AccountId,
            asset: BridgeableAsset,
            amount: Balance,
            eth_destination: EthereumAddress,
            fee: Balance,
        },
        /// Withdrawal signed by validator
        WithdrawalSigned {
            withdrawal_id: WithdrawalId,
            validator: T::AccountId,
        },
        /// Withdrawal ready for execution
        WithdrawalReady {
            withdrawal_id: WithdrawalId,
        },
        /// Withdrawal executed
        WithdrawalExecuted {
            withdrawal_id: WithdrawalId,
            eth_tx_hash: EthereumTxHash,
        },
        /// Validator added
        ValidatorAdded {
            account: T::AccountId,
            eth_address: EthereumAddress,
        },
        /// Validator removed
        ValidatorRemoved {
            account: T::AccountId,
        },
        /// Bridge paused
        BridgePaused,
        /// Bridge resumed
        BridgeResumed,
        /// Fraud proof submitted
        FraudProofSubmitted {
            challenger: T::AccountId,
            challenged_tx: H256,
            fraud_type: FraudType,
        },
        /// Bridge configuration updated
        BridgeConfigUpdated,
    }

    /// Bridge errors
    #[pallet::error]
    pub enum Error<T> {
        /// Not a bridge validator
        NotValidator,
        /// Bridge is paused
        BridgePaused,
        /// Deposit not found
        DepositNotFound,
        /// Withdrawal not found
        WithdrawalNotFound,
        /// Asset not supported
        UnsupportedAsset,
        /// Insufficient balance
        InsufficientBalance,
        /// Amount exceeds limits
        AmountExceedsLimit,
        /// Invalid Ethereum address
        InvalidEthereumAddress,
        /// Insufficient confirmations
        InsufficientConfirmations,
        /// Duplicate signature
        DuplicateSignature,
        /// Invalid signature
        InvalidSignature,
        /// Deposit already processed
        DepositAlreadyProcessed,
        /// Withdrawal already executed
        WithdrawalAlreadyExecuted,
        /// Validator already exists
        ValidatorAlreadyExists,
        /// Validator not found
        ValidatorNotFound,
        /// Arithmetic overflow
        ArithmeticOverflow,
        /// Asset creation failed
        AssetCreationFailed,
        /// Asset minting failed
        AssetMintingFailed,
        /// Asset burning failed
        AssetBurningFailed,
    }

    /// Bridge extrinsics
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Report a lock event observed on Ethereum
        ///
        /// This function is called by validators when they observe an AssetLocked
        /// event on the Ethereum bridge contract.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::report_lock_event())]
        pub fn report_lock_event(
            origin: OriginFor<T>,
            eth_tx_hash: EthereumTxHash,
            eth_address: EthereumAddress,
            recipient: T::AccountId,
            asset: BridgeableAsset,
            amount: Balance,
        ) -> DispatchResult {
            let validator = ensure_signed(origin)?;
            
            // Check if bridge is paused
            let config = Self::bridge_config();
            ensure!(!config.is_paused, Error::<T>::BridgePaused);
            
            // Verify caller is a validator
            let validator_info = Self::bridge_validators(&validator)
                .ok_or(Error::<T>::NotValidator)?;
            ensure!(validator_info.is_active, Error::<T>::NotValidator);
            
            // Check if deposit already processed
            ensure!(!Self::processed_deposits(&eth_tx_hash), Error::<T>::DepositAlreadyProcessed);
            
            // Verify amount is within limits
            TransactionVerifier::verify_deposit_amount(amount, &asset)
                .map_err(|_| Error::<T>::AmountExceedsLimit)?;
            
            let current_block = frame_system::Pallet::<T>::block_number();
            
            // Get or create deposit
            let mut deposit = Self::pending_deposits(&eth_tx_hash).unwrap_or_else(|| {
                BridgeDeposit {
                    eth_tx_hash,
                    eth_address,
                    recipient: recipient.clone(),
                    asset: asset.clone(),
                    amount,
                    eth_block_number: 0, // TODO: Get from Ethereum
                    confirmations_required: config.min_confirmations,
                    confirmations: 0,
                    status: DepositStatus::Pending,
                    validator_confirmations: Vec::new(),
                    reported_at: current_block,
                }
            });
            
            // Add validator confirmation if not already present
            if !deposit.validator_confirmations.contains(&validator) {
                deposit.validator_confirmations.push(validator.clone());
                
                // Update validator reputation
                Self::update_validator_reputation(&validator, true);
            }
            
            // Check if we have enough confirmations
            let active_validators = Self::get_active_validators();
            if MultiSigValidator::<T>::verify_deposit_confirmations(
                &deposit,
                &active_validators,
                config.signature_threshold,
            ) {
                deposit.status = DepositStatus::Confirmed;
                
                // Mint wrapped tokens
                Self::mint_wrapped_tokens(&deposit)?;
                deposit.status = DepositStatus::Minted;
                
                // Mark as processed
                ProcessedDeposits::<T>::insert(&eth_tx_hash, true);
                
                // Update statistics
                Self::update_bridge_stats(true, amount);
                
                Self::deposit_event(Event::DepositConfirmed {
                    tx_hash: eth_tx_hash,
                    recipient: deposit.recipient.clone(),
                    asset: asset.clone(),
                    amount,
                    wrapped_asset_id: asset.wrapped_asset_id(),
                });
            }
            
            // Store updated deposit
            PendingDeposits::<T>::insert(&eth_tx_hash, &deposit);
            
            Self::deposit_event(Event::LockEventReported {
                tx_hash: eth_tx_hash,
                validator,
                recipient,
                asset,
                amount,
            });
            
            Ok(())
        }

        /// Burn wrapped tokens and initiate withdrawal to Ethereum
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::burn_for_unlock())]
        pub fn burn_for_unlock(
            origin: OriginFor<T>,
            asset: BridgeableAsset,
            amount: Balance,
            eth_destination: EthereumAddress,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Check if bridge is paused
            let config = Self::bridge_config();
            ensure!(!config.is_paused, Error::<T>::BridgePaused);
            
            // Verify amount is within limits
            TransactionVerifier::verify_deposit_amount(amount, &asset)
                .map_err(|_| Error::<T>::AmountExceedsLimit)?;
            
            // Calculate fee
            let fee = DefaultBridgeFeeCalculator::calculate_unshielding_fee(amount);
            
            // Check user has enough balance for amount + fee
            let wrapped_asset_id = asset.wrapped_asset_id();
            let user_balance = T::Assets::balance(wrapped_asset_id, &who);
            let total_required = amount.saturating_add(fee.total_fee);
            ensure!(user_balance >= total_required, Error::<T>::InsufficientBalance);
            
            // Burn the wrapped tokens
            T::Assets::burn_from(
                wrapped_asset_id,
                &who,
                amount,
                Preservation::Expendable,
                frame_support::traits::tokens::Precision::Exact,
                frame_support::traits::tokens::Fortitude::Polite,
            ).map_err(|_| Error::<T>::AssetBurningFailed)?;
            
            // Collect fee
            T::Assets::burn_from(
                wrapped_asset_id,
                &who,
                fee.total_fee,
                Preservation::Expendable,
                frame_support::traits::tokens::Precision::Exact,
                frame_support::traits::tokens::Fortitude::Polite,
            ).map_err(|_| Error::<T>::AssetBurningFailed)?;
            
            // Create withdrawal request
            let withdrawal_id = Self::next_withdrawal_id();
            let current_block = frame_system::Pallet::<T>::block_number();
            
            let withdrawal = BridgeWithdrawal {
                withdrawal_id,
                from: who.clone(),
                eth_destination,
                asset: asset.clone(),
                amount,
                signatures: Vec::new(),
                signatures_required: config.signature_threshold,
                status: WithdrawalStatus::Pending,
                initiated_at: current_block,
                fee_paid: fee.total_fee,
            };
            
            PendingWithdrawals::<T>::insert(withdrawal_id, &withdrawal);
            NextWithdrawalId::<T>::put(withdrawal_id.saturating_add(1));
            
            // Update statistics
            Self::update_bridge_stats(false, amount);
            
            Self::deposit_event(Event::WithdrawalInitiated {
                withdrawal_id,
                from: who,
                asset,
                amount,
                eth_destination,
                fee: fee.total_fee,
            });
            
            Ok(())
        }

        /// Sign a withdrawal request
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::sign_withdrawal())]
        pub fn sign_withdrawal(
            origin: OriginFor<T>,
            withdrawal_id: WithdrawalId,
            signature: ValidatorSignature,
        ) -> DispatchResult {
            let validator = ensure_signed(origin)?;
            
            // Verify caller is a validator
            let validator_info = Self::bridge_validators(&validator)
                .ok_or(Error::<T>::NotValidator)?;
            ensure!(validator_info.is_active, Error::<T>::NotValidator);
            
            // Get withdrawal
            let mut withdrawal = Self::pending_withdrawals(withdrawal_id)
                .ok_or(Error::<T>::WithdrawalNotFound)?;
            
            // Check if already executed
            ensure!(!Self::executed_withdrawals(withdrawal_id), Error::<T>::WithdrawalAlreadyExecuted);
            
            // Check if validator already signed
            let already_signed = withdrawal.signatures.iter()
                .any(|sig| sig.validator == validator);
            ensure!(!already_signed, Error::<T>::DuplicateSignature);
            
            let current_block = frame_system::Pallet::<T>::block_number();
            
            // Add signature
            withdrawal.signatures.push(ValidatorWithdrawalSignature {
                validator: validator.clone(),
                signature,
                signed_at: current_block,
            });
            
            // Update validator reputation
            Self::update_validator_reputation(&validator, true);
            
            // Check if we have enough signatures
            if withdrawal.signatures.len() >= withdrawal.signatures_required as usize {
                withdrawal.status = WithdrawalStatus::ReadyToExecute;
                
                Self::deposit_event(Event::WithdrawalReady {
                    withdrawal_id,
                });
            }
            
            PendingWithdrawals::<T>::insert(withdrawal_id, &withdrawal);
            
            Self::deposit_event(Event::WithdrawalSigned {
                withdrawal_id,
                validator,
            });
            
            Ok(())
        }

        /// Add a new bridge validator
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::add_validator())]
        pub fn add_validator(
            origin: OriginFor<T>,
            account: T::AccountId,
            eth_address: EthereumAddress,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            // Check if validator already exists
            ensure!(!BridgeValidators::<T>::contains_key(&account), Error::<T>::ValidatorAlreadyExists);
            
            let current_block = frame_system::Pallet::<T>::block_number();
            
            let validator = BridgeValidator {
                account: account.clone(),
                eth_address,
                is_active: true,
                added_at: current_block,
            };
            
            BridgeValidators::<T>::insert(&account, validator);
            ValidatorReputations::<T>::insert(&account, ValidatorReputation::default());
            
            Self::deposit_event(Event::ValidatorAdded {
                account,
                eth_address,
            });
            
            Ok(())
        }

        /// Remove a bridge validator
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::remove_validator())]
        pub fn remove_validator(
            origin: OriginFor<T>,
            account: T::AccountId,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            // Get validator
            let mut validator = Self::bridge_validators(&account)
                .ok_or(Error::<T>::ValidatorNotFound)?;
            
            // Deactivate validator
            validator.is_active = false;
            BridgeValidators::<T>::insert(&account, validator);
            
            Self::deposit_event(Event::ValidatorRemoved {
                account,
            });
            
            Ok(())
        }

        /// Update bridge configuration
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::update_bridge_config())]
        pub fn update_bridge_config(
            origin: OriginFor<T>,
            new_config: types::BridgeConfig,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            BridgeConfig::<T>::put(&new_config);
            
            Self::deposit_event(Event::BridgeConfigUpdated);
            
            Ok(())
        }

        /// Pause bridge operations
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::pause_bridge())]
        pub fn pause_bridge(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            
            let mut config = Self::bridge_config();
            config.is_paused = true;
            BridgeConfig::<T>::put(&config);
            
            Self::deposit_event(Event::BridgePaused);
            
            Ok(())
        }

        /// Resume bridge operations
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::resume_bridge())]
        pub fn resume_bridge(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            
            let mut config = Self::bridge_config();
            config.is_paused = false;
            BridgeConfig::<T>::put(&config);
            
            Self::deposit_event(Event::BridgeResumed);
            
            Ok(())
        }

        /// Submit a fraud proof
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::submit_fraud_proof())]
        pub fn submit_fraud_proof(
            origin: OriginFor<T>,
            challenged_tx: H256,
            fraud_type: FraudType,
            evidence: Vec<u8>,
        ) -> DispatchResult {
            let challenger = ensure_signed(origin)?;
            
            let current_block = frame_system::Pallet::<T>::block_number();
            let config = Self::bridge_config();
            
            let fraud_proof = FraudProof {
                fraud_type: fraud_type.clone(),
                challenged_tx,
                evidence,
                challenger: challenger.clone(),
                submitted_at: current_block,
                reward: FraudProofValidator::<T>::calculate_fraud_proof_reward(
                    &fraud_type,
                    1_000_000_000_000_000_000, // Default amount for reward calculation
                ),
            };
            
            // Validate fraud proof
            FraudProofValidator::<T>::validate_fraud_proof(
                &fraud_proof,
                config.challenge_period,
                current_block,
            ).map_err(|_| Error::<T>::InvalidSignature)?;
            
            FraudProofs::<T>::insert(&challenged_tx, &fraud_proof);
            
            Self::deposit_event(Event::FraudProofSubmitted {
                challenger,
                challenged_tx,
                fraud_type,
            });
            
            Ok(())
        }
    }

    /// Internal functions
    impl<T: Config> Pallet<T> {
        /// Get the bridge account ID
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }

        /// Mint wrapped tokens for a confirmed deposit
        fn mint_wrapped_tokens(deposit: &BridgeDeposit<T::AccountId>) -> DispatchResult {
            let wrapped_asset_id = deposit.asset.wrapped_asset_id();
            
            // Ensure wrapped asset exists, create if not
            if !T::Assets::asset_exists(wrapped_asset_id) {
                T::Assets::create(
                    wrapped_asset_id,
                    Self::account_id(),
                    true, // is_sufficient
                    1,    // min_balance
                ).map_err(|_| Error::<T>::AssetCreationFailed)?;
            }
            
            // Mint tokens to recipient
            T::Assets::mint_into(
                wrapped_asset_id,
                &deposit.recipient,
                deposit.amount,
            ).map_err(|_| Error::<T>::AssetMintingFailed)?;
            
            Ok(())
        }

        /// Get all active validators
        fn get_active_validators() -> Vec<BridgeValidator<T::AccountId>> {
            BridgeValidators::<T>::iter()
                .filter_map(|(_, validator)| {
                    if validator.is_active {
                        Some(validator)
                    } else {
                        None
                    }
                })
                .collect()
        }

        /// Update validator reputation
        fn update_validator_reputation(validator: &T::AccountId, was_correct: bool) {
            ValidatorReputations::<T>::mutate(validator, |reputation| {
                if let Some(rep) = reputation {
                    rep.update_after_validation(was_correct);
                } else {
                    let mut new_rep = ValidatorReputation::default();
                    new_rep.update_after_validation(was_correct);
                    *reputation = Some(new_rep);
                }
            });
        }

        /// Update bridge statistics
        fn update_bridge_stats(is_deposit: bool, amount: Balance) {
            BridgeStats::<T>::mutate(|stats| {
                if is_deposit {
                    stats.total_deposits = stats.total_deposits.saturating_add(1);
                } else {
                    stats.total_withdrawals = stats.total_withdrawals.saturating_add(1);
                }
                stats.total_volume = stats.total_volume.saturating_add(amount);
            });
        }

        /// Insert a pending withdrawal (for testing)
        #[cfg(test)]
        pub fn insert_pending_withdrawal(
            withdrawal_id: WithdrawalId,
            withdrawal: BridgeWithdrawal<T::AccountId>,
        ) {
            PendingWithdrawals::<T>::insert(withdrawal_id, withdrawal);
        }
    }
}
