module hello_world::counter;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;
use stylus::storage as storage;

public struct Counter has key {
    // TODO: This should be a UID but we need to handle that case specifically
    id: address,
    owner: address,
    value: u64
}

public fun create(ctx: &mut TxContext) {
  let new_counter = Counter {
    id: @0x1234,
    owner: ctx.sender(),
    value: 25
  };

  transfer::share_object(new_counter);
}

/// Increment a counter by 1.
public fun increment(counter: &mut Counter) {
    counter.value = counter.value + 1;
}

/// Read counter.
public fun read(counter: &Counter): u64 {
    counter.value
}

/// Set value (only runnable by the Counter owner)
public fun set_value(counter: &mut Counter, value: u64, ctx: &TxContext) {
    assert!(counter.owner == ctx.sender(), 0);
    counter.value = value;
}
