mod common;
use common::*;

use calamari_runtime::{
	currency::KMA, Authorship, Balances, Origin, PolkadotXcm, Runtime, Treasury,
};

use frame_support::{
	assert_ok,
	codec::Encode,
	dispatch::Dispatchable,
	traits::{PalletInfo, StorageInfo, StorageInfoTrait},
	StorageHasher, Twox128,
};
use manta_primitives::{
	helpers::{get_account_id_from_seed, get_collator_keys_from_seed},
	AccountId, Header,
};

use pallet_transaction_payment::ChargeTransactionPayment;

use sp_consensus_aura::AURA_ENGINE_ID;
use sp_core::sr25519;
use sp_runtime::{
	generic::DigestItem,
	traits::{Header as HeaderT, SignedExtension},
	Percent,
};

#[test]
fn fast_track_available() {
	assert!(<calamari_runtime::Runtime as pallet_democracy::Config>::InstantAllowed::get());
}

#[test]
fn verify_pallet_prefixes() {
	fn is_pallet_prefix<P: 'static>(name: &str) {
		// Compares the unhashed pallet prefix in the `StorageInstance` implementation by every
		// storage item in the pallet P. This pallet prefix is used in conjunction with the
		// item name to get the unique storage key: hash(PalletPrefix) + hash(StorageName)
		// https://github.com/paritytech/substrate/blob/master/frame/support/procedural/src/pallet/
		// expand/storage.rs#L389-L401
		assert_eq!(
			<calamari_runtime::Runtime as frame_system::Config>::PalletInfo::name::<P>(),
			Some(name)
		);
	}
	// TODO: use StorageInfoTrait from https://github.com/paritytech/substrate/pull/9246
	// This is now available with polkadot-v0.9.9 dependencies
	is_pallet_prefix::<calamari_runtime::System>("System");
	is_pallet_prefix::<calamari_runtime::ParachainSystem>("ParachainSystem");
	is_pallet_prefix::<calamari_runtime::Timestamp>("Timestamp");
	is_pallet_prefix::<calamari_runtime::ParachainInfo>("ParachainInfo");
	is_pallet_prefix::<calamari_runtime::TransactionPause>("TransactionPause");
	is_pallet_prefix::<calamari_runtime::Balances>("Balances");
	is_pallet_prefix::<calamari_runtime::TransactionPayment>("TransactionPayment");
	is_pallet_prefix::<calamari_runtime::Democracy>("Democracy");
	is_pallet_prefix::<calamari_runtime::Council>("Council");
	is_pallet_prefix::<calamari_runtime::CouncilMembership>("CouncilMembership");
	is_pallet_prefix::<calamari_runtime::TechnicalCommittee>("TechnicalCommittee");
	is_pallet_prefix::<calamari_runtime::TechnicalMembership>("TechnicalMembership");
	is_pallet_prefix::<calamari_runtime::Authorship>("Authorship");
	is_pallet_prefix::<calamari_runtime::CollatorSelection>("CollatorSelection");
	is_pallet_prefix::<calamari_runtime::Session>("Session");
	is_pallet_prefix::<calamari_runtime::Aura>("Aura");
	is_pallet_prefix::<calamari_runtime::AuraExt>("AuraExt");
	is_pallet_prefix::<calamari_runtime::Treasury>("Treasury");
	is_pallet_prefix::<calamari_runtime::Scheduler>("Scheduler");
	is_pallet_prefix::<calamari_runtime::XcmpQueue>("XcmpQueue");
	is_pallet_prefix::<calamari_runtime::PolkadotXcm>("PolkadotXcm");
	is_pallet_prefix::<calamari_runtime::CumulusXcm>("CumulusXcm");
	is_pallet_prefix::<calamari_runtime::DmpQueue>("DmpQueue");
	is_pallet_prefix::<calamari_runtime::Utility>("Utility");
	is_pallet_prefix::<calamari_runtime::Multisig>("Multisig");
	is_pallet_prefix::<calamari_runtime::Sudo>("Sudo");
	is_pallet_prefix::<calamari_runtime::CalamariVesting>("CalamariVesting");

	let prefix = |pallet_name, storage_name| {
		let mut res = [0u8; 32];
		res[0..16].copy_from_slice(&Twox128::hash(pallet_name));
		res[16..32].copy_from_slice(&Twox128::hash(storage_name));
		res.to_vec()
	};
	assert_eq!(
		<calamari_runtime::Timestamp as StorageInfoTrait>::storage_info(),
		vec![
			StorageInfo {
				pallet_name: b"Timestamp".to_vec(),
				storage_name: b"Now".to_vec(),
				prefix: prefix(b"Timestamp", b"Now"),
				max_values: Some(1),
				max_size: Some(8),
			},
			StorageInfo {
				pallet_name: b"Timestamp".to_vec(),
				storage_name: b"DidUpdate".to_vec(),
				prefix: prefix(b"Timestamp", b"DidUpdate"),
				max_values: Some(1),
				max_size: Some(1),
			}
		]
	);
	assert_eq!(
		<calamari_runtime::Balances as StorageInfoTrait>::storage_info(),
		vec![
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"TotalIssuance".to_vec(),
				prefix: prefix(b"Balances", b"TotalIssuance"),
				max_values: Some(1),
				max_size: Some(16),
			},
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"Account".to_vec(),
				prefix: prefix(b"Balances", b"Account"),
				max_values: Some(300_000),
				max_size: Some(112),
			},
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"Locks".to_vec(),
				prefix: prefix(b"Balances", b"Locks"),
				max_values: Some(300_000),
				max_size: Some(1299),
			},
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"Reserves".to_vec(),
				prefix: prefix(b"Balances", b"Reserves"),
				max_values: None,
				max_size: Some(1249),
			},
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"StorageVersion".to_vec(),
				prefix: prefix(b"Balances", b"StorageVersion"),
				max_values: Some(1),
				max_size: Some(1),
			}
		]
	);
	assert_eq!(
		<calamari_runtime::Sudo as StorageInfoTrait>::storage_info(),
		vec![StorageInfo {
			pallet_name: b"Sudo".to_vec(),
			storage_name: b"Key".to_vec(),
			prefix: prefix(b"Sudo", b"Key"),
			max_values: Some(1),
			max_size: Some(32),
		}]
	);
}

