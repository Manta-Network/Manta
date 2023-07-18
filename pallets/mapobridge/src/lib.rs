// Copyright 2021-2023 UINB Technologies Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
extern crate core;

use codec::{Decode, Encode};
use frame_support::PalletId;
use fuso_support::chainbridge::EthereumCompatibleAddress;
pub use pallet::*;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

mod alt_bn128;
mod crypto;
mod hash;
mod macros;
mod mapclient;
mod serialization;
mod split;
pub mod traits;
mod types;

const MAPO_MODULE_ID: PalletId = PalletId(*b"mapo/bri");

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Eq, Encode, Decode, TypeInfo)]
pub struct TransferOutEvent {
    pub token_address: EthereumCompatibleAddress,
    pub receipt: EthereumCompatibleAddress,
    pub amount: u128,
    pub to_chain_id: u16,
}
#[frame_support::pallet]
pub mod pallet {
    use crate::{
        crypto::G2,
        mapclient::MapLightClient,
        traits::FromRlp,
        types::{
            common::{Address, ADDRESS_LENGTH},
            event::MapTransferOutEvent,
            header::Header,
            istanbul::{get_epoch_number, IstanbulExtra},
            proof::ReceiptProof,
        },
        TransferOutEvent, MAPO_MODULE_ID,
    };
    // use beefy_primitives::mmr::BeefyDataProvider;
    use frame_support::{log::info, pallet_prelude::*, traits::fungibles::Mutate, transactional};
    use frame_system::pallet_prelude::*;
    use fuso_support::{
        chainbridge::{AssetIdResourceIdProvider, EthereumCompatibleAddress},
        traits::{DecimalsTransformer, Token},
        ChainId,
    };
    use num_traits::ToPrimitive;
    use sp_core::crypto::AccountId32;
    use sp_runtime::{
        traits::{AccountIdConversion, IdentifyAccount, Verify},
        MultiSignature,
    };
    use sp_std::vec::Vec;
    pub type Signature = MultiSignature;

    pub(crate) type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

    type AssetId<T> =
        <<T as Config>::Fungibles as Token<<T as frame_system::Config>::AccountId>>::TokenId;

    type BalanceOf<T> =
        <<T as Config>::Fungibles as Token<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Expose customizable associated type of asset transfer, lock and unlock
        type Fungibles: Mutate<Self::AccountId, AssetId = AssetId<Self>, Balance = BalanceOf<Self>>
            + Token<Self::AccountId>
            + DecimalsTransformer<BalanceOf<Self>>;

