// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: BUSL-1.1

/// This module implements the Cross-Contract Call (CCC) infrastructure for the Stylus framework.
/// It provides a builder-pattern configuration for defining call parameters (gas, value, delegation)
/// and generic wrappers for safely handling EVM execution results.

module stylus::contract_calls;

/// Error code indicating that a cross-contract call has reverted or failed.
const ECallFailed: u64 = 101;

/// Alias for checking success on empty return results.
public use fun empty_result_succeded as ContractCallEmptyResult.succeded;

/// Configuration for a cross-contract call.
///
/// This struct uses a builder pattern to define the target address,
/// gas limits, attached ETH value, and execution context (Direct vs Delegate).
public struct CrossContractCall has drop, copy {
    contract_address: address,
    delegate: bool,
    gas: u64,
    value: u256,
}

/// Initializes a new call configuration for the specified contract address.
///
/// By default, the call is non-delegated, carries 0 value, and is
/// allocated the maximum possible gas.
public fun new(contract_address: address): CrossContractCall {
    CrossContractCall {
        contract_address,
        delegate: false,
        gas: 0xffffffffffffffff, // Max u64 gas
        value: 0,
    }
}

/// Sets a custom gas limit for the call.
public fun gas(mut self: CrossContractCall, gas: u64): CrossContractCall {
    self.gas = gas;
    self
}

/// Attaches a specific amount of ETH (in Wei) to the call.
public fun value(mut self: CrossContractCall, value: u256): CrossContractCall {
    self.value = value;
    self
}

/// Configures the call to be a 'delegatecall'.
///
/// The target code will execute using the storage and context of the
/// current contract. This is a permanent toggle for this configuration instance.
public fun delegate(mut self: CrossContractCall): CrossContractCall {
    self.delegate = true;
    self
}

/// A wrapper for the result of a cross-contract call that returns a value.
///
/// The `code` field indicates the success or failure status returned
/// by the VM (0 for success).
public struct ContractCallResult<RESULT> has drop {
    code: u8,
    result: RESULT,
}

/// Returns true if the cross-contract call completed without reverting.
public fun succeded<T>(self: &ContractCallResult<T>): bool {
    self.code == 0
}

/// Unwraps and returns the call result.
///
/// Aborts with `ECallFailed` if the call did not succeed.
public fun get_result<T>(self: ContractCallResult<T>): T {
    let ContractCallResult { code, result } = self;
    assert!(code == 0, ECallFailed);
    result
}

/// A wrapper for the result of a cross-contract call that returns no value.
public struct ContractCallEmptyResult has drop {
    code: u8,
}

/// Returns true if the void cross-contract call completed without reverting.
public fun empty_result_succeded(self: &ContractCallEmptyResult): bool {
    self.code == 0
}
