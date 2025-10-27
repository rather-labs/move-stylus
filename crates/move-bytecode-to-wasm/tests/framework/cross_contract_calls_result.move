module test::cross_contract_calls_result;

use test::callee_contract_interface as cci;
use test::callee_contract_interface::{Foo, Bar};
use stylus::contract_calls as contract_calls;

// ==============================================
// Common cross contract calls with empty result
// ==============================================

entry fun cc_call_view_res_1(contract_address: address): u64 {
    let cross_call = cci::new(contract_calls::new(contract_address));
    cross_call.call_view_1().get_result()
}

entry fun cc_call_view_res_2(contract_address: address): Foo {
    let cross_call = cci::new(contract_calls::new(contract_address));
    cross_call.call_view_2().get_result()
}

entry fun cc_call_view_res_3(contract_address: address): Bar {
    let cross_call = cci::new(contract_calls::new(contract_address));
    cross_call.call_view_3().get_result()
}

entry fun cc_call_view_res_4(contract_address: address): vector<u8> {
    let cross_call = cci::new(contract_calls::new(contract_address));
    cross_call.call_view_4().get_result()
}

entry fun cc_call_pure_res_1(contract_address: address): u64 {
    let cross_call = cci::new(contract_calls::new(contract_address));
    cross_call.call_pure_1().get_result()
}

entry fun cc_call_pure_res_2(contract_address: address): Foo {
    let cross_call = cci::new(contract_calls::new(contract_address));
    cross_call.call_pure_2().get_result()
}

entry fun cc_call_pure_res_3(contract_address: address): Bar {
    let cross_call = cci::new(contract_calls::new(contract_address));
    cross_call.call_pure_3().get_result()
}

entry fun cc_call_pure_res_4(contract_address: address): vector<u8> {
    let cross_call = cci::new(contract_calls::new(contract_address));
    cross_call.call_pure_4().get_result()
}

entry fun cc_call_view_pure_res_1(contract_address: address): u64 {
    let cross_call = cci::new(contract_calls::new(contract_address));
    cross_call.call_view_pure_1().get_result()
}

entry fun cc_call_view_pure_res_2(contract_address: address): Foo {
    let cross_call = cci::new(contract_calls::new(contract_address));
    cross_call.call_view_pure_2().get_result()
}

entry fun cc_call_view_pure_res_3(contract_address: address): Bar {
    let cross_call = cci::new(contract_calls::new(contract_address));
    cross_call.call_view_pure_3().get_result()
}

entry fun cc_call_view_pure_res_4(contract_address: address): vector<u8> {
    let cross_call = cci::new(contract_calls::new(contract_address));
    cross_call.call_view_pure_4().get_result()
}
