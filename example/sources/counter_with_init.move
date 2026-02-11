// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: BUSL-1.1

module hello_world::counter_with_init;

use stylus::{
    tx_context::TxContext, 
    object::{Self, UID}, 
    transfer::{Self}
};

public struct Counter has key {
    id: UID,
    owner: address,
    value: u64
}

entry fun init(ctx: &mut TxContext) {

  let counter = Counter {
    id: object::new(ctx),
    owner: ctx.sender(),
    value: 25
  };

  transfer::transfer(counter, ctx.sender());
}

/// Increment a counter by 1.
#[ext(owned_objects(counter))]
entry fun increment(counter: &mut Counter) {
    counter.value = counter.value + 1;
}

/// Read counter.
#[ext(owned_objects(counter))]
entry fun read(counter: &Counter): u64 {
    counter.value
}

/// Set value
#[ext(owned_objects(counter))]
entry fun set_value(counter: &mut Counter, value: u64) {
    counter.value = value;
}
