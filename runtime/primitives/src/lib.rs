#![allow(non_upper_case_globals)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::upper_case_acronyms)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod constants;
pub use constants::time;

use sp_runtime::traits::{BlakeTwo256, IdentifyAccount, Verify};

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = sp_runtime::MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Block header type as expected by this runtime.
pub type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
pub type Block = sp_runtime::generic::Block<Header, sp_runtime::OpaqueExtrinsic>;

/// Digest item type.
pub type DigestItem = sp_runtime::generic::DigestItem<Hash>;

// Aura consensus authority.
pub type AuraId = sp_consensus_aura::sr25519::AuthorityId;

// Moment
pub type Moment = u64;
