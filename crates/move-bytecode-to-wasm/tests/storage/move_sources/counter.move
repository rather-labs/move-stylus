module test::counter;

use stylus::tx_context::TxContext;
use stylus::object::{Self};
use stylus::object::UID;
use stylus::transfer::{Self};

public struct Counter has key {
    id: UID,
    owner: address,
    value: u64
}

entry fun create(ctx: &mut TxContext) {
  transfer::share_object(Counter {
    id: object::new(ctx),
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
