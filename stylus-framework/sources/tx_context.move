// Copyright (c) Mysten Labs, Inc.
// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// This module implements the Transaction Context for the Stylus framework.
/// It provides access to environmental data of the current transaction and the 
/// underlying block.
module stylus::tx_context;

/// Information about the transaction currently being executed.
///
/// This is a privileged object created by the VM and passed to module entry points.
/// It cannot be manually constructed, ensuring environmental data is authentic.
public struct TxContext has drop {}

/// Returns the address of the account that signed the current transaction.
/// Equivalent to Solidity's `msg.sender`.
public fun sender(_self: &TxContext): address {
    native_sender()
}
native fun native_sender(): address;

/// Returns the amount of ETH (in Wei) sent with the current message.
/// Equivalent to Solidity's `msg.value`.
public fun value(_self: &TxContext): u256 {
    native_msg_value()
}
native fun native_msg_value(): u256;

/// Returns the height (number) of the current block.
public fun block_number(_self: &TxContext): u64 {
    native_block_number()
}
native fun native_block_number(): u64;

/// Returns the base fee of the current block (EIP-1559).
public fun block_basefee(_self: &TxContext): u256 {
    native_block_basefee()
}
native fun native_block_basefee(): u256;

/// Returns the maximum amount of gas allowed in the current block.
public fun block_gas_limit(_self: &TxContext): u64 {
    native_block_gas_limit()
}
native fun native_block_gas_limit(): u64;

/// Returns the block's arrival time as Unix seconds since the epoch.
public fun block_timestamp(_self: &TxContext): u64 {
    native_block_timestamp()
}
native fun native_block_timestamp(): u64;

/// Returns the EIP-155 identifier of the current network chain.
public fun chain_id(_self: &TxContext): u64 {
    native_chain_id()
}
native fun native_chain_id(): u64;

/// Returns the gas price specified by the transaction.
public fun gas_price(_self: &TxContext): u256 {
    native_gas_price()
}
native fun native_gas_price(): u256;

/// Generates a globally unique address to be used as an Object ID.
/// 
/// This address is guaranteed to be distinct from any user-controlled 
/// Externally Owned Account (EOA).
public fun fresh_object_address(_ctx: &mut TxContext): address {
    fresh_id()
}
native fun fresh_id(): address;

/// Returns the raw input data (calldata) of the current transaction.
public fun data(_self: &TxContext): vector<u8> {
    native_data()
}
native fun native_data(): vector<u8>;