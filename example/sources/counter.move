module hello_world::counter;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;

public struct Counter has key {
    id: UID,
    owner: address,
    value: u64
}

public fun create(ctx: &mut TxContext) {
  transfer::share_object(Counter {
    id: object::new(ctx),
    owner: ctx.sender(),
    value: 25
  });
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
