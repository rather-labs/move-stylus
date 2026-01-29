// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

module hello_world::delegated_counter;

use stylus::{
    tx_context::TxContext, 
    object::{Self, UID}, 
    transfer::{Self}, 
    contract_calls::{Self}
};
use hello_world::delegated_counter_interface as dci;

public struct Counter has key {
    id: UID,
    owner: address,
    value: u64,
    contract_address: address,
}

entry fun create(contract_logic: address, ctx: &mut TxContext) {
  transfer::share_object(Counter {
    id: object::new(ctx),
    owner: ctx.sender(),
    value: 25,
    contract_address: contract_logic,
  });
}

/// Increment a counter by 1.
entry fun increment(counter: &mut Counter) {
    let delegated_counter = dci::new(
        contract_calls::new(counter.contract_address)
            .delegate()
    );
    let res = delegated_counter.increment(&mut counter.id);
    assert!(res.succeded(), 33);
}

entry fun increment_modify_before(counter: &mut Counter) {
    counter.value = counter.value + 10;
    let delegated_counter = dci::new(
        contract_calls::new(counter.contract_address)
            .delegate()
    );
    let res = delegated_counter.increment(&mut counter.id);
    assert!(res.succeded(), 33);
}

entry fun increment_modify_after(counter: &mut Counter) {
    let delegated_counter = dci::new(
        contract_calls::new(counter.contract_address)
            .delegate()
    );
    let res = delegated_counter.increment(&mut counter.id);
    assert!(res.succeded(), 33);
    counter.value = counter.value + 20;
}

entry fun increment_modify_before_after(counter: &mut Counter) {
    counter.value = counter.value + 10;
    let delegated_counter = dci::new(
        contract_calls::new(counter.contract_address)
            .delegate()
    );
    let res = delegated_counter.increment(&mut counter.id);
    assert!(res.succeded(), 33);
    counter.value = counter.value + 20;
}

/// Read counter.
entry fun read(counter: &Counter): u64 {
    counter.value
}

/// Read counter.
entry fun logic_address(counter: &Counter): address {
    counter.contract_address
}

/// Change the address where the delegated calls are made.
entry fun change_logic(counter: &mut Counter, logic_address: address) {
    counter.contract_address = logic_address;
}

/// Set value (only runnable by the Counter owner)
entry fun set_value(counter: &mut Counter, value: u64, ctx: &TxContext) {
    assert!(counter.owner == ctx.sender(), 0);
    let delegated_counter = dci::new(
        contract_calls::new(counter.contract_address)
            .delegate()
    );
    delegated_counter.set_value(&mut counter.id, value);
}
