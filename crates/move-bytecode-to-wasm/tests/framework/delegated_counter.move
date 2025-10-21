module test::delegated_counter;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;
use test::delegated_counter_interface as dci;

public struct Counter has key {
    id: UID,
    owner: address,
    value: u64,
    contract_address: address,
}

entry fun create(contract_logic: address, ctx: &mut TxContext) {
  transfer::share_object(Counter {
    id: object::new(ctx),
    owner: ctx.sender(),
    value: 25,
    contract_address: contract_logic,
  });
}

/// Increment a counter by 1.
entry fun increment(counter: &mut Counter) {
    let delegated_counter = dci::new(counter.contract_address, true);
    delegated_counter.increment(&mut counter.id);
}


/// Read counter.
entry fun read(counter: &Counter): u64 {
    counter.value
}

/// Set value (only runnable by the Counter owner)
entry fun set_value(counter: &mut Counter, value: u64, ctx: &TxContext) {
    assert!(counter.owner == ctx.sender(), 0);
    let delegated_counter = dci::new(counter.contract_address, true);
    delegated_counter.set_value(&mut counter.id, value);
}
