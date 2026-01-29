// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

module hello_world::delegated_counter_named_id_interface;

use stylus::contract_calls::{ContractCallEmptyResult, CrossContractCall};

#[ext(external_call)]
public struct CounterCall(CrossContractCall) has drop;

public fun new(configuration: CrossContractCall): CounterCall {
    CounterCall(configuration)
}

#[ext(external_call)]
public native fun increment(self: &CounterCall): ContractCallEmptyResult;

#[ext(external_call)]
public native fun set_value(self: &CounterCall, value: u64): ContractCallEmptyResult;
