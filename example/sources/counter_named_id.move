module hello_world::counter_named_id;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::NamedId;
use stylus::transfer as transfer;

public struct COUNTER_ has key {}

public struct Counter has key {
    id: NamedId<COUNTER_>,
    owner: address,
    value: u64
}

public fun create(ctx: &mut TxContext) {
  transfer::share_object(Counter {
    id: object::new_named_id<COUNTER_>(),
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

