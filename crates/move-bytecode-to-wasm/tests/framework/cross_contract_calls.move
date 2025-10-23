module test::cross_contract_calls;

use test::callee_contract_interface as cci;
use test::callee_contract_interface::{Foo, Bar};

entry fun cc_call_empty_res_1(contract_address: address): bool {
    let cross_call = cci::new(contract_address, false);
    cross_call.call_empty_res_1().succeded()
}


entry fun cc_call_empty_res_2(contract_address: address, value: u64): bool {
    let cross_call = cci::new(contract_address, false);
    cross_call.call_empty_res_2(value).succeded()
}

entry fun cc_call_empty_res_3(contract_address: address, foo: Foo): bool {
    let cross_call = cci::new(contract_address, false);
    cross_call.call_empty_res_3(foo).succeded()
}

entry fun cc_call_empty_res_4(contract_address: address, bar: Bar): bool {
    let cross_call = cci::new(contract_address, false);
    cross_call.call_empty_res_4(bar).succeded()
}
