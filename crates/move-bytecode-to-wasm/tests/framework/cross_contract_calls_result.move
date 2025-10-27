module test::cross_contract_calls_result;

use test::callee_contract_interface as cci;
use test::callee_contract_interface::{Foo, Bar};
use stylus::contract_calls as contract_calls;

// ==============================================
// Common cross contract calls with empty result
// ==============================================

entry fun cc_call_res_1(contract_address: address): u64 {
    let cross_call = cci::new(contract_calls::new(contract_address));
    cross_call.call_view_1().get_result()
}

