# Name Service Pallet

## Formatting Rules

- dependencies in alphabetical order in the `Cargo.toml` and at the top of each file
- prefer explicit imports to glob import syntax i.e. prefer `use::crate::{Ex1, Ex2, ..};` to `use super::*;`

## Description

Implements Name Service

Users can register usernames connected to their public key
Only one registered username can be chosen as primary. Primary usernames can be used instead of keys to do transfers

## Workflow

Register -> Pending Register Storage containing block amount wait time
accept_register -> Push the pending register name to the usernameRecords if the block number has been passed
set_primary_name -> Set registered/owned name as a primary name to be used for transfers

cancel_pending_register -> cancel a pending register
remove_register -> "unregister" a name, this would remove it from the primary, leaving the user without a primary

## History

Manta Network forked it in August 2022 to deploy staking on Calamari Network.

Since January 2021, Moonbeam's team has maintained this Delegated Proof of Stake (DPoS) pallet designed specifically for parachains.

Since April 2021, the development of this pallet has been supported by [a Web3 Foundation grant](https://github.com/w3f/Grants-Program/pull/389). The [first milestone](https://github.com/w3f/Grant-Milestone-Delivery/pull/218) was approved in June 2021.
