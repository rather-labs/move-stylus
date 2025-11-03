module hello_world::delegated_counter_named_id_logic_2;

use stylus::tx_context::TxContext;
use stylus::object::NamedId;

public struct COUNTER_ has key {}

#[ext(external_struct, module_name = b"delegated_counter_named_id", address = @0x0)]
public struct Counter has key {
    id: NamedId<COUNTER_>,
    owner: address,
    value: u64,
    contract_address: address,
}

/// Increment a counter by 2.
entry fun increment(counter: &mut Counter) {
    counter.value = counter.value + 2;
}

/// Set value (only runnable by the Counter owner)
entry fun set_value(counter: &mut Counter, value: u64, ctx: &TxContext) {
    assert!(counter.owner == ctx.sender(), 0);
    counter.value = value * 2;
}