#[test]
fn test_collectives_storage_item_prefixes() {
	for StorageInfo { pallet_name, .. } in
		<calamari_runtime::CouncilMembership as StorageInfoTrait>::storage_info()
	{
		assert_eq!(pallet_name, b"CouncilMembership".to_vec());
	}

	for StorageInfo { pallet_name, .. } in
		<calamari_runtime::TechnicalMembership as StorageInfoTrait>::storage_info()
	{
		assert_eq!(pallet_name, b"TechnicalMembership".to_vec());
	}
}

#[test]
fn verify_pallet_indices() {
	fn is_pallet_index<P: 'static>(index: usize) {
		assert_eq!(
			<calamari_runtime::Runtime as frame_system::Config>::PalletInfo::index::<P>(),
			Some(index)
		);
	}

	is_pallet_index::<calamari_runtime::System>(0);
	is_pallet_index::<calamari_runtime::ParachainSystem>(1);
	is_pallet_index::<calamari_runtime::Timestamp>(2);
	is_pallet_index::<calamari_runtime::ParachainInfo>(3);
	is_pallet_index::<calamari_runtime::TransactionPause>(9);
	is_pallet_index::<calamari_runtime::Balances>(10);
	is_pallet_index::<calamari_runtime::TransactionPayment>(11);
	is_pallet_index::<calamari_runtime::Democracy>(14);
	is_pallet_index::<calamari_runtime::Council>(15);
	is_pallet_index::<calamari_runtime::CouncilMembership>(16);
	is_pallet_index::<calamari_runtime::TechnicalCommittee>(17);
	is_pallet_index::<calamari_runtime::TechnicalMembership>(18);
	is_pallet_index::<calamari_runtime::Authorship>(20);
	is_pallet_index::<calamari_runtime::CollatorSelection>(21);
	is_pallet_index::<calamari_runtime::Session>(22);
	is_pallet_index::<calamari_runtime::Aura>(23);
	is_pallet_index::<calamari_runtime::AuraExt>(24);
	is_pallet_index::<calamari_runtime::Treasury>(26);
	is_pallet_index::<calamari_runtime::Scheduler>(29);
	is_pallet_index::<calamari_runtime::XcmpQueue>(30);
	is_pallet_index::<calamari_runtime::PolkadotXcm>(31);
	is_pallet_index::<calamari_runtime::CumulusXcm>(32);
	is_pallet_index::<calamari_runtime::DmpQueue>(33);
	is_pallet_index::<calamari_runtime::Utility>(40);
	is_pallet_index::<calamari_runtime::Multisig>(41);
	is_pallet_index::<calamari_runtime::Sudo>(42);
	is_pallet_index::<calamari_runtime::CalamariVesting>(50);
}

