module hello_world::delegated_counter_interface;

use stylus::contract_calls::ContractCallEmptyResult;
use stylus::object::UID;

#[ext(external_call)]
public struct CounterCall has drop {
    contract_address: address,
    delegate: bool,
}

public fun new(contract_address: address, delegate: bool): CounterCall {
    CounterCall {
        contract_address,
        delegate,
    }
}

#[ext(external_call)]
public native fun increment(self: &CounterCall, counter: &mut UID): ContractCallEmptyResult;

#[ext(external_call)]
public native fun set_value(self: &CounterCall, counter: &mut UID, value: u64): ContractCallEmptyResult;
