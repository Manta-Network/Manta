#![allow(non_camel_case_types)]
use manta_primitives::Balance;

pub const MA: Balance = 1_000_000_000_000_000_000; // 18 decimal
pub const cMA: Balance = MA / 100; // 16 decimal, cent-MA
pub const mMA: Balance = MA / 1_000; // 15 decimal, milli-MA
pub const uMA: Balance = MA / 1_000_000; // 12 decimal, micro-MA

pub const fn deposit(items: u32, bytes: u32) -> Balance {
	items as Balance * 15 * mMA + (bytes as Balance) * 6 * mMA // TODO: revisit the storage cost here
}
