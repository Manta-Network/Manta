#![allow(non_camel_case_types)]
use manta_primitives::Balance;

pub const KMA: Balance = 1_000_000_000_000; // 12 decimal
pub const cKMA: Balance = KMA / 100; // 10 decimal, cent-MA
pub const mKMA: Balance = KMA / 1_000; // 9 decimal, milli-MA
pub const uKMA: Balance = KMA / 1_000_000; // 6 decimal, micro-MA

pub const fn deposit(items: u32, bytes: u32) -> Balance {
	items as Balance * 15 * mKMA + (bytes as Balance) * 6 * mKMA // TODO: revisit the storage cost here
}
