module hello_world::counter_with_init;

use stylus::{
    tx_context::TxContext, 
    object::{Self, UID}, 
    transfer::{Self}, 
    types as types
};

public struct Counter has key {
    id: UID,
    owner: address,
    value: u64
}

public struct COUNTER_WITH_INIT has drop {}

entry fun init(otw: COUNTER_WITH_INIT, ctx: &mut TxContext) {

  assert!(types::is_one_time_witness(&otw), 0);

  let counter = Counter {
    id: object::new(ctx),
    owner: ctx.sender(),
    value: 25
  };

  transfer::transfer(counter, ctx.sender());
}

/// Increment a counter by 1.
entry fun increment(counter: &mut Counter) {
    counter.value = counter.value + 1;
}

/// Read counter.
entry fun read(counter: &Counter): u64 {
    counter.value
}

/// Set value
entry fun set_value(counter: &mut Counter, value: u64) {
    counter.value = value;
}
