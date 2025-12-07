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

//! # Chameleon Bridge Pallet - PRODUCTION GRADE
//!
//! Cross-chain bridge for ETH, USDC, USDT, WBTC between Ethereum and Chameleon.
//!
//! ## Features
//! - Lock & Mint: Lock on Ethereum, mint wrapped tokens on Chameleon
//! - Burn & Unlock: Burn on Chameleon, unlock on Ethereum
//! - Multi-sig validation (5-of-9 threshold)
//! - ACTUAL TOKEN MINTING/BURNING via pallet-assets

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        traits::tokens::fungibles::{Inspect, Mutate},
        weights::Weight,
    };
    use frame_system::pallet_prelude::*;
    use sp_core::{H160, H256};
    use sp_runtime::{traits::{Zero, Saturating}, RuntimeDebug};
    use codec::{Encode, Decode};
    use scale_info::TypeInfo;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    /// Wrapped asset IDs on Chameleon
    /// These are the asset IDs in pallet-assets for wrapped tokens
    pub const WRAPPED_ETH_ASSET_ID: u128 = 1;
    pub const WRAPPED_USDC_ASSET_ID: u128 = 2;
    pub const WRAPPED_USDT_ASSET_ID: u128 = 3;
    pub const WRAPPED_WBTC_ASSET_ID: u128 = 4;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Asset ID type
        type AssetId: Parameter + Copy + From<u128> + MaxEncodedLen;

        /// Balance type
        type Balance: Parameter + Copy + Zero + Saturating + From<u128> + Into<u128> + MaxEncodedLen + Default;

        /// Asset operations for minting/burning wrapped tokens
        type Assets: Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = Self::Balance> +
                     Mutate<Self::AccountId>;

        /// Minimum confirmations required
        #[pallet::constant]
        type MinConfirmations: Get<u32>;

        /// Validator signature threshold (5-of-9)
        #[pallet::constant]
        type SignatureThreshold: Get<u32>;
    }

    /// Bridgeable assets
    #[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, MaxEncodedLen, Copy)]
    pub enum BridgeableAsset {
        ETH,
        USDC,
        USDT,
        WBTC,
    }

    impl Default for BridgeableAsset {
        fn default() -> Self { BridgeableAsset::ETH }
    }

    impl BridgeableAsset {
        /// Get the wrapped asset ID for this bridgeable asset
        pub fn wrapped_asset_id<T: Config>(&self) -> T::AssetId {
            match self {
                BridgeableAsset::ETH => WRAPPED_ETH_ASSET_ID.into(),
                BridgeableAsset::USDC => WRAPPED_USDC_ASSET_ID.into(),
                BridgeableAsset::USDT => WRAPPED_USDT_ASSET_ID.into(),
                BridgeableAsset::WBTC => WRAPPED_WBTC_ASSET_ID.into(),
            }
        }
    }

    /// Deposit status
    #[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, MaxEncodedLen, Default)]
    pub enum DepositStatus {
        #[default]
        Pending,
        Confirmed,
        Minted,
        Failed,
    }

    /// Withdrawal status
    #[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, MaxEncodedLen, Default)]
    pub enum WithdrawalStatus {
        #[default]
        Pending,
        ReadyToExecute,
        Completed,
        Failed,
    }

    /// Bridge deposit record
    #[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct BridgeDeposit<T: Config> {
        pub eth_tx_hash: H256,
        pub recipient: T::AccountId,
        pub asset: BridgeableAsset,
        pub amount: T::Balance,
        pub status: DepositStatus,
        pub approvals: u32,
    }

    /// Bridge withdrawal record
    #[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct BridgeWithdrawal<T: Config> {
        pub withdrawal_id: u64,
        pub from: T::AccountId,
        pub eth_destination: H160,
        pub asset: BridgeableAsset,
        pub amount: T::Balance,
        pub status: WithdrawalStatus,
        pub signature_count: u32,
    }

    /// Pending deposits awaiting validator approval
    #[pallet::storage]
    #[pallet::getter(fn pending_deposits)]
    pub type PendingDeposits<T: Config> = StorageMap<
        _, Blake2_128Concat, H256, BridgeDeposit<T>, OptionQuery
    >;

    /// Processed deposits (to prevent replay)
    #[pallet::storage]
    pub type ProcessedDeposits<T: Config> = StorageMap<
        _, Blake2_128Concat, H256, bool, ValueQuery
    >;

    /// Validator approvals per deposit
    #[pallet::storage]
    pub type DepositApprovals<T: Config> = StorageDoubleMap<
        _, Blake2_128Concat, H256, Blake2_128Concat, T::AccountId, bool, ValueQuery
    >;

    /// Pending withdrawals
    #[pallet::storage]
    #[pallet::getter(fn pending_withdrawals)]
    pub type PendingWithdrawals<T: Config> = StorageMap<
        _, Blake2_128Concat, u64, BridgeWithdrawal<T>, OptionQuery
    >;

    /// Withdrawal signatures
    #[pallet::storage]
    pub type WithdrawalSignatures<T: Config> = StorageDoubleMap<
        _, Blake2_128Concat, u64, Blake2_128Concat, T::AccountId, bool, ValueQuery
    >;

    /// Registered bridge validators
    #[pallet::storage]
    #[pallet::getter(fn validators)]
    pub type BridgeValidators<T: Config> = StorageMap<
        _, Blake2_128Concat, T::AccountId, bool, ValueQuery
    >;

    /// Next withdrawal ID
    #[pallet::storage]
    pub type NextWithdrawalId<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Total bridged per asset
    #[pallet::storage]
    #[pallet::getter(fn total_bridged)]
    pub type TotalBridged<T: Config> = StorageMap<
        _, Blake2_128Concat, BridgeableAsset, T::Balance, ValueQuery
    >;

    /// Bridge paused
    #[pallet::storage]
    #[pallet::getter(fn is_paused)]
    pub type IsPaused<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Deposit reported by validator
        DepositReported { eth_tx_hash: H256, validator: T::AccountId, asset: BridgeableAsset, amount: T::Balance },
        /// Deposit approved and tokens minted
        DepositMinted { eth_tx_hash: H256, recipient: T::AccountId, asset: BridgeableAsset, amount: T::Balance },
        /// Withdrawal initiated, tokens burned
        WithdrawalInitiated { withdrawal_id: u64, from: T::AccountId, eth_destination: H160, asset: BridgeableAsset, amount: T::Balance },
        /// Withdrawal signed by validator
        WithdrawalSigned { withdrawal_id: u64, validator: T::AccountId, signature_count: u32 },
        /// Withdrawal ready for Ethereum execution
        WithdrawalReady { withdrawal_id: u64 },
        /// Validator added
        ValidatorAdded { validator: T::AccountId },
        /// Validator removed
        ValidatorRemoved { validator: T::AccountId },
        /// Bridge paused
        BridgePaused,
        /// Bridge resumed
        BridgeResumed,
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Bridge is paused
        BridgePaused,
        /// Not a registered validator
        NotValidator,
        /// Deposit already processed
        DepositAlreadyProcessed,
        /// Deposit not found
        DepositNotFound,
        /// Withdrawal not found
        WithdrawalNotFound,
        /// Already approved this deposit
        AlreadyApproved,
        /// Already signed this withdrawal
        AlreadySigned,
        /// Insufficient balance to burn
        InsufficientBalance,
        /// Token operation failed
        TokenOperationFailed,
        /// Invalid amount
        InvalidAmount,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Validator reports seeing a lock event on Ethereum
        /// When threshold approvals reached, mints wrapped tokens
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn report_deposit(
            origin: OriginFor<T>,
            eth_tx_hash: H256,
            recipient: T::AccountId,
            asset: BridgeableAsset,
            amount: T::Balance,
        ) -> DispatchResult {
            let validator = ensure_signed(origin)?;
            
            // Must be registered validator
            ensure!(BridgeValidators::<T>::get(&validator), Error::<T>::NotValidator);
            ensure!(!IsPaused::<T>::get(), Error::<T>::BridgePaused);
            ensure!(!amount.is_zero(), Error::<T>::InvalidAmount);
            
            // Cannot process same deposit twice
            ensure!(!ProcessedDeposits::<T>::get(eth_tx_hash), Error::<T>::DepositAlreadyProcessed);
            
            // Check if already approved by this validator
            ensure!(!DepositApprovals::<T>::get(eth_tx_hash, &validator), Error::<T>::AlreadyApproved);
            
            // Record approval
            DepositApprovals::<T>::insert(eth_tx_hash, &validator, true);
            
            // Get or create deposit record
            let mut deposit = PendingDeposits::<T>::get(eth_tx_hash).unwrap_or(BridgeDeposit {
                eth_tx_hash,
                recipient: recipient.clone(),
                asset,
                amount,
                status: DepositStatus::Pending,
                approvals: 0,
            });
            
            deposit.approvals += 1;
            
            Self::deposit_event(Event::DepositReported {
                eth_tx_hash,
                validator: validator.clone(),
                asset,
                amount,
            });
            
            // Check if threshold reached
            if deposit.approvals >= T::SignatureThreshold::get() {
                // ========== MINT WRAPPED TOKENS ==========
                let wrapped_asset_id = asset.wrapped_asset_id::<T>();
                
                T::Assets::mint_into(
                    wrapped_asset_id,
                    &deposit.recipient,
                    amount,
                ).map_err(|_| Error::<T>::TokenOperationFailed)?;
                // ==========================================
                
                deposit.status = DepositStatus::Minted;
                ProcessedDeposits::<T>::insert(eth_tx_hash, true);
                TotalBridged::<T>::mutate(asset, |total| *total = total.saturating_add(amount));
                
                Self::deposit_event(Event::DepositMinted {
                    eth_tx_hash,
                    recipient: deposit.recipient.clone(),
                    asset,
                    amount,
                });
            }
            
            PendingDeposits::<T>::insert(eth_tx_hash, deposit);
            
            Ok(())
        }

        /// Initiate withdrawal - burns wrapped tokens immediately
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn initiate_withdrawal(
            origin: OriginFor<T>,
            asset: BridgeableAsset,
            amount: T::Balance,
            eth_destination: H160,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            ensure!(!IsPaused::<T>::get(), Error::<T>::BridgePaused);
            ensure!(!amount.is_zero(), Error::<T>::InvalidAmount);
            
            let wrapped_asset_id = asset.wrapped_asset_id::<T>();
            
            // ========== BURN WRAPPED TOKENS ==========
            T::Assets::burn_from(
                wrapped_asset_id,
                &who,
                amount,
                frame_support::traits::tokens::Precision::Exact,
                frame_support::traits::tokens::Fortitude::Polite,
            ).map_err(|_| Error::<T>::TokenOperationFailed)?;
            // ==========================================
            
            let withdrawal_id = NextWithdrawalId::<T>::get();
            NextWithdrawalId::<T>::put(withdrawal_id + 1);
            
            let withdrawal = BridgeWithdrawal {
                withdrawal_id,
                from: who.clone(),
                eth_destination,
                asset,
                amount,
                status: WithdrawalStatus::Pending,
                signature_count: 0,
            };
            
            PendingWithdrawals::<T>::insert(withdrawal_id, withdrawal);
            TotalBridged::<T>::mutate(asset, |total| *total = total.saturating_sub(amount));
            
            Self::deposit_event(Event::WithdrawalInitiated {
                withdrawal_id,
                from: who,
                eth_destination,
                asset,
                amount,
            });
            
            Ok(())
        }

        /// Validator signs withdrawal for Ethereum execution
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn sign_withdrawal(
            origin: OriginFor<T>,
            withdrawal_id: u64,
        ) -> DispatchResult {
            let validator = ensure_signed(origin)?;
            
            ensure!(BridgeValidators::<T>::get(&validator), Error::<T>::NotValidator);
            ensure!(!WithdrawalSignatures::<T>::get(withdrawal_id, &validator), Error::<T>::AlreadySigned);
            
            PendingWithdrawals::<T>::try_mutate(withdrawal_id, |maybe_withdrawal| {
                let withdrawal = maybe_withdrawal.as_mut().ok_or(Error::<T>::WithdrawalNotFound)?;
                
                WithdrawalSignatures::<T>::insert(withdrawal_id, &validator, true);
                withdrawal.signature_count += 1;
                
                Self::deposit_event(Event::WithdrawalSigned {
                    withdrawal_id,
                    validator: validator.clone(),
                    signature_count: withdrawal.signature_count,
                });
                
                if withdrawal.signature_count >= T::SignatureThreshold::get() {
                    withdrawal.status = WithdrawalStatus::ReadyToExecute;
                    Self::deposit_event(Event::WithdrawalReady { withdrawal_id });
                }
                
                Ok(())
            })
        }

        /// Add a bridge validator (root only)
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn add_validator(origin: OriginFor<T>, validator: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;
            BridgeValidators::<T>::insert(&validator, true);
            Self::deposit_event(Event::ValidatorAdded { validator });
            Ok(())
        }

        /// Remove a bridge validator (root only)
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn remove_validator(origin: OriginFor<T>, validator: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;
            BridgeValidators::<T>::remove(&validator);
            Self::deposit_event(Event::ValidatorRemoved { validator });
            Ok(())
        }

        /// Pause bridge (root only)
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn pause_bridge(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            IsPaused::<T>::put(true);
            Self::deposit_event(Event::BridgePaused);
            Ok(())
        }

        /// Resume bridge (root only)
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn resume_bridge(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            IsPaused::<T>::put(false);
            Self::deposit_event(Event::BridgeResumed);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests;
