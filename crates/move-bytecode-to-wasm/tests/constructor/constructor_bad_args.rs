use crate::common::runtime;
use alloy_sol_types::{SolCall, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::rstest;

sol!(
    #[allow(missing_docs)]
    function constructor() public view;
);

#[rstest]
#[should_panic]
fn test_constructor_bad_args_1(
    #[with(
        "constructor_bad_args_1",
        "tests/constructor/move_sources/constructor_bad_args_1.move"
    )]
    runtime: RuntimeSandbox,
) {
    let call_data = constructorCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
}

#[rstest]
#[should_panic]
fn test_constructor_bad_args_2(
    #[with(
        "constructor_bad_args_2",
        "tests/constructor/move_sources/constructor_bad_args_2.move"
    )]
    runtime: RuntimeSandbox,
) {
    let call_data = constructorCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
}

#[rstest]
#[should_panic]
fn test_constructor_bad_args_3(
    #[with(
        "constructor_bad_args_3",
        "tests/constructor/move_sources/constructor_bad_args_3.move"
    )]
    runtime: RuntimeSandbox,
) {
    let call_data = constructorCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
}

#[rstest]
#[should_panic]
fn test_constructor_bad_args_4(
    #[with(
        "constructor_bad_args_4",
        "tests/constructor/move_sources/constructor_bad_args_4.move"
    )]
    runtime: RuntimeSandbox,
) {
    let call_data = constructorCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
}

#[rstest]
#[should_panic]
fn test_constructor_bad_args_5(
    #[with(
        "constructor_bad_args_5",
        "tests/constructor/move_sources/constructor_bad_args_5.move"
    )]
    runtime: RuntimeSandbox,
) {
    let call_data = constructorCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
}
