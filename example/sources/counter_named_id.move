// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: BUSL-1.1

module hello_world::counter_named_id;

use stylus::{
    tx_context::TxContext, 
    object::{Self, NamedId}, 
    transfer::{Self}
};

public struct COUNTER_ {}

public struct Counter has key {
    id: NamedId<COUNTER_>,
    owner: address,
    value: u64
}

entry fun create(ctx: &TxContext) {
  transfer::share_object(Counter {
    id: object::new_named_id<COUNTER_>(),
    owner: ctx.sender(),
    value: 25
  });
}

/// Increment a counter by 1.
#[ext(shared_objects(counter))]
entry fun increment(counter: &mut Counter) {
    counter.value = counter.value + 1;
}


/// Read counter.
#[ext(shared_objects(counter))]
entry fun read(counter: &Counter): u64 {
    counter.value
}

/// Set value (only runnable by the Counter owner)
#[ext(shared_objects(counter))]
entry fun set_value(counter: &mut Counter, value: u64, ctx: &TxContext) {
    assert!(counter.owner == ctx.sender(), 0);
    counter.value = value;
}

