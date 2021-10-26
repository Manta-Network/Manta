use frame_support::parameter_types;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use sp_std::time::Duration;

use super::*;
use crate as manta_vesting;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub type AccountId = u128;
pub type BlockNumber = u64;
pub type Balance = u128;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		MantaVesting: manta_vesting::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
}
impl frame_system::Config for Test {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockHashCount = BlockHashCount;
	type BlockLength = ();
	type BlockNumber = BlockNumber;
	type BlockWeights = ();
	type Call = Call;
	type DbWeight = ();
	type Event = Event;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Header = Header;
	type Index = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type Origin = Origin;
	type PalletInfo = PalletInfo;
	type SS58Prefix = ();
	type SystemWeightInfo = ();
	type Version = ();
}

parameter_types! {
	pub const MaxLocks: u32 = 10;
}
impl pallet_balances::Config for Test {
	type AccountStore = System;
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = 12_000 / 2;
}
impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const MinVestedTransfer: Balance = 1;
	pub static ExistentialDeposit: Balance = 1;
	pub VestingSchedule: [(Percent, Duration); 7] = [
		(Percent::from_percent(34), Duration::from_secs(1635120000)),
		(Percent::from_percent(11), Duration::from_secs(1636502400)),
		(Percent::from_percent(11), Duration::from_secs(1641340800)),
		(Percent::from_percent(11), Duration::from_secs(1646179200)),
		(Percent::from_percent(11), Duration::from_secs(1651017600)),
		(Percent::from_percent(11), Duration::from_secs(1655856000)),
		(Percent::from_percent(11), Duration::from_secs(1660694400)),
	];
}
impl Config for Test {
	type Currency = Balances;
	type Event = Event;
	type Timestamp = Timestamp;
	type MinVestedTransfer = MinVestedTransfer;
	type VestingSchedule = VestingSchedule;
}

pub struct ExtBuilder {
	existential_deposit: Balance,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			existential_deposit: 1,
		}
	}
}

impl ExtBuilder {
	pub fn existential_deposit(mut self, existential_deposit: Balance) -> Self {
		self.existential_deposit = existential_deposit;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		pallet_balances::GenesisConfig::<Test> {
			balances: vec![(ALICE, 10_000 * self.existential_deposit)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| {
			// 1635120000 - 3 * 6000
			Timestamp::set_timestamp(1635102000000);
			System::set_block_number(1);
		});
		ext
	}
}

pub(crate) fn run_to_block(n: u64) {
	System::set_block_number(n);
}
