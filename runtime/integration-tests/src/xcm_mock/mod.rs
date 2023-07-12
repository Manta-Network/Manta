// Copyright 2020-2023 Manta Network.
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

#![cfg(test)]

pub mod parachain;
pub mod relay_chain;
pub mod xcm_tests;

use frame_support::traits::GenesisBuild;
use polkadot_parachain::primitives::Id as ParaId;
use sp_runtime::traits::AccountIdConversion;
use xcm_simulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};
pub const ALICE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([0u8; 32]);
pub const INITIAL_BALANCE: u128 = 10_000_000_000_000_000;
pub const PARA_A_ID: u32 = 1;
pub const PARA_B_ID: u32 = 2;
pub const PARA_C_ID: u32 = 3;

decl_test_parachain! {
    pub struct ParaA {
        Runtime = parachain::Runtime,
        XcmpMessageHandler = parachain::XcmpQueue,
        DmpMessageHandler = parachain::MsgQueue,
        new_ext = para_ext(PARA_A_ID),
    }
}

decl_test_parachain! {
    pub struct ParaB {
        Runtime = parachain::Runtime,
        XcmpMessageHandler = parachain::XcmpQueue,
        DmpMessageHandler = parachain::MsgQueue,
        new_ext = para_ext(PARA_B_ID),
    }
}

decl_test_parachain! {
    pub struct ParaC {
        Runtime = parachain::Runtime,
        XcmpMessageHandler = parachain::XcmpQueue,
        DmpMessageHandler = parachain::MsgQueue,
        new_ext = para_ext(PARA_C_ID),
    }
}

decl_test_relay_chain! {
    pub struct Relay {
        Runtime = relay_chain::Runtime,
        XcmConfig = relay_chain::XcmExecutorConfig,
        new_ext = relay_ext(),
    }
}

decl_test_network! {
    pub struct MockNet {
        relay_chain = Relay,
        parachains = vec![
            (1, ParaA),
            (2, ParaB),
            (3, ParaC),
        ],
    }
}

pub fn para_account_id(id: u32) -> relay_chain::AccountId {
    ParaId::from(id).into_account_truncating()
}

pub fn para_ext(para_id: u32) -> sp_io::TestExternalities {
    use parachain::{MsgQueue, Runtime, System};

    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap();

    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![(ALICE, INITIAL_BALANCE)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let parachain_info_config = parachain_info::GenesisConfig {
        parachain_id: para_id.into(),
    };
    <parachain_info::GenesisConfig as GenesisBuild<Runtime, _>>::assimilate_storage(
        &parachain_info_config,
        &mut t,
    )
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        MsgQueue::set_para_id(para_id.into());
    });
    ext
}

pub fn relay_ext() -> sp_io::TestExternalities {
    use relay_chain::{Runtime, System};

    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap();

    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![
            (ALICE, INITIAL_BALANCE),
            (para_account_id(1), INITIAL_BALANCE),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub type RelayChainPalletXcm = pallet_xcm::Pallet<relay_chain::Runtime>;
pub type RelayBalances = pallet_balances::Pallet<relay_chain::Runtime>;
pub type ParachainPalletXcm = pallet_xcm::Pallet<parachain::Runtime>;
