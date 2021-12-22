// Copyright 2020-2021 Manta Network.
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

use super::*;
use sp_runtime::{traits::AccountIdConversion, AccountId32};
use xcm_simulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};

pub mod parachain;
pub mod relaychain;

pub type AccountId = AccountId32;
pub type Balance = u128;

pub const ALICE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([1u8; 32]);
pub const BOB: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([2u8; 32]);
pub const INITIAL_BALANCE: u128 = 1_000_000_000;

decl_test_network! {
	pub struct TestNet {
		relay_chain = Relay,
		parachains = vec![
			(2015, MantaPara),
			(2084, CalamariPara),
		],
	}
}

decl_test_parachain! {
	pub struct MantaPara {
		Runtime = parachain::Runtime,
		XcmpMessageHandler = parachain::XcmpQueue,
		DmpMessageHandler = parachain::DmpQueue,
		new_ext = para_ext(2015),
	}
}

decl_test_parachain! {
	pub struct CalamariPara {
		Runtime = parachain::Runtime,
		XcmpMessageHandler = parachain::XcmpQueue,
		DmpMessageHandler = parachain::DmpQueue,
		new_ext = para_ext(2084),
	}
}

decl_test_relay_chain! {
	pub struct Relay {
		Runtime = relaychain::Runtime,
		XcmConfig = relaychain::XcmConfig,
		new_ext = relay_ext(),
	}
}

pub fn para_account_id(id: u32) -> AccountId {
	ParaId::from(id).into_account()
}

pub fn para_ext(para_id: u32) -> sp_io::TestExternalities {
	use parachain::{Runtime, System};

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
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn relay_ext() -> sp_io::TestExternalities {
	use relaychain::{Runtime, System};

	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![
			(ALICE, INITIAL_BALANCE),
			(para_account_id(2015), INITIAL_BALANCE),
			(para_account_id(2084), INITIAL_BALANCE),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
