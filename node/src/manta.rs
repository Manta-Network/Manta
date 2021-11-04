//! Manta/Calamari Parachain CLI
#![warn(missing_docs)]

mod chain_specs;
mod cli;
mod command;
mod rpc;
mod service;

fn main() -> sc_cli::Result<()> {
	command::run()
}
