//! Setup code for benchmarking.

use crate::service::FullClient;
use solochain_template_runtime as runtime;

use sc_cli::Result;
use sc_client_api::BlockBackend;
use sp_core::{Encode, Pair};
use sp_inherents::{InherentData, InherentDataProvider};
use sp_keyring::Sr25519Keyring;
use sp_runtime::{OpaqueExtrinsic, SaturatedConversion};

use std::{sync::Arc, time::Duration};

/// Generates extrinsics for the `benchmark overhead` command.
///
/// Note: Should only be used for benchmarking.
pub struct RemarkBuilder {
    client: Arc<FullClient>,
}

impl RemarkBuilder {
    /// Creates a new [`Self`] from the given client.
    pub fn new(client: Arc<FullClient>) -> Self {
        Self { client }
    }
}

impl frame_benchmarking_cli::ExtrinsicBuilder for RemarkBuilder {
    fn pallet(&self) -> &str {
        "system"
    }

    fn extrinsic(&self) -> &str {
        "remark"
    }

    fn build(&self, nonce: u32) -> std::result::Result<OpaqueExtrinsic, &'static str> {
        // This is a minimal implementation - benchmarking may not work fully
        // but this allows the node to compile
        Err("Benchmarking not fully implemented")
    }
}

/// Generates inherent data for the `benchmark overhead` command.
///
/// Note: Should only be used for benchmarking.
pub fn inherent_benchmark_data() -> Result<InherentData> {
    let mut inherent_data = InherentData::new();
    let d = Duration::from_millis(0);
    let timestamp = sp_timestamp::InherentDataProvider::new(d.into());

    futures::executor::block_on(timestamp.provide_inherent_data(&mut inherent_data))
        .map_err(|e| format!("creating inherent data: {:?}", e))?;
    Ok(inherent_data)
}
