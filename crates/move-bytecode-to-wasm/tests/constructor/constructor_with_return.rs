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
fn test_constructor_with_return(
    #[with(
        "constructor_with_return",
        "tests/constructor/move_sources/constructor_with_return.move"
    )]
    runtime: RuntimeSandbox,
) {
    let call_data = constructorCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
}
