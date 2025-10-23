module test::callee_contract_interface;

use stylus::contract_calls::{ContractCallEmptyResult, ContractCallResult};
use stylus::object::UID;

#[ext(external_call)]
public struct CrossContractCall has drop {
    contract_address: address,
    delegate: bool,
}

public fun new(contract_address: address, delegate: bool): CrossContractCall {
    CrossContractCall {
        contract_address,
        delegate,
    }
}

#[ext(external_call)]
public native fun increment(self: &CrossContractCall, counter: &mut UID): ContractCallEmptyResult;

#[ext(external_call)]
public native fun set_value(self: &CrossContractCall, counter: &mut UID, value: u64): ContractCallEmptyResult;

#[ext(external_call, view)]
public native fun read(self: &CrossContractCall, counter: &UID): ContractCallResult<u64>;
