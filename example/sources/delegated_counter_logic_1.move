module hello_world::delegated_counter_logic_1;

use stylus::tx_context::TxContext;
use stylus::object::UID;

#[ext(external_struct, module_name = b"delegated_counter", address = @0x0)]
public struct Counter has key {
    id: UID,
    owner: address,
    value: u64,
    contract_address: address,
}

/// Increment a counter by 2.
entry fun increment(counter: &mut Counter) {
    counter.value = counter.value + 1;
}

