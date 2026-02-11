// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: BUSL-1.1

/// This module provides utilities for querying account state on the network.
///
/// These functions are direct wrappers to account related Stylus host functions.
///
/// For the underlying implementation, see:
/// https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/stylus-sdk/src/hostio.rs

module stylus::account;

/// Returns the size of the smart contract bytecode at the specified address.
///
/// This is commonly used to differentiate between an Externally Owned Account (EOA),
/// which returns 0, and a Smart Contract.
public fun get_account_code_size(account_address: address): u32 {
    account_code_size(account_address)
}

/// Returns the native ETH balance of the specified address, denominated in Wei.
///
/// Note: 1 ETH = 10^18 Wei.
public fun get_account_balance(account_address: address): u256 {
    account_balance(account_address)
}

/// Retrieves the size of the code in bytes at the given address.
/// 
/// The semantics are identical to the EVM's [`EXT_CODESIZE`] opcode.
///
/// [`EXT_CODESIZE`]: https://www.evm.codes/#3B
native fun account_code_size(account_address: address): u32;

/// Retrieves the ETH balance in Wei for the account at the given address.
/// 
/// The semantics are identical to the EVM's [`BALANCE`] opcode.
///
/// [`BALANCE`]: https://www.evm.codes/#31
native fun account_balance(account_address: address): u256;