        /// Map of cross-chain asset ID & name
        type AssetIdByName: AssetIdResourceIdProvider<AssetId<Self>>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        //mapTokenAddress, receipt, Amount, to_chain_id
        TransferOut(
            EthereumCompatibleAddress,
            EthereumCompatibleAddress,
            BalanceOf<T>,
            u128,
        ),
    }

    #[pallet::error]
    pub enum Error<T> {
        HeaderError,
        HeaderVerifyFailed,
        EpochRecordNotFound,
        ProofError,
        TokenError,
        EventError,
    }

    #[pallet::storage]
    #[pallet::getter(fn map_client)]
    pub type MapClientInfo<T: Config> = StorageValue<_, MapLightClient, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn bridge_address)]
    pub type MapBridgeAddress<T: Config> = StorageValue<_, Address, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn transfer_out_events)]
    pub type TransferOutEvents<T: Config> = StorageMap<_, Blake2_256, T::BlockNumber, Vec<Vec<u8>>>;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        <T as frame_system::Config>::AccountId: From<AccountId32>,
        BalanceOf<T>: From<u128>,
    {
        #[pallet::weight(8_790_000_000)]
        pub fn update_block_header(
            origin: OriginFor<T>,
            hearder_bytes: Vec<u8>,
            agg_pk: G2,
        ) -> DispatchResultWithPostInfo {
            let caller = ensure_signed(origin)?;

            let header =
                Header::from_rlp(hearder_bytes.as_ref()).map_err(|_| Error::<T>::HeaderError)?;
            let block_num = header.number.to_u64().ok_or(Error::<T>::HeaderError)?;
            let mut map_client: MapLightClient = Self::map_client();
            let block_exp = map_client.header_height + map_client.epoch_size;
            ensure!(block_exp == block_num, Error::<T>::HeaderError);

            // check ecdsa and bls signature
            let epoch = get_epoch_number(block_num, map_client.epoch_size as u64);
            let mut extra = IstanbulExtra::from_rlp(&header.extra).unwrap();

            let cur_epoch_record = map_client
                .epoch_records
                .get(&epoch)
                .ok_or(Error::<T>::EpochRecordNotFound)?
                .clone();
            map_client
                .verify_signatures(&header, agg_pk, &extra, &cur_epoch_record)
                .map_err(|_| Error::<T>::HeaderVerifyFailed)?;

            // update validators' pair keys
            map_client.update_next_validators(&cur_epoch_record, &mut extra);

            map_client.header_height = block_num;
            <MapClientInfo<T>>::put(map_client);
            info!(
                "block header {} is updated for the next epoch {} by {:?}",
                block_num,
                epoch + 1,
                caller
            );
            Ok(().into())
        }

        #[pallet::weight(8_790_000_000)]
        pub fn transfer_in(
            origin: OriginFor<T>,
            receipt_proof: ReceiptProof,
            index: u64,
        ) -> DispatchResultWithPostInfo
        where
            <T as frame_system::Config>::AccountId: From<AccountId32>,
            BalanceOf<T>: From<u128>,
        {
            let caller = ensure_signed(origin)?;
            ensure!(
                index < receipt_proof.receipt.logs.len() as u64,
                Error::<T>::ProofError
            );
            let event = MapTransferOutEvent::from_log_entry_data(
                receipt_proof.receipt.logs.get(index as usize).unwrap(),
            )
            .ok_or(Error::<T>::ProofError)?;
            info!("get transfer in event: {:?}", event);
            ensure!(
                Self::bridge_address() == event.map_bridge_address,
                Error::<T>::ProofError
            );
            let to_len = event.to.len();
            ensure!(to_len == 32usize, Error::<T>::EventError);
            let mut slice = [0u8; 32];
            slice.copy_from_slice(event.to.as_slice());
            let to_address: AccountId = AccountId::new(slice);
            let token_contract = event.to_chain_token.clone();
            ensure!(
                token_contract.len() == ADDRESS_LENGTH,
                Error::<T>::TokenError
            );

            ensure!(event.to_chain < 65535, Error::<T>::EventError);
            let chain_id = event.to_chain as u16;
            let amount = event.amount;

            let mclient: MapLightClient = Self::map_client();
            mclient
                .verify_proof_data(receipt_proof)
                .map_err(|_| Error::<T>::ProofError)?;
            Self::process_transfer_in(token_contract, amount.into(), to_address.into(), chain_id)?;
            /*
                   self.check_not_paused(PAUSE_TRANSFER_IN);
            assert!(env::is_valid_account_id(event.to.as_slice()), "invalid to address: {:?}", event.to);
            assert!(env::is_valid_account_id(event.to_chain_token.as_slice()),
                    "invalid to chain token address: {:?}", event.to_chain_token);
            let to_chain_token = String::from_utf8(event.to_chain_token.clone()).unwrap();
            assert_eq!(self.near_chain_id, event.to_chain.0, "unexpected to chain: {}", event.to_chain.0);
            assert!(self.mcs_tokens.get(&to_chain_token).is_some()
                        || self.fungible_tokens.get(&to_chain_token).is_some() || self.is_native_token(event.to_chain_token.clone()),
                    "to_chain_token {} is not mcs token or fungible token or native token", to_chain_token);
            assert_eq!(false, self.is_used_event(&event.order_id), "the event with order id {} is used", hex::encode(event.order_id));

            ext_map_light_client::ext(self.map_client_account.clone())
                .with_static_gas(VERIFY_LOG_ENTRY_GAS)
                .verify_proof_data(receipt_proof)
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(TRANSFER_IN_SINGLE_EVENT_GAS + FINISH_TRANSFER_IN_GAS)
                        .with_attached_deposit(env::attached_deposit())
                        .finish_verify_proof(&event)
                )
                */
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        #[transactional]
        fn process_transfer_in(
            token_contract: Vec<u8>,
            amount: BalanceOf<T>,
            to_address: T::AccountId,
            chain_id: u16,
        ) -> DispatchResultWithPostInfo {
            let mapo_account: T::AccountId = MAPO_MODULE_ID.try_into_account().unwrap();
            if Self::is_native(&token_contract) {
                Self::do_unlock(mapo_account, to_address.clone(), amount)?;
            } else {
                Self::do_mint_assets(to_address.clone(), amount, token_contract, chain_id)?;
            }
            Ok(().into())
        }

        fn is_native(token_contract: &Vec<u8>) -> bool {
            //TODO
            true
        }

        pub(crate) fn do_unlock(
            sender: T::AccountId,
            to: T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let native_token_id = T::Fungibles::native_token_id();
            T::Fungibles::transfer_token(&sender, native_token_id, amount, &to)?;
            Ok(())
        }

        pub(crate) fn do_mint_assets(
            who: T::AccountId,
            amount: BalanceOf<T>,
            contract_address: Vec<u8>,
            chain_id: ChainId,
        ) -> DispatchResult {
            let token_id = T::AssetIdByName::try_get_asset_id(chain_id, contract_address)
                .map_err(|_| Error::<T>::TokenError)?;
            T::Fungibles::mint_into(token_id, &who, amount)?;
            Ok(())
        }
    }

    // impl<T: Config> BeefyDataProvider<Vec<Vec<u8>>> for Pallet<T> {
    //     fn extra_data() -> Vec<Vec<u8>> {
    //         let block_number = frame_system::Pallet::<T>::block_number();
    //         let extra: Option<Vec<Vec<u8>>> = Self::transfer_out_events(block_number);
    //         extra.unwrap_or_default()
    //     }
    // }
}