fn seal_header(mut header: Header, author: AccountId) -> Header {
	{
		let digest = header.digest_mut();
		digest
			.logs
			.push(DigestItem::PreRuntime(AURA_ENGINE_ID, author.encode()));
		digest
			.logs
			.push(DigestItem::Seal(AURA_ENGINE_ID, author.encode()));
	}

	header
}

#[test]
fn reward_fees_to_block_author_and_treasury() {
	let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
	let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
	let charlie = get_account_id_from_seed::<sr25519::Public>("Charlie");

	ExtBuilder::default()
		.with_balances(vec![
			(alice.clone(), 1_000_000_000_000 * KMA),
			(bob.clone(), 1_000_000_000_000 * KMA),
			(charlie.clone(), 1_000_000_000_000 * KMA),
		])
		.with_authorities(vec![(alice.clone(), get_collator_keys_from_seed("Alice"))])
		.with_collators(vec![alice.clone()])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();

			let author = alice.clone();
			let mut header = seal_header(
				Header::new(
					0,
					Default::default(),
					Default::default(),
					Default::default(),
					Default::default(),
				),
				author.clone(),
			);

			header.digest_mut().pop(); // pop the seal off.
			calamari_runtime::System::initialize(&1, &Default::default(), header.digest());
			assert_eq!(Authorship::author().unwrap(), author);

			let call = Call::Balances(pallet_balances::Call::transfer {
				dest: sp_runtime::MultiAddress::Id(charlie),
				value: 10 * KMA,
			});

			let len = 10;
			let info = info_from_weight(100);
			let maybe_pre = ChargeTransactionPayment::<Runtime>::from(0)
				.pre_dispatch(&bob, &call, &info, len)
				.unwrap();

			let res = call.clone().dispatch(Origin::signed(bob));

			let post_info = match res {
				Ok(info) => info,
				Err(err) => err.post_info,
			};

			let _res = ChargeTransactionPayment::<Runtime>::post_dispatch(
				Some(maybe_pre),
				&info,
				&post_info,
				len,
				&res.map(|_| ()).map_err(|e| e.error),
			);

			let rewarded_amount = Balances::free_balance(alice) - 1_000_000_000_000 * KMA;
			println!("The rewarded_amount is: {:?}", rewarded_amount);

			let p = Percent::from_percent(60);
			let actual_fee =
				TransactionPayment::compute_actual_fee(len as u32, &info, &post_info, 0);
			assert_eq!(rewarded_amount, p * actual_fee);

			// Treasury gets 40% of fee
			let p = Percent::from_percent(40);
			assert_eq!(
				Balances::free_balance(Treasury::account_id()),
				p * actual_fee
			);
		});
}

#[test]
fn root_can_change_default_xcm_vers() {
	ExtBuilder::default().build().execute_with(|| {
		// Root sets the defaultXcm
		assert_ok!(PolkadotXcm::force_default_xcm_version(
			root_origin(),
			Some(2)
		));
	})
}
