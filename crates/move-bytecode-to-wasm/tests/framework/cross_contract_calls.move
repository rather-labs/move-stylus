module test::cross_contract_calls;

use test::callee_contract_interface as cci;
use test::callee_contract_interface::{Foo, Bar};
use stylus::contract_calls as contract_calls;

entry fun cc_call_empty_res_1(contract_address: address): bool {
    let cross_call = cci::new(contract_calls::new(contract_address));
    cross_call.call_empty_res_1().succeded()
}

entry fun cc_call_empty_res_2(contract_address: address, value: u64): bool {
    let cross_call = cci::new(contract_calls::new(contract_address));
    cross_call.call_empty_res_2(value).succeded()
}

entry fun cc_call_empty_res_3(contract_address: address, foo: Foo): bool {
    let cross_call = cci::new(contract_calls::new(contract_address));
    cross_call.call_empty_res_3(foo).succeded()
}

entry fun cc_call_empty_res_4(contract_address: address, bar: Bar): bool {
    let cross_call = cci::new(contract_calls::new(contract_address));
    cross_call.call_empty_res_4(bar).succeded()
}

entry fun cc_call_empty_res_5(contract_address: address, value: vector<u8>): bool {
    let cross_call = cci::new(contract_calls::new(contract_address));
    cross_call.call_empty_res_5(value).succeded()
}

entry fun cc_call_empty_res_1_with_gas(contract_address: address, gas: u64): bool {
    let cross_call = cci::new(
        contract_calls::new(contract_address)
            .gas(gas)
    );
    cross_call.call_empty_res_1().succeded()
}

entry fun cc_call_empty_res_2_with_gas(contract_address: address, gas: u64, value: u64): bool {
    let cross_call = cci::new(
        contract_calls::new(contract_address)
            .gas(gas)
    );
    cross_call.call_empty_res_2(value).succeded()
}

entry fun cc_call_empty_res_3_with_gas(contract_address: address, gas: u64, foo: Foo): bool {
    let cross_call = cci::new(
        contract_calls::new(contract_address)
            .gas(gas)
    );
    cross_call.call_empty_res_3(foo).succeded()
}

entry fun cc_call_empty_res_1_payable(contract_address: address, payable_value: u256): bool {
    let cross_call = cci::new(
        contract_calls::new(contract_address)
            .value(payable_value)
    );
    cross_call.call_empty_res_1_payable().succeded()
}

entry fun cc_call_empty_res_2_payable(contract_address: address, payable_value: u256, value: u64): bool {
    let cross_call = cci::new(
        contract_calls::new(contract_address)
            .value(payable_value)
    );
    cross_call.call_empty_res_2_payable(value).succeded()
}

entry fun cc_call_empty_res_3_payable(contract_address: address, payable_value: u256, foo: Foo): bool {
    let cross_call = cci::new(
        contract_calls::new(contract_address)
            .value(payable_value)
    );
    cross_call.call_empty_res_3_payable(foo).succeded()
}

entry fun cc_call_empty_res_4_payable(contract_address: address, payable_value: u256, bar: Bar): bool {
    let cross_call = cci::new(
        contract_calls::new(contract_address)
            .value(payable_value)
    );
    cross_call.call_empty_res_4_payable(bar).succeded()
}

entry fun cc_call_empty_res_5_payable(contract_address: address, payable_value: u256, value: vector<u8>): bool {
    let cross_call = cci::new(
        contract_calls::new(contract_address)
            .value(payable_value)
    );
    cross_call.call_empty_res_5_payable(value).succeded()
}

entry fun cc_call_empty_res_1_payable_gas(contract_address: address, payable_value: u256, gas: u64): bool {
    let cross_call = cci::new(
        contract_calls::new(contract_address)
            .value(payable_value)
            .gas(gas)
    );
    cross_call.call_empty_res_1_payable().succeded()
}

entry fun cc_call_empty_res_2_payable_gas(contract_address: address, payable_value: u256, gas: u64, value: u64): bool {
    let cross_call = cci::new(
        contract_calls::new(contract_address)
            .value(payable_value)
            .gas(gas)
    );
    cross_call.call_empty_res_2_payable(value).succeded()
}

entry fun cc_call_empty_res_3_payable_gas(contract_address: address, payable_value: u256, gas: u64, foo: Foo): bool {
    let cross_call = cci::new(
        contract_calls::new(contract_address)
            .value(payable_value)
            .gas(gas)
    );
    cross_call.call_empty_res_3_payable(foo).succeded()
}
