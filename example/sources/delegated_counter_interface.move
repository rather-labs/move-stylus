// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

module hello_world::delegated_counter_interface;

use stylus::{
    contract_calls::{ContractCallEmptyResult, CrossContractCall}, 
    object::UID
};

#[ext(external_call)]
public struct CounterCall(CrossContractCall) has drop;

public fun new(configuration: CrossContractCall): CounterCall {
    CounterCall(configuration)
}

#[ext(external_call)]
public native fun increment(self: &CounterCall, counter: &mut UID): ContractCallEmptyResult;

#[ext(external_call)]
public native fun set_value(self: &CounterCall, counter: &mut UID, value: u64): ContractCallEmptyResult;
