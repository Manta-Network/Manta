//! Calamari Parachain CLI
#![warn(missing_docs)]

mod chain_specs;
#[macro_use]
mod service;
mod cli;
mod command;

fn main() -> sc_cli::Result<()> {
	command::run()
}
