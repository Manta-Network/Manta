//! Substrate Node Template CLI library.
#![warn(missing_docs)]
// suppressing linter on complex Result
// leaving a FIXME; and will be fixed in future
#![allow(clippy::type_complexity)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;
mod rpc;

fn main() -> sc_cli::Result<()> {
	command::run()
}
