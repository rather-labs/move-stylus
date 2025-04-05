use alloy::{dyn_abi::SolType, sol, sol_types::SolCall};
use common::{ModuleData, setup_wasmtime_module, translate_test_package};
use walrus::Module;

mod common;

fn run_test(translated_package: &mut Module, call_data: Vec<u8>, expected_result: Vec<u8>) {
    let data = ModuleData {
        data: call_data,
        return_data: vec![],
    };
    let data_len = data.data.len() as i32;

    let (_, mut store, entrypoint) = setup_wasmtime_module(translated_package, data);

    let result = entrypoint.call(&mut store, data_len).unwrap();
    assert_eq!(result, 0);

    let store_data = store.data();

    assert_eq!(store_data.return_data, expected_result);
}

#[test]
fn test_uint_8() {
    let mut translated_package = translate_test_package("tests/primitives");

    sol!(
        #[allow(missing_docs)]
        function getConst() external returns (uint8);
    );

    let data = getConstCall::abi_encode(&getConstCall::new(()));
    let expected_result = <sol!((uint8,))>::abi_encode_params(&(88,));

    run_test(&mut translated_package, data, expected_result);
}
