module hello_world::counter_with_init;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;
use stylus::types as types;

public struct Counter has key {
    id: UID,
    owner: address,
    value: u64
}

public struct COUNTER_WITH_INIT has drop {}

fun init(otw: COUNTER_WITH_INIT, ctx: &mut TxContext) {

  assert!(types::is_one_time_witness(&otw), 0);

  let counter = Counter {
    id: object::new(ctx),
    owner: ctx.sender(),
    value: 25
  };

  transfer::transfer(counter, ctx.sender());
}

/// Increment a counter by 1.
public fun increment(counter: &mut Counter) {
    counter.value = counter.value + 1;
}

/// Read counter.
public fun read(counter: &Counter): u64 {
    counter.value
}

/// Set value
public fun set_value(counter: &mut Counter, value: u64, ctx: &TxContext) {
    counter.value = value;
}
