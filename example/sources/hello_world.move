module hello_world::hello_world;

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
    // id: object::new(ctx),
    id: @0x1234,
    owner: ctx.sender(),
    value: 0xFFFFFFFF
  };


  transfer::share_object(new_counter);

  /*
   let new_counter2 = Counter {
    // id: object::new(ctx),
    id: @0x1234,
    owner: ctx.sender(),
    value: 0xFFFFFFFF
  };


  storage::save_in_slot(new_counter2, 20);
  */
}

/// Increment a counter by 1.
public fun increment(counter: &mut Counter) {
    counter.value = counter.value + 1;
}
