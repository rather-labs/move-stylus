module hello_world::counter_named_id;

use stylus::tx_context::TxContext;
use stylus::object::{Self, NamedId};
use stylus::transfer::{Self};

public struct COUNTER_ has key {}

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
entry fun increment(counter: &mut Counter) {
    counter.value = counter.value + 1;
}


/// Read counter.
entry fun read(counter: &Counter): u64 {
    counter.value
}

/// Set value (only runnable by the Counter owner)
entry fun set_value(counter: &mut Counter, value: u64, ctx: &TxContext) {
    assert!(counter.owner == ctx.sender(), 0);
    counter.value = value;
}

