// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: BUSL-1.1

module hello_world::delegated_counter_logic_2;

use stylus::{
    tx_context::TxContext, 
    object::UID
};

#[ext(external_struct(module_name = b"delegated_counter", address = @0x0))]
public struct Counter has key {
    id: UID,
    owner: address,
    value: u64,
    contract_address: address,
}

/// Increment a counter by 2.
#[ext(shared_objects(counter))]
entry fun increment(counter: &mut Counter) {
    counter.value = counter.value + 2;
}

/// Set value (only runnable by the Counter owner)
#[ext(shared_objects(counter))]
entry fun set_value(counter: &mut Counter, value: u64, ctx: &TxContext) {
    assert!(counter.owner == ctx.sender(), 0);
    counter.value = value * 2;
}
