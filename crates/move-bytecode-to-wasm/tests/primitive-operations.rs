use alloy::{dyn_abi::SolType, sol, sol_types::SolCall};
use common::{runtime_sandbox::RuntimeSandbox, translate_test_package};

mod common;

fn run_test(runtime: &RuntimeSandbox, call_data: Vec<u8>, expected_result: Vec<u8>) {
    let (result, return_data) = runtime.call_entrypoint(call_data);
    assert_eq!(result, 0);
    assert_eq!(return_data, expected_result);
}

#[test]
fn test_bool() {
    const MODULE_NAME: &str = "bool_type";
    const SOURCE_PATH: &str = "tests/primitive-operations/bool.move";

    sol!(
        #[allow(missing_docs)]
        function notTrue() external returns (bool);
        function not(bool x) external returns (bool);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = notTrueCall::abi_encode(&notTrueCall::new(()));
    let expected_result = <sol!((bool,))>::abi_encode_params(&(false,));
    run_test(&runtime, data, expected_result);

    let data = notCall::abi_encode(&notCall::new((false,)));
    let expected_result = <sol!((bool,))>::abi_encode_params(&(true,));
    run_test(&runtime, data, expected_result);

    let data = notCall::abi_encode(&notCall::new((true,)));
    let expected_result = <sol!((bool,))>::abi_encode_params(&(false,));
    run_test(&runtime, data, expected_result);
}
