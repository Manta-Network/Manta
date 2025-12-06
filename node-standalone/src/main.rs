// Copyright 2020-2024 Chameleon Network.
// SPDX-License-Identifier: GPL-3.0

//! Chameleon Network standalone node
//!
//! A privacy-focused blockchain with MEV protection.

#![warn(missing_docs)]

mod chain_spec;
mod cli;
mod command;
mod rpc;
mod service;

fn main() -> sc_cli::Result<()> {
    command::run()
}
